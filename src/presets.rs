use crate::lagrange::LagrangePointFactory;
use crate::model::*;
use crate::physics::G;
use macroquad::prelude::*;
use nalgebra::Vector3;
pub(crate) const STAR_MASS: f64 = 1.0e16;
pub(crate) const ORBIT_RADIUS: f64 = 150.0;
pub(crate) struct PlanetFactory;

impl PlanetFactory {
    /// 各プリセットの重心付近を通過する、共通の観測惑星を生成する。
    pub(crate) fn create(stars: &[Star]) -> Planet {
        let total_mass: f64 = stars.iter().map(|star| star.mass).sum();
        let center = stars
            .iter()
            .map(|star| star.position * star.mass)
            .sum::<Vector3<f64>>()
            / total_mass;
        let center_velocity = stars
            .iter()
            .map(|star| star.velocity * star.mass)
            .sum::<Vector3<f64>>()
            / total_mass;

        Planet {
            position: center + Vector3::new(0.0, -220.0, 0.0),
            velocity: center_velocity + Vector3::new(55.0, 0.0, 0.0),
        }
    }
}

pub(crate) struct SimulationPresetFactory;

impl SimulationPresetFactory {
    pub(crate) const COUNT: usize = 10;

    /// 0〜9の番号から、異なる動きをする3天体の初期値を生成する。
    /// 範囲外の番号は0番（正三角形軌道）として扱う。
    pub(crate) fn create(index: usize) -> SimulationPreset {
        let center = Vector3::new(400.0, 300.0, 0.0);
        let star = |mass, x, y, vx, vy, color| Star {
            mass,
            position: center + Vector3::new(x, y, 0.0),
            velocity: Vector3::new(vx, vy, 0.0),
            radius: 16.0,
            color,
        };

        match index {
            // 3つの等質量星が正三角形を保ちながら回転する初期値。
            0 => {
                let speed = (G * STAR_MASS / (3.0_f64.sqrt() * ORBIT_RADIUS)).sqrt();
                SimulationPreset {
                    name: "Triangle orbit",
                    stars: vec![
                        star(STAR_MASS, ORBIT_RADIUS, 0.0, 0.0, speed, YELLOW),
                        star(
                            STAR_MASS,
                            -ORBIT_RADIUS / 2.0,
                            ORBIT_RADIUS * 3.0_f64.sqrt() / 2.0,
                            -speed * 3.0_f64.sqrt() / 2.0,
                            -speed / 2.0,
                            SKYBLUE,
                        ),
                        star(
                            STAR_MASS,
                            -ORBIT_RADIUS / 2.0,
                            -ORBIT_RADIUS * 3.0_f64.sqrt() / 2.0,
                            speed * 3.0_f64.sqrt() / 2.0,
                            -speed / 2.0,
                            PINK,
                        ),
                    ],
                }
            }
            // 有名な三体問題の周期解。3つの星が同じ8の字軌道を追いかける。
            1 => {
                let position_scale = 120.0;
                let velocity_scale = (G * STAR_MASS / position_scale).sqrt();
                SimulationPreset {
                    name: "Figure eight",
                    stars: vec![
                        star(
                            STAR_MASS,
                            0.97000436 * position_scale,
                            -0.24308753 * position_scale,
                            0.466203685 * velocity_scale,
                            0.43236573 * velocity_scale,
                            YELLOW,
                        ),
                        star(
                            STAR_MASS,
                            -0.97000436 * position_scale,
                            0.24308753 * position_scale,
                            0.466203685 * velocity_scale,
                            0.43236573 * velocity_scale,
                            SKYBLUE,
                        ),
                        star(
                            STAR_MASS,
                            0.0,
                            0.0,
                            -0.93240737 * velocity_scale,
                            -0.86473146 * velocity_scale,
                            PINK,
                        ),
                    ],
                }
            }
            // 速度0で一直線に並べ、重力だけで互いに落下させる単純な例。
            2 => SimulationPreset {
                name: "Collinear fall",
                stars: vec![
                    star(STAR_MASS, -160.0, 0.0, 0.0, 0.0, YELLOW),
                    star(STAR_MASS, 0.0, 0.0, 0.0, 0.0, SKYBLUE),
                    star(STAR_MASS, 160.0, 0.0, 0.0, 0.0, PINK),
                ],
            },
            // 2つの重い星の近くを、軽い3つ目の星が横切る初期値。
            3 => SimulationPreset {
                name: "Binary and visitor",
                stars: vec![
                    star(1.5e16, -90.0, 0.0, 0.0, -45.0, YELLOW),
                    star(1.5e16, 90.0, 0.0, 0.0, 45.0, SKYBLUE),
                    star(2.0e15, -260.0, -130.0, 105.0, 45.0, PINK),
                ],
            },
            // 質量も速度も揃えず、予測しにくい動きを観察するための例。
            4 => SimulationPreset {
                name: "Asymmetric dance",
                stars: vec![
                    star(1.8e16, -130.0, -70.0, 25.0, -30.0, YELLOW),
                    star(8.0e15, 120.0, -40.0, 20.0, 65.0, SKYBLUE),
                    star(1.2e16, 10.0, 150.0, -55.0, -20.0, PINK),
                ],
            },
            // 6〜10番は、軽い第3天体を各ラグランジュ点へ配置する観察用セット。
            5 => Self::create_lagrange_probe(0, "Probe at L1"),
            6 => Self::create_lagrange_probe(1, "Probe at L2"),
            7 => Self::create_lagrange_probe(2, "Probe at L3"),
            8 => Self::create_lagrange_probe(3, "Probe at L4"),
            9 => Self::create_lagrange_probe(4, "Probe at L5"),
            _ => Self::create(0),
        }
    }

    /// 質量の違う2つの主星を円運動させ、指定したラグランジュ点へ
    /// 重力への影響がほぼない軽い観測星を置く。
    pub(crate) fn create_lagrange_probe(
        point_index: usize,
        name: &'static str,
    ) -> SimulationPreset {
        let center = Vector3::new(400.0, 300.0, 0.0);
        let primary_mass = 2.0e16;
        let secondary_mass = 1.0e16;
        let distance: f64 = 220.0;
        let total_mass = primary_mass + secondary_mass;
        let angular_speed = (G * total_mass / distance.powi(3)).sqrt();

        // 重心が画面中央に来るよう、質量比に応じて主星を左右へ配置する。
        let primary_position =
            center + Vector3::new(-distance * secondary_mass / total_mass, 0.0, 0.0);
        let secondary_position =
            center + Vector3::new(distance * primary_mass / total_mass, 0.0, 0.0);
        let circular_velocity = |position: Vector3<f64>| {
            let offset = position - center;
            Vector3::new(-angular_speed * offset.y, angular_speed * offset.x, 0.0)
        };

        let primary = Star {
            mass: primary_mass,
            position: primary_position,
            velocity: circular_velocity(primary_position),
            radius: 18.0,
            color: YELLOW,
        };
        let secondary = Star {
            mass: secondary_mass,
            position: secondary_position,
            velocity: circular_velocity(secondary_position),
            radius: 14.0,
            color: SKYBLUE,
        };

        let probe_position =
            LagrangePointFactory::create(&primary, &secondary)[point_index].position;
        let probe = Star {
            // 円制限三体問題へ近づけるため、観測星の質量は主星より十分小さくする。
            mass: 1.0e8,
            position: probe_position,
            velocity: circular_velocity(probe_position),
            radius: 8.0,
            color: PINK,
        };

        SimulationPreset {
            name,
            stars: vec![primary, secondary, probe],
        }
    }
}
