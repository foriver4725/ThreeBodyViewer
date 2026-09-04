use macroquad::prelude::*;
use nalgebra::Vector3;
use std::collections::VecDeque;

const G: f64 = 6.67430e-11; // 万有引力定数
const STEP_PER_FRAME: i32 = 100; // 1フレームあたりのシミュレーションステップ数
const STAR_MASS: f64 = 1.0e16;
const ORBIT_RADIUS: f64 = 150.0;

// 恒星
struct Star {
    mass: f64,              // 質量
    position: Vector3<f64>, // 位置
    velocity: Vector3<f64>, // 速度
    radius: f64,            // 半径
    color: Color,           // 表示色
}

// 恒星の運動には影響を与えない、観測地点となる軽い惑星。
struct Planet {
    position: Vector3<f64>,
    velocity: Vector3<f64>,
}

struct PlanetFactory;

impl PlanetFactory {
    /// 各プリセットの重心付近を通過する、共通の観測惑星を生成する。
    fn create(stars: &[Star]) -> Planet {
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

// 実験しやすいよう、3つの星の質量・位置・速度をひとまとめにした初期値セット。
struct SimulationPreset {
    name: &'static str,
    stars: Vec<Star>,
}

struct SimulationPresetFactory;

impl SimulationPresetFactory {
    const COUNT: usize = 10;

    /// 0〜9の番号から、異なる動きをする3天体の初期値を生成する。
    /// 範囲外の番号は0番（正三角形軌道）として扱う。
    fn create(index: usize) -> SimulationPreset {
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
    fn create_lagrange_probe(point_index: usize, name: &'static str) -> SimulationPreset {
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

// ラグランジュ点は「天体」ではなく、2つの主星と一緒に回る座標系で
// 重力と遠心力が釣り合う場所。通常は L1〜L5 の5点が存在する。
struct LagrangePoint {
    name: &'static str,
    position: Vector3<f64>,
}

struct LagrangePointFactory;

impl LagrangePointFactory {
    /// 2つの主星から5つのラグランジュ点を生成する。
    ///
    /// 円制限三体問題（主星同士は円運動し、置く物体の質量は無視できる）を
    /// 仮定している。L1〜L3は数値計算、L4・L5は正三角形から求める。
    fn create(primary: &Star, secondary: &Star) -> [LagrangePoint; 5] {
        let relative = secondary.position - primary.position;
        let distance = relative.norm();
        assert!(distance > f64::EPSILON, "主星同士の位置が重なっています");
        assert!(
            primary.mass > 0.0 && secondary.mass > 0.0,
            "主星の質量は正の値にしてください"
        );

        let x_axis = relative / distance;
        // 現在はXY平面上の運動を扱うため、x軸を90度回した方向をy軸にする。
        let y_axis = Vector3::new(-x_axis.y, x_axis.x, 0.0);
        let total_mass = primary.mass + secondary.mass;
        let mu = secondary.mass / total_mass;
        let center =
            (primary.position * primary.mass + secondary.position * secondary.mass) / total_mass;

        // 重心を原点、主星間距離を1とした回転座標系でL1〜L3を解く。
        let l1_x = Self::solve_collinear(mu, -mu, 1.0 - mu);
        let l2_x = Self::solve_collinear(mu, 1.0 - mu, 2.0);
        let l3_x = Self::solve_collinear(mu, -2.0, -mu);

        let point = |name, x: f64, y: f64| LagrangePoint {
            name,
            position: center + (x_axis * x + y_axis * y) * distance,
        };

        [
            point("L1", l1_x, 0.0),
            point("L2", l2_x, 0.0),
            point("L3", l3_x, 0.0),
            point("L4", 0.5 - mu, 3.0_f64.sqrt() / 2.0),
            point("L5", 0.5 - mu, -3.0_f64.sqrt() / 2.0),
        ]
    }

    // x方向の有効力が0になる場所を二分法で探す。
    fn solve_collinear(mu: f64, left: f64, right: f64) -> f64 {
        // 主星そのもの（区間端）では式が発散するため、わずかに内側から探す。
        let epsilon = 1.0e-9;
        let mut low = left + epsilon;
        let mut high = right - epsilon;

        for _ in 0..100 {
            let middle = (low + high) / 2.0;
            if Self::effective_force(mu, low).signum() == Self::effective_force(mu, middle).signum()
            {
                low = middle;
            } else {
                high = middle;
            }
        }

        (low + high) / 2.0
    }

    // 正規化した回転座標系における、x方向の重力と遠心力の合計。
    fn effective_force(mu: f64, x: f64) -> f64 {
        let primary_offset = x + mu;
        let secondary_offset = x - (1.0 - mu);
        x - (1.0 - mu) * primary_offset / primary_offset.abs().powi(3)
            - mu * secondary_offset / secondary_offset.abs().powi(3)
    }
}

// 2つの恒星間に働く万有引力の大きさを計算する
fn calculate_gravity_norm(star1: &Star, star2: &Star) -> f64 {
    let distance_squared = (star2.position - star1.position).norm_squared();
    G * star1.mass * star2.mass / distance_squared
}

// 惑星は恒星を動かさない「質量0の試験粒子」として、3恒星からの加速度だけを受ける。
fn update_planet(planet: &mut Planet, stars: &[Star], dt: f64) {
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

fn draw_stars(stars: &[Star]) {
    for star in stars {
        draw_circle(
            star.position.x as f32,
            star.position.y as f32,
            star.radius as f32,
            star.color,
        );
    }
}

fn draw_planet(planet: &Planet) {
    let x = planet.position.x as f32;
    let y = planet.position.y as f32;
    draw_circle(x, y, 7.0, BLUE);
    draw_circle_lines(x, y, 11.0, 2.0, WHITE);
    draw_text("Planet", x + 14.0, y - 8.0, 16.0, WHITE);
}

// 惑星から見た各恒星の方向を、全天図の円周上の座標へ変換する。
fn sky_positions(planet: &Planet, stars: &[Star], center: Vec2, radius: f32) -> [Vec2; 3] {
    std::array::from_fn(|index| {
        let direction = stars[index].position - planet.position;
        let direction_2d = vec2(direction.x as f32, direction.y as f32).normalize_or_zero();
        center + direction_2d * radius
    })
}

fn draw_planet_sky(
    planet: &Planet,
    stars: &[Star],
    history: &VecDeque<[Vec2; 3]>,
    center: Vec2,
    radius: f32,
) {
    // 右側を惑星の空として塗り直し、俯瞰図と明確に分ける。
    draw_rectangle(
        800.0,
        0.0,
        400.0,
        700.0,
        Color::new(0.015, 0.025, 0.08, 1.0),
    );
    draw_line(800.0, 0.0, 800.0, 700.0, 2.0, DARKGRAY);
    draw_text("Sky from the planet", 830.0, 42.0, 28.0, WHITE);
    draw_text("N", center.x - 6.0, center.y - radius - 15.0, 18.0, GRAY);
    draw_text("E", center.x + radius + 8.0, center.y + 5.0, 18.0, GRAY);
    draw_text("S", center.x - 6.0, center.y + radius + 25.0, 18.0, GRAY);
    draw_text("W", center.x - radius - 25.0, center.y + 5.0, 18.0, GRAY);
    draw_circle_lines(center.x, center.y, radius, 2.0, GRAY);
    draw_circle(center.x, center.y, 6.0, BLUE);

    // 過去の見かけ方向を薄い線で残し、惑星の空での移動を読めるようにする。
    for star_index in 0..3 {
        let mut samples = history.iter();
        let Some(mut previous) = samples.next() else {
            continue;
        };
        for (age, current) in samples.enumerate() {
            let alpha = 0.08 + 0.35 * age as f32 / history.len().max(1) as f32;
            let color = Color::new(
                stars[star_index].color.r,
                stars[star_index].color.g,
                stars[star_index].color.b,
                alpha,
            );
            draw_line(
                previous[star_index].x,
                previous[star_index].y,
                current[star_index].x,
                current[star_index].y,
                2.0,
                color,
            );
            previous = current;
        }
    }

    let positions = sky_positions(planet, stars, center, radius);
    for (index, (star, position)) in stars.iter().zip(positions).enumerate() {
        let distance = (star.position - planet.position).norm() as f32;
        // 近い恒星ほど大きく見せる。実半径ではなく視認性を優先した表現。
        let apparent_radius = (1800.0 / distance.max(1.0)).clamp(6.0, 28.0);
        draw_circle(position.x, position.y, apparent_radius, star.color);
        draw_text(
            &format!("Star {}  d={:.0}", index + 1, distance),
            830.0,
            560.0 + index as f32 * 28.0,
            20.0,
            star.color,
        );
    }
}

fn draw_lagrange_points(points: &[LagrangePoint], primary: &Star, secondary: &Star) {
    let guide_color = Color::new(0.2, 0.8, 0.3, 0.25);
    let draw_guide = |from: Vector3<f64>, to: Vector3<f64>| {
        draw_line(
            from.x as f32,
            from.y as f32,
            to.x as f32,
            to.y as f32,
            1.0,
            guide_color,
        );
    };

    // 主星間の軸と、L4・L5が作る正三角形を薄い補助線で示す。
    draw_guide(primary.position, secondary.position);
    for point in &points[3..=4] {
        draw_guide(primary.position, point.position);
        draw_guide(secondary.position, point.position);
    }

    for point in points {
        let x = point.position.x as f32;
        let y = point.position.y as f32;
        draw_circle_lines(x, y, 7.0, 2.0, GREEN);
        draw_text(point.name, x + 10.0, y - 8.0, 18.0, GREEN);
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "ThreeBodyViewer".to_owned(),
        window_width: 1200,
        window_height: 700,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut preset_index = 0;
    let mut preset = SimulationPresetFactory::create(preset_index);
    let mut planet = PlanetFactory::create(&preset.stars);
    let mut sky_history: VecDeque<[Vec2; 3]> = VecDeque::new();
    let mut history_timer = 0.0;

    loop {
        // 数字キー1〜9と0（10番）で、その場で別の初期値へリセットできる。
        let preset_keys = [
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
            KeyCode::Key0,
        ];
        for (index, key) in preset_keys.iter().enumerate() {
            if is_key_pressed(*key) {
                preset_index = index;
                preset = SimulationPresetFactory::create(preset_index);
                planet = PlanetFactory::create(&preset.stars);
                sky_history.clear();
                history_timer = 0.0;
            }
        }

        let dt = get_frame_time();
        let sub_dt = dt as f64 / STEP_PER_FRAME as f64;

        clear_background(BLACK);

        {
            // シミュレート
            for _ in 0..STEP_PER_FRAME {
                let mut forces = vec![Vector3::new(0.0, 0.0, 0.0); preset.stars.len()];

                // 万有引力の計算
                for i in 0..preset.stars.len() {
                    for j in 0..preset.stars.len() {
                        if i != j {
                            let gravity_norm =
                                calculate_gravity_norm(&preset.stars[i], &preset.stars[j]);
                            let direction =
                                (preset.stars[j].position - preset.stars[i].position).normalize();
                            forces[i] += direction * gravity_norm;
                        }
                    }
                }

                // 速度と位置の更新
                for (star, force) in preset.stars.iter_mut().zip(forces) {
                    let acceleration = force / star.mass;
                    star.velocity += acceleration * sub_dt;
                    star.position += star.velocity * sub_dt;
                }
                update_planet(&mut planet, &preset.stars, sub_dt);
            }

            // 描画
            draw_stars(&preset.stars);
            draw_planet(&planet);
            // 先頭2つの星を主星として、現在位置からL1〜L5を表示する。
            let lagrange_points = LagrangePointFactory::create(&preset.stars[0], &preset.stars[1]);
            draw_lagrange_points(&lagrange_points, &preset.stars[0], &preset.stars[1]);
        }

        let sky_center = vec2(1000.0, 310.0);
        let sky_radius = 145.0;
        history_timer += dt;
        if history_timer >= 0.08 {
            sky_history.push_back(sky_positions(
                &planet,
                &preset.stars,
                sky_center,
                sky_radius,
            ));
            if sky_history.len() > 120 {
                sky_history.pop_front();
            }
            history_timer = 0.0;
        }
        draw_planet_sky(&planet, &preset.stars, &sky_history, sky_center, sky_radius);

        draw_text(&format!("FPS: {}", get_fps()), 10.0, 30.0, 24.0, DARKGRAY);
        draw_text(
            &format!(
                "Preset {}/{}: {}  [keys 1-9, 0]",
                preset_index + 1,
                SimulationPresetFactory::COUNT,
                preset.name
            ),
            10.0,
            55.0,
            22.0,
            LIGHTGRAY,
        );

        next_frame().await
    }
}
