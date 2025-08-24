// Ensure the nzint module exists and is declared
mod nzint;
mod nzfloat;
mod nzsign;
use crate::nzint::NzInt;
use crate::nzfloat::NzFloat;
use crate::nzsign::nzSign;

fn main() {
    let a = NzInt::new(3).unwrap();
    let b = NzInt::new(-3).unwrap();
    let c = a.checked_add(b);            // Err(NzError::ZeroResult)

    let d = NzInt::new(7).unwrap();
    let e = NzInt::new(2).unwrap();
    let q = d.checked_div(e).unwrap();   // 3 (NzInt), but would be Err if result were 0

    println!("Result of addition: {:?}", c);
    println!("Result of division: {:?}", q);

    let f = NzFloat::new(3.5).unwrap();
    let g = NzFloat::new(-3.5).unwrap();
    let h = f.checked_add(g);            // Err(NzfError::ZeroResult)

    let i = NzFloat::new(7.0).unwrap();
    let j = NzFloat::new(2.0).unwrap();
    let r = i.checked_div(j).unwrap();   // 3.5 (NzFloat), but would be Err if result were 0

    println!("Result of float addition: {:?}", h);
    println!("Result of float division: {:?}", r);

    let k = nzSign::from_bool(true);
    let l = nzSign::from_bool(false);
    println!("Result of nzSign from_bool(true): {:?}", k);
    println!("Result of nzSign from_bool(false): {:?}", l);
    println!("Result of nzSign Pos && Neg: {:?}", k.and(l));
    println!("Result of nzSign Pos || Neg: {:?}", k.or(l));
}
