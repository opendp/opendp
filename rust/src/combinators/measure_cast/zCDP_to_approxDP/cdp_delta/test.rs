use super::*;
#[test]
fn test_edge_cases() -> Fallible<()> {
    // negativity checks
    assert!(cdp_delta(-0., 0.).is_err());
    assert!(cdp_delta(0., -0.).is_err());

    assert_eq!(cdp_delta(0., 0.)?, 0.);
    assert_eq!(cdp_delta(0.5, 0.)?, 0.5588356393474351);
    assert!(cdp_delta(0.1, 0.1)? > 0.);
    assert_eq!(cdp_delta(0.1, f64::INFINITY)?, 0.);
    assert_eq!(cdp_delta(f64::INFINITY, 1.)?, 1.0);

    Ok(())
}
