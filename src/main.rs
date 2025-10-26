use macroquad::{miniquad::window::screen_size, prelude::*};

use assets::*;
use player::*;
use utils::*;

use crate::utils::create_camera;

mod assets;
mod physics;
mod player;
mod utils;

struct PumpkinEngine<'a> {
    assets: &'a Assets,
    player: Player,
    world: World,
    pixel_camera: Camera2D,
    frame: u32,
    /// Camera used to render the world.
    ///
    /// World is only rendered once. It is rendered to a texture that can then be drawn every frame.
    world_camera: Camera2D,
    poi_location: Vec2,
    pumpkin_guy_animation: Option<f32>,
}

impl<'a> PumpkinEngine<'a> {
    fn new(assets: &'a Assets) -> Self {
        let world = World::default();

        let world_width = ((world.x_max - world.x_min) * 8) as f32 + 16.0 * 8.0;
        let world_height = ((world.y_max - world.y_min) * 8) as f32 + 16.0 * 8.0;

        // render world
        let mut world_camera = create_camera(world_width, world_height);
        world_camera.target = vec2(
            (world.x_min + world.x_max + 16) as f32 / 2.0 * 8.0,
            (world.y_min + world.y_max + 16) as f32 / 2.0 * 8.0,
        );
        set_camera(&world_camera);
        clear_background(BLACK.with_alpha(0.0));

        for chunk in &world.background {
            chunk.draw(assets);
        }
        for chunk in &world.collision {
            chunk.draw(assets);
        }
        for chunk in &world.details {
            chunk.draw(assets);
        }
        for chunk in &world.one_way_collision {
            chunk.draw(assets);
        }

        let poi_location = world.get_interactable_spawn(160).unwrap();

        let mut player = Player::new();
        player.pos = vec2(0.0, -8.0);
        player.camera_pos = vec2(0.0, -100.0);
        player.poi_location = poi_location;

        let pixel_camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);

        PumpkinEngine {
            poi_location,
            frame: 0,
            assets,
            world,
            player,
            pixel_camera,
            world_camera,
            pumpkin_guy_animation: None,
        }
    }
    fn update(&mut self) {
        self.frame += 1;

        // cap delta time to a minimum of 60 fps.
        let delta_time = get_frame_time().min(1.0 / 60.0);
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        self.player.update(&mut self.world, delta_time);
        self.pixel_camera.target = self.player.camera_pos.floor();
        set_camera(&self.pixel_camera);
        clear_background(Color::from_hex(0x567c7d));

        // position world texture
        draw_texture_ex(
            &self.world_camera.render_target.as_ref().unwrap().texture,
            (self.world.x_min) as f32 * 8.0,
            (self.world.y_min) as f32 * 8.0,
            WHITE,
            DrawTextureParams::default(),
        );
        draw_texture(
            self.assets.poi.get_at_time((get_time() * 1000.0) as u32),
            self.poi_location.x,
            self.poi_location.y - 3.0 * 8.0,
            WHITE,
        );
        if let Some(pumpkin_guy_animation) = &mut self.pumpkin_guy_animation {
            draw_texture(
                self.assets
                    .pumpkin_guy_animation
                    .get_at_time((*pumpkin_guy_animation * 1000.0) as u32),
                self.poi_location.x,
                self.poi_location.y - 24.0,
                WHITE,
            );
            *pumpkin_guy_animation += delta_time;
            if *pumpkin_guy_animation * 1000.0
                > self.assets.pumpkin_guy_animation.total_length as f32
            {
                panic!("wa");
            }
        }
        for pumpkin in self.world.pumpkins.iter_mut() {
            pumpkin.draw(self.assets, &self.player.pos, self.player.on_ground);
            pumpkin.update(
                delta_time,
                &self.world.collision,
                &self.world.one_way_collision,
            );
        }
        self.player.draw(self.assets);
        if self.pumpkin_guy_animation.is_none()
            && self.player.pos.distance(self.poi_location) <= 16.0
        {
            if self.player.carrying.is_none() {
                let text = "bring me a pumpkin\nand i will bake you a pie!";
                draw_rectangle(
                    self.poi_location.x - 6.0 * 8.0,
                    self.poi_location.y - 3.0 * 8.0 + 2.0,
                    ({
                        let mut s = text.lines().collect::<Vec<&str>>();
                        s.sort_by(|a, b| b.len().cmp(&a.len()));
                        s
                    }[0]
                    .len()
                        + 2) as f32
                        * 4.0,
                    (text.lines().count() + 2) as f32 * 5.0,
                    WHITE,
                );
                self.assets.draw_text(
                    text,
                    self.poi_location.x + 4.0 - 6.0 * 8.0,
                    self.poi_location.y + 5.0 - 3.0 * 8.0 + 2.0,
                );
            } else {
                if show_tooltip("e: give pumpkin", self.assets, &self.player) {
                    self.pumpkin_guy_animation = Some(0.0);
                    self.player.carrying = None;
                }
            }
        }

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
        //draw_text(&get_fps().to_string(), 48.0, 48.0, 32.0, WHITE);
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "mail2".to_string(),
        window_width: SCREEN_WIDTH as i32 * 3,
        window_height: SCREEN_HEIGHT as i32 * 3,
        //platform: macroquad::miniquad::conf::Platform {
        //    swap_interval: Some(0),
        //    ..Default::default()
        //},
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() {
    let assets = Assets::default();
    let mut mail_engine = PumpkinEngine::new(&assets);

    loop {
        mail_engine.update();
        next_frame().await
    }
}
