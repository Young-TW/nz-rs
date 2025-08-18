// Ensure the nzint module exists and is declared
mod nzint;
use crate::nzint::NzInt;

fn main() {
    let a = NzInt::new(3).unwrap();
    let b = NzInt::new(-3).unwrap();
    let c = a.checked_add(b);            // Err(NzError::ZeroResult)

    let d = NzInt::new(7).unwrap();
    let e = NzInt::new(2).unwrap();
    let q = d.checked_div(e).unwrap();   // 3 (NzInt), but would be Err if result were 0

    println!("Result of addition: {:?}", c);
    println!("Result of division: {:?}", q);
}
