use macroquad::prelude::*;

pub const SCREEN_WIDTH: f32 = 256.0;
pub const SCREEN_HEIGHT: f32 = 144.0;

pub const MAX_VELOCITY: f32 = 0.7 * 60.0;
pub const GROUND_FRICTION: f32 = 0.21 * 60.0;
pub const AIR_DRAG: f32 = 0.07 * 60.0;
pub const GRAVITY: f32 = 0.25 * 3600.0;

pub const PUMPKIN_PICKUP_DIST: f32 = 8.0;

pub fn create_camera(w: f32, h: f32) -> Camera2D {
    let rt = render_target(w as u32, h as u32);
    rt.texture.set_filter(FilterMode::Nearest);

    Camera2D {
        render_target: Some(rt),
        zoom: Vec2::new(1.0 / w * 2.0, 1.0 / h * 2.0),
        ..Default::default()
    }
}
