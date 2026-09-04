use crate::lagrange::*;
use crate::model::*;
use macroquad::prelude::*;
use nalgebra::Vector3;
pub(crate) fn draw_stars(stars: &[Star]) {
    for star in stars {
        draw_circle(
            star.position.x as f32,
            star.position.y as f32,
            star.radius as f32,
            star.color,
        );
    }
}

pub(crate) fn draw_planet(planet: &Planet) {
    let x = planet.position.x as f32;
    let y = planet.position.y as f32;
    draw_circle(x, y, 7.0, BLUE);
    draw_circle_lines(x, y, 11.0, 2.0, WHITE);
    draw_text("Planet", x + 14.0, y - 8.0, 16.0, WHITE);
}

// 北緯25度の観測者を仮定。自転軸は軌道面に垂直で、1日は60シミュレーション秒。
pub(crate) fn draw_lagrange_points(points: &[LagrangePoint], primary: &Star, secondary: &Star) {
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
pub(crate) fn draw_observer_overview(planet: &Planet, stars: &[Star], time: f64) {
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
