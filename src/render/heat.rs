use crate::model::*;
use crate::observer::*;
use macroquad::prelude::*;
pub(crate) fn draw_heat_panel(planet: &Planet, stars: &[Star], time: f64) {
    let phase = -std::f64::consts::FRAC_PI_2 + time * std::f64::consts::TAU / 60.0;
    let incoming: Vec<f64> = stars
        .iter()
        .map(|star| relative_heat(star, planet))
        .collect();
    // 地上の水平面が受ける直射成分。地平線以下は0、天頂では最大。
    // 大気・反射・蓄熱・食は含めないので、地表温度そのものではない。
    let surface: Vec<f64> = stars
        .iter()
        .zip(&incoming)
        .map(|(star, heat)| {
            let (_, altitude) = horizontal_coordinates(star.position - planet.position, phase);
            heat * altitude.sin().max(0.0)
        })
        .collect();
    let total: f64 = incoming.iter().sum();
    let ground: f64 = surface.iter().sum();
    draw_rectangle(
        12.0,
        565.0,
        776.0,
        128.0,
        Color::new(0.015, 0.02, 0.025, 0.92),
    );
    draw_text(
        &format!(
            "RELATIVE HEAT   Space: {:.2}x   Ground: {:.2}x",
            total, ground
        ),
        25.0,
        589.0,
        22.0,
        WHITE,
    );
    for (index, star) in stars.iter().enumerate() {
        let y = 607.0 + index as f32 * 21.0;
        draw_text(
            &format!("Star {}", index + 1),
            25.0,
            y + 11.0,
            18.0,
            star.color,
        );
        draw_rectangle(102.0, y, 230.0, 10.0, Color::new(0.16, 0.17, 0.19, 1.0));
        // ゲージの最大は5倍。範囲外でも右側の数値は丸めずに表示する。
        draw_rectangle(
            102.0,
            y,
            230.0 * (surface[index] / 5.0).clamp(0.0, 1.0) as f32,
            10.0,
            star.color,
        );
        draw_text(
            &format!(
                "Ground {:.2}x / Space {:.2}x",
                surface[index], incoming[index]
            ),
            350.0,
            y + 11.0,
            18.0,
            star.color,
        );
    }
    draw_text(
        "1x = reference star at distance 200 | bars: 0-5x | not temperature",
        25.0,
        683.0,
        16.0,
        GRAY,
    );
}
