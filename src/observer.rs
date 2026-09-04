use crate::model::*;
use crate::presets::STAR_MASS;
use nalgebra::Vector3;
pub(crate) fn horizontal_coordinates(direction: Vector3<f64>, phase: f64) -> (f64, f64) {
    let latitude = 25.0_f64.to_radians();
    let up = Vector3::new(
        phase.cos() * latitude.cos(),
        phase.sin() * latitude.cos(),
        latitude.sin(),
    );
    let east = Vector3::new(-phase.sin(), phase.cos(), 0.0);
    let north = up.cross(&east);
    let unit = direction.try_normalize(1.0e-12).unwrap_or(Vector3::zeros());
    (
        unit.dot(&east).atan2(unit.dot(&north)),
        unit.dot(&up).clamp(-1.0, 1.0).asin(),
    )
}

// 小説をモチーフにした視覚演出であり、実際の恒星大気の物理モデルではない。
// 通常の距離では飛星のまま、半径の8倍まで接近すると球へ瞬時に切り替える。
// 境界の前後で混ぜないことが、この演出の要点。
pub(crate) const SUN_REVEAL_RADIUS_MULTIPLIER: f64 = 8.0;

pub(crate) fn gas_visibility(distance: f64, radius: f64) -> f32 {
    if distance <= radius.max(1.0) * SUN_REVEAL_RADIUS_MULTIPLIER {
        1.0
    } else {
        0.0
    }
}

// 描画先の縮小率を補正し、実際のパネル上で半径4px以上の飛星を保証する。
pub(crate) fn flying_star_radius(panel_scale: f32) -> f32 {
    4.0 / panel_scale.max(0.01)
}

pub(crate) fn relative_heat(star: &Star, planet: &Planet) -> f64 {
    // 飛星の間は光・熱を届けないという演出上のルール。重力は従来通り作用する。
    if gas_visibility((star.position - planet.position).norm(), star.radius) == 0.0 {
        return 0.0;
    }
    let distance = (star.position - planet.position)
        .norm()
        .max(star.radius)
        .max(1.0);
    (star.mass / STAR_MASS) * (200.0 / distance).powi(2)
}
