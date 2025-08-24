# Non-zero VM

# 1. 設計總覽

**目標

* 建立一個**暫存器式、64-bit、little-endian、編譯式**的 VM。
* 全系統採 Zero-Free Universe（ZFU）**：數值、位址、長度、位元組與布林等**值域不含 0**。
* 任何導致「0」的運算或載入，**同步 trap 終止；無例外處理。
* 正常結束以 `HALT nzint`，建議**成功碼固定 1**（系統不使用 0）。

非目標（MVP）

* 不支援多模組連結、動態載入、JIT 與複雜系統呼叫。
* 不相容 POSIX；不提供傳統 `exit(0)` 與 NUL 串。

# 2. 值域與型別

核心值域

* `i64⁺≠0`：64 位整數，**不含 0**；二補數語意，整數溢位採 wraparound**（模 2⁶⁴）。
* `f64≠0.0, !NaN`：64 位浮點，**不得為 ±0.0 或 NaN**；∞ 允許。
* `nzbyte ∈ [1..=255]`：記憶體元素，嚴禁 0x00。
* `nzlen ∈ [1..=2⁶⁴−1]`：長度不可為 0。
* `nzaddr ∈ [1..=2⁶⁴−1]`：位址從 **1 起跳，0 位址不存在。
* `bool₂`：布林以 `TRUE=1`、`FALSE=-1` 表示（避免 0/1 慣例）。
* `⊥`（bottom/undefined）：唯一合法的「無」語意；**不以 0 表示**。

指標

* 指標本身屬 `nzaddr`，**不可為 0**；不允許 NULL。
* 「可缺席」抽象以 `⊥` 或 sum type 處理，不以 0 或空長度代替。

字串與容器

* 不使用 NUL 結尾。字串/陣列以（指標, nzlen）或長度前綴表示；**空字串/空陣列不合法**。
* 任何會生成 0 長度或含 0x00 的表示法皆屬規格違反。

# 3. 記憶體模型與 NZCodec

線性記憶體

* 單一連續區，以 `nzbyte` 為元素；最低有效位址為 1。
* 任意寫入若導致 0x00 落地 → `TRAP_NZ_BYTE`**。
* 讀寫 64-bit 整數/浮點使用 **little-endian 邏輯值，但以 NZCodec 編碼後實體存取。

NZCodec（版本 1）

* 避免 0x00 出現在記憶體內。
* 編碼：`1..=254 → [b]`；`0 → [255,1]`；`255 → [255,2]`。
* 解碼：遇 `255` 讀後 1 位元組標籤復原原值；其他值直譯。
* .data 區與任何 I/O 介面均須使用 NZCodec。
* 規格要求載入器於裝載 .data 前**全域掃描**，若出現 0x00 → 拒載並回報 `BAD_MODULE`。

# 4. 執行模型與暫存器

暫存器檔

* 整數：`r0..r15`（64-bit）

  * 保留：`r15=SP`（棧指標）、`r14=FP`（框架指標，可選）、`r13=TP`（執行緒指標，預留）
  * 返回值：`r0`
* 浮點：`f0..f15`（64-bit IEEE-754）

  * 返回值：`f0`

程式計數器（PC）

* 指向 .text 位元碼的位元組位址。指令可變長，PC 總是落在**指令邊界**。

棧

* 向**低位址**成長，8-byte 對齊。
* 棧框（frame）大小由符號表宣告（固定大小）；呼叫時顯式建/拆框。
* 棧記憶體亦為 `nzbyte`；任一棧寫入不可產生 0x00。

# 5. 呼叫慣例（ABI，MVP）

參數傳遞

* 整數參數：`r1..r6`；超過者**自右向左**推入棧。
* 浮點參數：`f1..f6`；超過者推入棧。
* 所有參數必屬非零值域；呼叫點若來源可能不明，須先完成「非零化」（見 §8 Verifier）。

返回

* 整數用 `r0`；浮點用 `f0`；返回值必為非零。
* 呼叫者保存（caller-save）：`r1..r6, f1..f6`；其餘由被呼叫者保存。

序言/返還

* 被呼叫者負責建立/回收固定大小之 frame；對齊 8。

# 6. 指令集（MVP）

> 總則**：凡**寫回**目標暫存器之指令，若結果為 0，**同步 trap：`ZERO_RESULT`**。
> 除法除數為 0（理論上不應出現），**trap：`DIV_ZERO`**。
> 浮點結果如為 `±0.0` 或 `NaN`，**trap：`ZERO_RESULT` / `ILLEGAL_FP`**（實作可合併視為 `ZERO_RESULT`）。

## 6.1 常數與搬移

* `ICONST rD, imm64≠0`：載入非零整數常數。
* `FCONST fD, imm64(b64 of f64; ≠±0.0, !NaN)`：載入非零浮點常數。
* `MOV rD, rS` / `FMOV fD, fS`：搬移（來源已具非零保證）。

## 6.2 記憶體

* `LOADNZ rD, [rB+off]`：自位址 `rB+off` 讀 8 位元組（NZCodec 解碼）為 `i64`；若解得 0 → trap。
* `FLOADNZ fD, [rB+off]`：同上，浮點；若解得 `±0.0/NaN` → trap。
* `STORE [rB+off], rS`／`FSTORE [rB+off], fS`：以 NZCodec 寫入。
* `NZCHK rS`：防衛性檢查，`rS==0` → trap（理論上不應被觸發，供載入邊界強化用）。

> **位移 `off`**：有號 32-bit，位址必落在已配置之有效區；越界 → `SEGFAULT`。

## 6.3 算術（整數）

* `ADDNZ rD, rA, rB`、`SUBNZ`、`MULNZ`、`DIVNZ`（帶符號除法，截位朝 0）。
* 溢位：允許 wraparound；僅對「結果==0」的情形進行 trap。

## 6.4 算術（浮點）

* `FADDNZ fD, fA, fB`、`FSUBNZ`、`FMULNZ`、`FDIVNZ`（遵循 IEEE-754）。
* 若結果為 `±0.0` 或 `NaN` → trap。∞ 允許。

## 6.5 比較與分支

* `CMP rA, rB`：將比較序（`LT/ EQ/ GT`）編碼成非零序數寫入 `r0`：

  * `LT → 1`、`EQ → 2`、`GT → 3`。
* `FCMP fA, fB`：同上；任一為 NaN 屬非法來源，應於載入/運算即 trap。
* `BEQ/BNE/BLT/BLE/BGT/BGE label`：依 `r0` 之序數分支。
* `BRA label`：無條件跳躍。

> **分支目標**：以 **PC 相對**的有號 32-bit 位移；目標須對齊至**指令起始邊界**。

## 6.6 呼叫與返回

* `CALL func_id`：呼叫符號表中之函式。
* `RET`：返回至呼叫點。
* 所有參數/返回值均須於**呼叫點**與**返回點**滿足非零值域。

## 6.7 結束

* `HALT imm64≠0`：立即以非零代碼結束 VM；**建議 1 代表成功**。

# 7. 位元碼編碼（.text）與操作數格式

**指令格式（可變長）

* `OPC`（1 byte）：操作碼。
* `MOD`（1 byte）：變體與旗標；最低位保留為「是否攜帶 imm64」。
* `OPA/OPB/OPC`（各 1 byte）：暫存器索引（低 5 bits）或尋址修飾；不使用之欄位置 0。
* `RSVD`（1 byte）：保留。
* `IMM`（0 或 8 bytes）：little-endian 立即數或位移資料（依 MOD 旗標）。

暫存器索引

* 合法值為 `0..15`；其他值 → `ILLEGAL`。

分支位移

* 以額外 `IMM32` 表示（PC 相對）；組譯時回填；驗證器需檢查落點合法且邊界對齊。

# 8. 驗證器（Verifier）規格目的**：

在執行前阻擋結構與語意錯誤，降低執行期 trap 的不可預期性。

## 8.1 結構檢查（必做）

* 檔頭/區段表合法，區段不重疊、邊界落在檔案內。
* `.data` 內**不得含 0x00**；否則 `BAD_MODULE`。
* `.text`：每條指令長度一致性、立即數存在性、未知 opcode → `ILLEGAL`。
* 分支目標必位於 `.text` 內且對齊指令邊界。
* `CALL func_id` 必對應 `.sym` 表內既有函式。

## 8.2 非零性資料流分析（MVP 推薦）

* lattice：`Unknown < NZ`（逐暫存器/浮點暫存器追蹤）。
* 初值：入口基本塊所有暫存器為 `Unknown`。
* 轉移：

  * `ICONST/FCONST/*NZ/NZCHK` → 目標/來源標記為 `NZ`；
  * `MOV/FMOV` → 目標繼承來源標記；
  * `CMP/FCMP` → `r0` 標記 `NZ`（序數 ∈{1,2,3}）；
  * 其他未知效果 → 回退 `Unknown`。
* 基本塊交會採 meet。
* **政策**：

  * **寬鬆模式（MVP）**：`Unknown` 值用於需要非零的地方時允許載入，但執行期若真為 0 會 trap。
  * **嚴格模式（進階）**：此情況直接拒載。

# 9. 模組與檔案格式（.nzbc）

**檔頭

* `magic="NZVM"`（ASCII 4 bytes，非 0x00）。
* `version=0x0003`。
* `endian=1`（little-endian）。
* `sec_count=u32`：區段數。

區段表（`sec_count` 條）

* `type: u32`：`1=.text`、`2=.data`、`3=.sym`、`4=.strtab`、`5=.meta`。
* `align: u32`：建議對齊（MVP 建議 16）。
* `offset: u64`、`size: u64`：檔內偏移與長度。

各區段語意

* `.text`：位元碼；PC 起點由入口符號指定。
* `.data`：**已經過 NZCodec 編碼**之原始資料切片（不得含 0x00；載入前仍須掃描確認）。
* `.sym`（函式表）：

  * `u32 count`；每筆：

    * `u32 func_id`
    * u32 name_off`（索引至 .strtab`，**不得為 0**）
    * u64 entry_off`（位於 .text`）
    * `u32 frame_size`（8 對齊，**不得為 0**）
    * `u32 rsvd=0`
* `.strtab`：以 0x01 作為分隔符**之名稱字串池；名稱字串採 ASCII 或 UTF-8，但**不得包含 0x00 與 0x01**。
* `.meta`：鍵值表（例如 `zfu=1`、`nzcodec=1`、版本戳記等），同樣不含 0x00；建議以 `key=val` 配對，行間以 0x01 分隔。

**入口點

* .sym 中 name=="main" 的條目視為入口；缺少 → `NO_ENTRY`。

# 10. 錯誤與 Trap 規範

Trap 類型（建議枚舉）

* `OK`：正常結束（由 `HALT` 觸發；外部訊號用途）。
* ZERO_RESULT`：產生了 0（整數/浮點或 `LOADNZ 解得 0）。
* `DIV_ZERO`：除數為 0。
* `NZ_BYTE`：嘗試將 0x00 寫入記憶體。
* `ILLEGAL`：未知或格式錯誤的指令/操作數。
* `BAD_MODULE`：檔頭/區段不合法或 .data 檢查失敗。
* `NO_ENTRY`：缺入口符號。
* `SEGFAULT`：位址/長度越界。

退出碼

* 透過 `HALT nzint`；**不得為 0**；建議 `1` 代表成功。

# 11. 指令語意補充

整數運算

* `ADDNZ/SUBNZ/MULNZ`：結果以 64-bit wraparound；**若結果數值 == 0 → trap**。
* `DIVNZ`：被除數 ÷ 除數，向 0 截斷；除數為 0 → `DIV_ZERO`；結果為 0 → `ZERO_RESULT`。

浮點運算

* 遵循 IEEE-754；但 VM 值域不含 `±0.0` 與 `NaN`。
* 任一來源若為 NaN，應在**載入/上游**即被攔截，否則屬規格違反。
* 結果如為 `±0.0` 或 NaN → `ZERO_RESULT`（或 `ILLEGAL`，依實作歸類）。

比較/分支

* `CMP/FCMP` 不寫旗標，僅將序數寫入 `r0`。
* 分支以 `r0` 值判斷（1/2/3），**不依賴 Z 旗標**，避免與「零值」語意衝突。

載入/存儲

* LOADNZ/FLOADNZ 必做 NZCodec 解碼後的**非零檢查**。
* STORE/FSTORE 必以 NZCodec 寫入，並保證**記憶體不得出現 0x00**。
* 任何以位址或長度為 0 的操作都不可能構造（因 `nzaddr/nzlen`），若出現表示載入器或 Verifier 失職，應整體拒載。

# 12. 區段/字串細節

對齊

* .text/.data 建議 16-byte 對齊；`.sym/.strtab/.meta` 建議 8-byte。

.strtab 字串

* 以 0x01 分隔多個字串；字串內容不含 0x00 與 0x01。
* 建議以 UTF-8 儲存（不含 `U+0000`）；長度不可為 0。
* `name_off` 指向字串起始位址；字串結束以下一個 0x01 或區段末尾判定。

# 13. 版本與相容性

版本欄位

* 檔頭 `version=0x0003`。
* .meta 至少包含：`zfu=1`、`nzcodec=1`。

向前/向後相容

* 未來版本若擴充 opcode 或 MOD 旗標，保留未知位為 0。
* 載入器在面對較新版本時，若關鍵語意不變，得以「警告後載入」；否則拒載並回報 `BAD_MODULE`。

# 14. 安全與沙箱（MVP）

* MVP 僅保留 `HALT`**；不內建 I/O。需要 I/O 時，另行定義 `IO_READ/IO_WRITE` 並強制使用 NZCodec。
* 位址與長度皆為非零域，外加越界檢查，避免任意位址任意長度的記憶體破壞。
* `.data` 及所有輸入皆須經 NZCodec；內部再度寫入時亦須經 NZCodec，確保 0x00 不會出現。

# 15. 測試與合規建議

**單元測試（必要）

* ICONST/FCONST 載入 0 或 ±0.0 → 應拒絕（載入期或執行期 trap）。
* ADDNZ 1 + (-1)`、`MULNZ X * 0（若可構造） → `ZERO_RESULT`。
* `FADDNZ 1.0 + (-1.0)` → `ZERO_RESULT`。
* `LOADNZ` 對應 `.data`：保證無法解出 0；若模擬 I/O 回傳 0 → 應 trap。

結構測試

* 分支落點、符號參照、區段邊界與對齊錯誤 → `ILLEGAL`/`BAD_MODULE`。
* .strtab 若含 0x00 或 0x01 → `BAD_MODULE`。

互通測試

* 同一語意以「強制 *NZ*」與「unchecked + 顯式 `NZCHK`」兩路生成，行為應一致（均不得允許 0 滲透）。

# 16. 可選擴充

* **嚴格 Verifier**：將「可疑來源」全面拒載。
* **位元碼壓縮**：縮短 OPA/OPB/OPC 至 12 bits 彙編欄位，保留簡單直覺。
* **多模組**：導入/導出表與重定位；仍維持 `nzbyte` 與 NZCodec。
* **更豐富型別**：非零無號整數族、固定小整數域（排除 0），以及 `bool` 的原生指令。
