static CONSTS_GT_6_125: [f64; 8] = [
    2.93243101e-8,
    1.22150334e-6,
    2.84108955e-5,
    3.93552968e-4,
    3.02698812e-3,
    4.83185798e-3,
    -2.64646143e-1,
    8.40016484e-1,
];

static CONSTS_LTE_6_125: [f64; 9] = [
    1.43285448e-7,
    1.22774793e-6,
    1.12963626e-7,
    -5.61530760e-5,
    -1.47697632e-4,
    2.31468678e-3,
    1.15392581e-2,
    -2.32015476e-1,
    8.86226892e-1,
];

pub fn erf_inv(a: f64) -> f64 {
    let t = (-a * a + 1.0).ln();
    let p = if t.abs() > 6.125 {
        // maximum ulp error = 2.35793
        let mut p = 3.03697567e-10;
        for c in CONSTS_GT_6_125 {
            p *= t;
            p += c;
        }
        p
    } else {
        // maximum ulp error = 2.35002
        let mut p = 5.43877832e-9;
        for c in CONSTS_LTE_6_125 {
            p *= t;
            p += c;
        }
        p
    };
    return a * p;
}
