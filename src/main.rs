mod lagrange;
mod model;
mod observer;
mod physics;
mod presets;
mod render;
#[cfg(test)]
mod tests;

use macroquad::prelude::*;
use physics::*;
use presets::{PlanetFactory, SimulationPresetFactory};
use render::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "ThreeBodyViewer".to_owned(),
        window_width: 1600,
        window_height: 900,
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
                step(&mut preset.stars, &mut planet, sub_dt);
            }
        }

        set_camera(&panel_camera(orbit_target.clone()));
        clear_background(Color::new(0.015, 0.02, 0.04, 1.0));
        draw_observer_overview(&planet, &preset.stars, simulation_time);
        set_camera(&panel_camera(ground_target.clone()));
        clear_background(BLACK);
        draw_ground_sky(&planet, &preset.stars, simulation_time);
        draw_heat_panel(&planet, &preset.stars, simulation_time);
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
