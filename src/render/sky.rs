use crate::model::*;
use crate::observer::*;
use macroquad::prelude::*;
pub(crate) fn draw_stellar_appearance(
    x: f32,
    y: f32,
    radius: f32,
    gas: f32,
    color: Color,
    time: f64,
) {
    // 球の見た目は元の円盤と淡い光芒。追加の気体層は描かない。
    for layer in (1..=6).rev() {
        draw_circle(
            x,
            y,
            radius * (1.0 + layer as f32 * 0.24),
            Color::new(color.r, color.g, color.b, gas * 0.025),
        );
    }
    draw_circle(x, y, radius, Color::new(color.r, color.g, color.b, gas));
    draw_circle(x, y, radius * 0.72, Color::new(1.0, 0.96, 0.84, gas * 0.9));

    // 遠方では球面を描かず、小さな点光源と細い光条で「飛星」を表現する。
    // 光条は気体層ではなく見え方の演出。位置の運動は元の軌道計算を使う。
    let point = 1.0 - gas;
    let panel_scale =
        (screen_width() * 0.55 / 800.0).min((screen_height() - 84.0).max(1.0) / 700.0);
    let point_radius = flying_star_radius(panel_scale);
    let shimmer = 1.0 + 0.12 * (time as f32 * 5.0 + color.g * 7.0).sin();
    let length = point_radius * 2.5 * shimmer;
    draw_circle(
        x,
        y,
        point_radius * 2.0,
        Color::new(color.r, color.g, color.b, point * 0.12),
    );
    draw_line(
        x - length,
        y,
        x + length,
        y,
        1.0,
        Color::new(color.r, color.g, color.b, point * 0.8),
    );
    draw_line(
        x,
        y - length,
        x,
        y + length,
        1.0,
        Color::new(color.r, color.g, color.b, point * 0.8),
    );
    // 中心の大きさと不透明度はまたたきで変えず、視認性を維持する。
    draw_circle(x, y, point_radius, Color::new(1.0, 0.98, 0.9, point));
}

// 地上から周囲360度を見渡したパノラマ。地平線より下の恒星は地面で隠す。
// 軌道計算は2次元だが、観測地点の緯度と自転から空の高度を求める。
pub(crate) fn draw_ground_sky(planet: &Planet, stars: &[Star], time: f64) {
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
        .zip(stars)
        .map(|((_, altitude), star)| {
            // 飛星の点光源は視認用の目印だけ。空と地面を照らさない。
            let visible = gas_visibility((star.position - planet.position).norm(), star.radius);
            (altitude.sin() * 3.0 + 0.15).clamp(0.0, 1.0) * visible as f64
        })
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
        let gas = gas_visibility(distance, star.radius);
        if y - (radius * 2.6 * 1.8).max(8.0) > horizon {
            continue;
        }
        // パノラマの左右の継ぎ目でも太陽が途切れないよう複製する。
        for wrap in [-w, 0.0, w] {
            draw_stellar_appearance(x + wrap, y, radius, gas, star.color, time);
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
