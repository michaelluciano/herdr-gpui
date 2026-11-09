use super::*;

#[test]
fn content_that_fits_gives_its_ratio() {
    let floor = split_floor(200., 40., 600.);
    assert!((floor - 0.4).abs() < 1e-6);
}

#[test]
fn floor_clamps_low_and_high() {
    assert_eq!(split_floor(0., 0., 600.), 0.1);
    assert_eq!(split_floor(10., 10., 600.), 0.1);
    assert_eq!(split_floor(5000., 40., 600.), 0.9);
}

#[test]
fn unusable_available_space_gives_the_minimum() {
    for available in [0., -10., f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(split_floor(300., 40., available), 0.1);
    }
    assert_eq!(split_floor(f32::NAN, 40., 600.), 0.1);
}
