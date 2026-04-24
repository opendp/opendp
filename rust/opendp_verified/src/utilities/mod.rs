use dashu::{integer::UBig, rational::RBig};

fn gcd_ubig(mut a: UBig, mut b: UBig) -> UBig {
    while !b.is_zero() {
        let r = &a % &b;
        a = b;
        b = r;
    }
    a
}

pub fn div_rbig_by_ubig_exact(numer: &UBig, denom: &UBig, k: &UBig) -> RBig {
    assert!(!k.is_zero(), "division by zero");

    if numer.is_zero() {
        return RBig::ZERO;
    }

    let g = gcd_ubig(numer.clone(), k.clone());
    let n_red = numer / &g;
    let k_red = k / g;

    RBig::from_parts(n_red.into(), (denom * k_red).into())
}
