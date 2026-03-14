use crate::traits::samplers::cks20::gcd_ubig;

use dashu::integer::UBig;

fn u(n: u64) -> UBig {
    UBig::from(n)
}

#[test]
fn gcd_basic_identities() {
    assert_eq!(gcd_ubig(u(0), u(0)), u(0));
    assert_eq!(gcd_ubig(u(0), u(7)), u(7));
    assert_eq!(gcd_ubig(u(7), u(0)), u(7));
    assert_eq!(gcd_ubig(u(1), u(0)), u(1));
    assert_eq!(gcd_ubig(u(0), u(1)), u(1));
}

#[test]
fn gcd_symmetry() {
    let a = u(54);
    let b = u(24);
    assert_eq!(gcd_ubig(a.clone(), b.clone()), gcd_ubig(b, a));
}

#[test]
fn gcd_known_values() {
    assert_eq!(gcd_ubig(u(54), u(24)), u(6));
    assert_eq!(gcd_ubig(u(48), u(180)), u(12));
    assert_eq!(gcd_ubig(u(17), u(29)), u(1));
    assert_eq!(gcd_ubig(u(100), u(10)), u(10));
    assert_eq!(gcd_ubig(u(10), u(100)), u(10));
    assert_eq!(gcd_ubig(u(270), u(192)), u(6));
}

#[test]
fn gcd_common_factor_property() {
    let a = u(21);
    let b = u(6);
    let c = u(14);

    let left = gcd_ubig(&a * &c, &b * &c);
    let right = gcd_ubig(a, b) * c;

    assert_eq!(left, right);
}

#[test]
fn gcd_divides_both() {
    let a = u(123456);
    let b = u(7890);
    let g = gcd_ubig(a.clone(), b.clone());

    if !g.is_zero() {
        assert!((&a % &g).is_zero());
        assert!((&b % &g).is_zero());
    } else {
        assert!(a.is_zero() && b.is_zero());
    }
}

#[test]
fn gcd_large_constructed() {
    let q = u(1_000_003);
    let p = u(12345);
    let r = u(67891);

    let a = &p * &q;
    let b = &r * &q;

    assert_eq!(gcd_ubig(a, b), q);
}
