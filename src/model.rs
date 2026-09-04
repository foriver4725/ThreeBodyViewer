use macroquad::prelude::*;
use nalgebra::Vector3;
pub(crate) struct Star {
    pub(crate) mass: f64,              // 質量
    pub(crate) position: Vector3<f64>, // 位置
    pub(crate) velocity: Vector3<f64>, // 速度
    pub(crate) radius: f64,            // 半径
    pub(crate) color: Color,           // 表示色
}

// 恒星の運動には影響を与えない、観測地点となる軽い惑星。
pub(crate) struct Planet {
    pub(crate) position: Vector3<f64>,
    pub(crate) velocity: Vector3<f64>,
}

pub(crate) struct SimulationPreset {
    pub(crate) name: &'static str,
    pub(crate) stars: Vec<Star>,
}
