use crate::model::*;
use nalgebra::Vector3;
// 1ステップ分の更新を一箇所にまとめ、ネイティブとWebで同じ物理計算を使う。
pub(crate) fn step(stars: &mut [Star], planet: &mut Planet, dt: f64) {
    let mut forces = vec![Vector3::new(0.0, 0.0, 0.0); stars.len()];

    // 万有引力の計算
    for i in 0..stars.len() {
        for j in 0..stars.len() {
            if i != j {
                let gravity_norm = calculate_gravity_norm(&stars[i], &stars[j]);
                let direction = (stars[j].position - stars[i].position).normalize();
                forces[i] += direction * gravity_norm;
            }
        }
    }

    // 速度と位置の更新
    for (star, force) in stars.iter_mut().zip(forces) {
        let acceleration = force / star.mass;
        star.velocity += acceleration * dt;
        star.position += star.velocity * dt;
    }
    update_planet(planet, stars, dt);
}
pub(crate) const G: f64 = 6.67430e-11;
pub(crate) const FIXED_DT: f64 = 1.0 / 600.0;
pub(crate) fn calculate_gravity_norm(star1: &Star, star2: &Star) -> f64 {
    let distance_squared = (star2.position - star1.position).norm_squared();
    G * star1.mass * star2.mass / distance_squared
}

// 惑星は恒星を動かさない「質量0の試験粒子」として、3恒星からの加速度だけを受ける。
pub(crate) fn update_planet(planet: &mut Planet, stars: &[Star], dt: f64) {
    let mut acceleration = Vector3::zeros();
    for star in stars {
        let offset = star.position - planet.position;
        // 極端な接近時にも数値が発散しないよう、星の半径程度の軟化を入れる。
        let softened_distance_squared = offset.norm_squared() + star.radius.powi(2);
        acceleration += offset * G * star.mass / softened_distance_squared.powf(1.5);
    }
    planet.velocity += acceleration * dt;
    planet.position += planet.velocity * dt;
}
