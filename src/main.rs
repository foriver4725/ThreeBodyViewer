use macroquad::prelude::*;
use nalgebra::Vector3;

const G: f64 = 6.67430e-11; // 万有引力定数
const FIXED_DT: f64 = 1.0 / 600.0; // 早送りでも物理計算の刻み幅は固定
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

// 北緯25度の観測者を仮定。自転軸は軌道面に垂直で、1日は60シミュレーション秒。
// 地上に固定された東・北・天頂の基底へ恒星方向を変換する。
fn horizontal_coordinates(direction: Vector3<f64>, phase: f64) -> (f64, f64) {
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

// 地上から周囲360度を見渡したパノラマ。地平線より下の恒星は地面で隠す。
// 軌道計算は2次元だが、観測地点の緯度と自転から空の高度を求める。
fn draw_ground_sky(planet: &Planet, stars: &[Star], time: f64) {
    let w = 800.0;
    let h = 700.0;
    let horizon = h * 0.76;
    let phase = -std::f64::consts::FRAC_PI_2 + time * std::f64::consts::TAU / 60.0;
    let coordinates: Vec<_> = stars
        .iter()
        .map(|star| horizontal_coordinates(star.position - planet.position, phase))
        .collect();
    let daylight = coordinates
        .iter()
        .map(|(_, altitude)| (altitude.sin() * 3.0 + 0.15).clamp(0.0, 1.0))
        .fold(0.0_f64, f64::max) as f32;
    // 高度に応じた空のグラデーションと、地平線付近の霞。
    for row in 0..100 {
        let t = row as f32 / 100.0;
        draw_rectangle(
            0.0,
            horizon * t,
            w,
            horizon / 100.0 + 1.0,
            Color::new(
                0.015 + daylight * (0.12 + t * 0.26),
                0.02 + daylight * (0.25 + t * 0.23),
                0.06 + daylight * (0.40 + t * 0.12),
                1.0,
            ),
        );
    }
    for (star, (azimuth, altitude)) in stars.iter().zip(coordinates) {
        let distance = (star.position - planet.position).norm().max(1.0);
        let x = ((azimuth / std::f64::consts::TAU + 0.5) as f32) * w;
        let y = horizon - (altitude / std::f64::consts::FRAC_PI_2) as f32 * (horizon - 100.0);
        // 半径は演出用の恒星サイズ。角半径から画面サイズへ変換する。
        let radius = ((star.radius / distance).atan() as f32 * (horizon - 100.0)
            / std::f32::consts::FRAC_PI_2)
            .clamp(2.0, 120.0);
        if y - radius > horizon {
            continue;
        }
        // パノラマの左右の継ぎ目でも太陽が途切れないよう複製する。
        for wrap in [-w, 0.0, w] {
            for layer in (1..=6).rev() {
                draw_circle(
                    x + wrap,
                    y,
                    radius * (1.0 + layer as f32 * 0.24),
                    Color::new(star.color.r, star.color.g, star.color.b, 0.025),
                );
            }
            draw_circle(x + wrap, y, radius, star.color);
            draw_circle(x + wrap, y, radius * 0.72, Color::new(1.0, 0.96, 0.84, 0.9));
        }
    }
    // 地面を最後に描くことで、日没中の太陽も地平線で正しく隠れる。
    draw_rectangle(
        0.0,
        horizon,
        w,
        h - horizon,
        Color::new(
            0.04 + daylight * 0.13,
            0.035 + daylight * 0.09,
            0.03 + daylight * 0.055,
            1.0,
        ),
    );
    for i in 0..16 {
        let x = i as f32 * w / 15.0;
        draw_line(
            w * 0.5,
            horizon,
            x,
            h,
            1.0,
            Color::new(0.35, 0.27, 0.18, 0.18),
        );
    }
    for (fraction, label) in [(0.0, "S"), (0.25, "W"), (0.5, "N"), (0.75, "E"), (1.0, "S")] {
        draw_text(
            label,
            (fraction * w - 7.0).clamp(4.0, w - 18.0),
            horizon + 25.0,
            20.0,
            LIGHTGRAY,
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

// 恒星・惑星・ラグランジュ点を収める自動縮尺。惑星が遠くへ飛んでも追跡できる。
fn draw_observer_overview(planet: &Planet, stars: &[Star], time: f64) {
    let points = LagrangePointFactory::create(&stars[0], &stars[1]);
    let mut min = vec2(planet.position.x as f32, planet.position.y as f32);
    let mut max = min;
    for position in stars
        .iter()
        .map(|s| s.position)
        .chain(points.iter().map(|p| p.position))
    {
        let p = vec2(position.x as f32, position.y as f32);
        min = min.min(p);
        max = max.max(p);
    }
    let span = max - min;
    let scale = (660.0 / span.x.max(1.0))
        .min(480.0 / span.y.max(1.0))
        .min(1.5);
    let center = (min + max) * 0.5;
    let project = |p: Vector3<f64>| {
        let xy = (vec2(p.x as f32, p.y as f32) - center) * scale + vec2(400.0, 340.0);
        Vector3::new(xy.x as f64, xy.y as f64, 0.0)
    };
    let projected: Vec<_> = stars
        .iter()
        .map(|star| Star {
            position: project(star.position),
            mass: star.mass,
            velocity: star.velocity,
            radius: star.radius,
            color: star.color,
        })
        .collect();
    let projected_points: Vec<_> = points
        .iter()
        .map(|p| LagrangePoint {
            name: p.name,
            position: project(p.position),
        })
        .collect();
    draw_lagrange_points(&projected_points, &projected[0], &projected[1]);
    draw_stars(&projected);
    let p = project(planet.position);
    draw_planet(&Planet {
        position: p,
        velocity: planet.velocity,
    });
    let center = vec2(p.x as f32, p.y as f32);
    let phase = -std::f64::consts::FRAC_PI_2 + time * std::f64::consts::TAU / 60.0;
    let up = vec2(phase.cos() as f32, phase.sin() as f32);
    // 白点は地表の観測者（位置を拡大表示）。矢印は地上の天頂のXY投影。
    let observer = center + up * 12.0;
    let tip = center + up * 48.0;
    let side = vec2(-up.y, up.x);
    draw_circle(observer.x, observer.y, 4.0, WHITE);
    draw_line(observer.x, observer.y, tip.x, tip.y, 2.0, WHITE);
    draw_triangle(
        tip,
        tip - up * 10.0 + side * 5.0,
        tip - up * 10.0 - side * 5.0,
        WHITE,
    );
    draw_text("YOU", center.x + 18.0, center.y + 25.0, 22.0, WHITE);
    draw_text("ORBIT / observer location", 20.0, 35.0, 24.0, WHITE);
    draw_text(
        "Blue: planet  White dot: observer (enlarged)",
        20.0,
        650.0,
        20.0,
        LIGHTGRAY,
    );
    draw_text(
        "Arrow: local zenith projected onto orbit plane",
        20.0,
        678.0,
        20.0,
        LIGHTGRAY,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ground_projection_distinguishes_zenith_and_below_horizon() {
        let latitude = 25.0_f64.to_radians();
        let up = Vector3::new(latitude.cos(), 0.0, latitude.sin());
        let (_, altitude) = horizontal_coordinates(up, 0.0);
        assert!((altitude - std::f64::consts::FRAC_PI_2).abs() < 1.0e-7);
        assert!(horizontal_coordinates(-up, 0.0).1 < 0.0);
    }

    #[test]
    fn rotation_changes_sun_altitude() {
        let direction = Vector3::new(1.0, 0.0, 0.0);
        assert!(horizontal_coordinates(direction, 0.0).1 > 0.0);
        assert!(horizontal_coordinates(direction, std::f64::consts::PI).1 < 0.0);
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
    let mut simulation_time = 0.0;
    let mut accumulator = 0.0;
    let mut boost: f64 = 8.0;
    // 独立した描画先に描いてから並べるため、パネル間ではみ出さない。
    let orbit_target = render_target(800, 700);
    let ground_target = render_target(800, 700);
    let panel_camera = |target: RenderTarget| Camera2D {
        render_target: Some(target),
        ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, 800.0, 700.0))
    };

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
                simulation_time = 0.0;
                accumulator = 0.0;
            }
        }

        if is_key_pressed(KeyCode::Up) {
            boost = (boost * 2.0).min(32.0);
        }
        if is_key_pressed(KeyCode::Down) {
            boost = (boost / 2.0).max(2.0);
        }
        let speed = if is_key_down(KeyCode::Space) {
            boost
        } else {
            1.0
        };
        accumulator += get_frame_time().min(0.1) as f64 * speed;
        let sub_dt = FIXED_DT;

        clear_background(BLACK);

        {
            // シミュレート
            while accumulator >= FIXED_DT {
                accumulator -= FIXED_DT;
                simulation_time += FIXED_DT;
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
        }

        set_camera(&panel_camera(orbit_target.clone()));
        clear_background(Color::new(0.015, 0.02, 0.04, 1.0));
        draw_observer_overview(&planet, &preset.stars, simulation_time);
        set_camera(&panel_camera(ground_target.clone()));
        clear_background(BLACK);
        draw_ground_sky(&planet, &preset.stars, simulation_time);
        draw_text("GROUND / 360-degree panorama", 20.0, 35.0, 24.0, WHITE);
        set_default_camera();
        let split = screen_width() * 0.45;
        let height = (screen_height() - 84.0).max(1.0);
        for (target, x, width) in [
            (&orbit_target, 0.0, split),
            (&ground_target, split, screen_width() - split),
        ] {
            draw_texture_ex(
                &target.texture,
                x,
                84.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(width, height)),
                    flip_y: true,
                    ..Default::default()
                },
            );
        }
        draw_line(split, 84.0, split, screen_height(), 2.0, GRAY);
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            84.0,
            Color::new(0.0, 0.0, 0.0, 0.6),
        );
        draw_text(
            &format!(
                "SPACE: hold to speed up | UP/DOWN: boost {:.0}x | {:.0}x  t={:.1}s",
                boost, speed, simulation_time
            ),
            10.0,
            77.0,
            19.0,
            WHITE,
        );

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
