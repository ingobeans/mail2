use macroquad::{miniquad::window::screen_size, prelude::*};

use assets::*;
use player::*;
use utils::*;

use crate::utils::create_camera;

mod assets;
mod player;
mod utils;

struct MailEngine<'a> {
    assets: &'a Assets,
    player: Player,
    world: World,
    pixel_camera: Camera2D,
    world_camera: Camera2D,
}

impl<'a> MailEngine<'a> {
    fn new(assets: &'a Assets) -> Self {
        let world = World::default();

        let world_width = ((world.x_max - world.x_min) * 8) as f32 + 16.0 * 8.0;
        let world_height = ((world.y_max - world.y_min) * 8) as f32 + 16.0 * 8.0;

        let mut world_camera = create_camera(world_width, world_height);
        world_camera.target = vec2(
            world_width / 2.0 - 16.0 * 8.0,
            world_height / 2. - 16.0 * 8.0,
        );
        set_camera(&world_camera);
        clear_background(BLACK.with_alpha(0.0));
        clear_background(PINK);

        for chunk in &world.background {
            chunk.draw(&assets);
        }
        for chunk in &world.collision {
            chunk.draw(&assets);
        }
        for chunk in &world.details {
            chunk.draw(&assets);
        }
        for chunk in &world.one_way_collision {
            chunk.draw(&assets);
        }

        let mut player = Player::new();
        player.pos = vec2(0.0, -8.0);
        player.camera_pos = vec2(0.0, -100.0);

        let pixel_camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);
        MailEngine {
            assets: &assets,
            world,
            player,
            pixel_camera,
            world_camera,
        }
    }
    fn update(&mut self) {
        let delta_time = get_frame_time();
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);

        self.player.update(&self.world, delta_time);
        self.pixel_camera.target = self.player.camera_pos.floor();
        set_camera(&self.pixel_camera);
        clear_background(Color::from_hex(0xbcbc9d));

        //let world_width = ((self.world.x_max - self.world.x_min) * 8) as f32 + 16.0 * 8.0;
        //let world_height = ((self.world.y_max - self.world.y_min) * 8) as f32 + 16.0 * 8.0;
        draw_texture_ex(
            &self.world_camera.render_target.as_ref().unwrap().texture,
            //-(world_width / 2.0 - 16.0 * 8.0) / 2.0,
            //-(world_height / 2. - 16.0 * 8.0) / 2.0,
            (self.world.x_min - 16) as f32 * 8.0 / 2.0,
            (self.world.y_min - 16) as f32 * 8.0 / 2.0,
            WHITE,
            DrawTextureParams::default(),
        );
        self.player.draw(self.assets);

        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &self.pixel_camera.render_target.as_ref().unwrap().texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(
                    SCREEN_WIDTH * scale_factor,
                    SCREEN_HEIGHT * scale_factor,
                )),
                ..Default::default()
            },
        );
    }
}

#[macroquad::main("mail2")]
async fn main() {
    let assets = Assets::default();
    let mut mail_engine = MailEngine::new(&assets);

    loop {
        mail_engine.update();
        next_frame().await
    }
}
