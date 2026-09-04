use crate::model::*;
use crate::observer::*;
use crate::presets::*;
use nalgebra::Vector3;

#[test]
fn gas_layer_switches_abruptly_at_boundary() {
    assert_eq!(gas_visibility(200.0, 16.0), 0.0);
    assert_eq!(gas_visibility(128.01, 16.0), 0.0);
    assert_eq!(gas_visibility(128.0, 16.0), 1.0);
    assert_eq!(gas_visibility(127.99, 16.0), 1.0);
}

#[test]
fn flying_star_keeps_four_pixel_radius_after_scaling() {
    for scale in [0.25, 0.5, 1.0, 2.0] {
        assert!((flying_star_radius(scale) * scale - 4.0).abs() < 1.0e-6);
    }
}

#[test]
fn ground_projection_distinguishes_zenith_and_below_horizon() {
    let latitude = 25.0_f64.to_radians();
    let up = Vector3::new(latitude.cos(), 0.0, latitude.sin());
    let (_, altitude) = horizontal_coordinates(up, 0.0);
    assert!((altitude - std::f64::consts::FRAC_PI_2).abs() < 1.0e-7);
    assert!(horizontal_coordinates(-up, 0.0).1 < 0.0);
}

#[test]
fn heat_follows_inverse_square_distance() {
    let mut preset = SimulationPresetFactory::create(0);
    let star = &mut preset.stars[0];
    star.position = Vector3::zeros();
    let mut planet = Planet {
        position: Vector3::new(50.0, 0.0, 0.0),
        velocity: Vector3::zeros(),
    };
    let near = relative_heat(star, &planet);
    assert!((near - 16.0).abs() < 1.0e-10);
    planet.position.x = 100.0;
    assert!((relative_heat(star, &planet) - near / 4.0).abs() < 1.0e-10);
    planet.position.x = 128.01;
    assert_eq!(relative_heat(star, &planet), 0.0);
    planet.position = Vector3::zeros();
    assert!(relative_heat(star, &planet).is_finite());
}

#[test]
fn rotation_changes_sun_altitude() {
    let direction = Vector3::new(1.0, 0.0, 0.0);
    assert!(horizontal_coordinates(direction, 0.0).1 > 0.0);
    assert!(horizontal_coordinates(direction, std::f64::consts::PI).1 < 0.0);
}
