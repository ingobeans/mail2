use macroquad::prelude::*;

use crate::{assets::*, utils::*};

fn get_tile(chunks: &[&Chunk], x: i16, y: i16) -> i16 {
    let cx = ((x as f32 / 16.0).floor() * 16.0) as i16;
    let cy = ((y as f32 / 16.0).floor() * 16.0) as i16;
    let Some(chunk) = chunks.iter().find(|f| f.x == cx && f.y == cy) else {
        return 0;
    };
    let local_x = x - chunk.x;
    let local_y = y - chunk.y;
    chunk.tile_at(local_x as _, local_y as _).unwrap_or(0)
}

fn ceil_g(a: f32) -> f32 {
    if a < 0.0 { a.floor() } else { a.ceil() }
}

#[derive(PartialEq)]
pub enum Tag {}

pub struct Player {
    pub pos: Vec2,
    pub camera_pos: Vec2,
    pub velocity: Vec2,
    pub anim_frame: f32,
    pub facing_right: bool,
    pub on_ground: bool,
    pub jump_frames: f32,
    pub tags: Vec<Tag>,

    pub carrying: Option<Pumpkin>,

    body_animation: Animation,
    carry_animation: Animation,
    walk_animation: Animation,
    idle_animation: Animation,
}
impl Player {
    pub fn new() -> Self {
        Self {
            carrying: None,
            pos: Vec2::ZERO,
            camera_pos: Vec2::ZERO,
            velocity: Vec2::ZERO,
            anim_frame: 0.0,
            jump_frames: 0.0,
            facing_right: true,
            on_ground: false,
            tags: Vec::new(),
            body_animation: Animation::from_file(include_bytes!("../assets/player_body.ase")),
            carry_animation: Animation::from_file(include_bytes!("../assets/player_carry.ase")),
            walk_animation: Animation::from_file(include_bytes!("../assets/player_walk.ase")),
            idle_animation: Animation::from_file(include_bytes!("../assets/player_idle.ase")),
        }
    }
    pub fn update(&mut self, world: &mut World, delta_time: f32) {
        self.anim_frame += delta_time * 1000.0;

        // only allow noclip on debug builds
        #[cfg(debug_assertions)]
        let noclip = is_key_down(KeyCode::LeftShift);
        #[cfg(not(debug_assertions))]
        let noclip = { false };

        let can_move = true;

        let mut forces = Vec2::ZERO;

        if !noclip {
            forces.y += GRAVITY;
        }

        let interacted = is_key_pressed(KeyCode::E) || is_mouse_button_pressed(MouseButton::Left);

        if interacted && self.carrying.is_none() {
            let nearby_pumpkin = world
                .pumpkins
                .iter()
                .position(|f| f.pos.distance(self.pos + 4.0) <= PUMPKIN_PICKUP_DIST);
            if let Some(pumpkin) = nearby_pumpkin {
                let pumpkin = world.pumpkins.remove(pumpkin);
                self.carrying = Some(pumpkin);
            }
        } else if interacted {
            let pumpkin = self.carrying.take().unwrap();
            world.pumpkins.push(pumpkin);
        }

        if can_move {
            if is_key_down(KeyCode::A) {
                forces.x -= 1.0 * 3600.0;
                self.facing_right = false;
            }
            if is_key_down(KeyCode::D) {
                forces.x += 1.0 * 3600.0;
                self.facing_right = true;
            }
        }

        if self.on_ground {
            self.jump_frames = 0.0;
        }
        if can_move
            && is_key_down(KeyCode::Space)
            && (self.on_ground || (self.jump_frames > 0.0 && self.jump_frames < 0.2))
        {
            if self.jump_frames == 0.0 {
                self.velocity.y -= 2.5 * 60.0;
            } else {
                forces.y -= delta_time * 3600.0 * 12.0
            }
            self.jump_frames += delta_time;
        }

        if noclip {
            if is_key_down(KeyCode::W) {
                forces.y -= 1.0 * 3600.0;
            }
            if is_key_down(KeyCode::S) {
                forces.y += 1.0 * 3600.0;
            }
            self.velocity += forces * 1.0 * delta_time;

            self.velocity = self.velocity.lerp(Vec2::ZERO, GROUND_FRICTION * delta_time);

            self.pos += self.velocity * delta_time;
            self.camera_pos = self.pos.floor();
            return;
        }

        forces.x -= self.velocity.x
            * if self.on_ground {
                GROUND_FRICTION
            } else {
                AIR_DRAG
            };

        self.velocity += forces * delta_time;

        let mut new = self.pos + self.velocity * delta_time;

        let tile_x = self.pos.x / 8.0;
        let tile_y = self.pos.y / 8.0;

        let tiles_y = [
            (tile_x.trunc(), ceil_g(new.y / 8.0)),
            (ceil_g(tile_x), ceil_g(new.y / 8.0)),
            (tile_x.trunc(), (new.y / 8.0).trunc()),
            (ceil_g(tile_x), (new.y / 8.0).trunc()),
        ];

        let chunks_pos: [(i16, i16); 4] = std::array::from_fn(|f| {
            let cx = ((tiles_y[f].0 / 16.0).floor() * 16.0) as i16;
            let cy = ((tiles_y[f].1 / 16.0).floor() * 16.0) as i16;
            (cx, cy)
        });

        let chunks: Vec<&Chunk> = world
            .collision
            .iter()
            .filter(|f| chunks_pos.contains(&(f.x, f.y)))
            .collect();

        let one_way_chunks: Vec<&Chunk> = world
            .one_way_collision
            .iter()
            .filter(|f| chunks_pos.contains(&(f.x, f.y)))
            .collect();

        self.on_ground = false;
        for (tx, ty) in tiles_y {
            let tile = get_tile(&chunks, tx as i16, ty as i16);
            if tile != 0 {
                let c = if self.velocity.y < 0.0 {
                    tile_y.floor() * 8.0
                } else {
                    self.on_ground = true;
                    tile_y.ceil() * 8.0
                };
                new.y = c;
                self.velocity.y = 0.0;
                break;
            }

            // handle single way platforms
            if self.velocity.y > 0.0
                && ty.trunc() > tile_y.trunc()
                && get_tile(&one_way_chunks, tx as i16, ty as i16) != 0
            {
                new.y = tile_y.ceil() * 8.0;
                self.velocity.y = 0.0;
                self.on_ground = true;
                break;
            }
        }
        let tiles_x = [
            ((new.x / 8.0).trunc(), ceil_g(new.y / 8.0)),
            (ceil_g(new.x / 8.0), ceil_g(new.y / 8.0)),
            (ceil_g(new.x / 8.0), (new.y / 8.0).trunc()),
            ((new.x / 8.0).trunc(), (new.y / 8.0).trunc()),
        ];

        let chunks_pos: [(i16, i16); 4] = std::array::from_fn(|f| {
            let cx = ((tiles_x[f].0 / 16.0).floor() * 16.0) as i16;
            let cy = ((tiles_x[f].1 / 16.0).floor() * 16.0) as i16;
            (cx, cy)
        });

        let chunks: Vec<&Chunk> = world
            .collision
            .iter()
            .filter(|f| chunks_pos.contains(&(f.x, f.y)))
            .collect();

        for (tx, ty) in tiles_x {
            let tile = get_tile(&chunks, tx as i16, ty as i16);
            if tile != 0 {
                let c = if self.velocity.x < 0.0 {
                    tile_x.floor() * 8.0
                } else {
                    tile_x.ceil() * 8.0
                };
                new.x = c;
                self.velocity.x = 0.0;
                break;
            }
        }

        if self.velocity.x.abs() * delta_time <= 0.3 {
            self.velocity.x = 0.0;
        }
        self.velocity.x = self.velocity.x.clamp(-MAX_VELOCITY, MAX_VELOCITY);
        self.pos = new;
        self.camera_pos.x = self.pos.x.floor();
        let delta = self.camera_pos.y - self.pos.y.floor();
        let max_delta = 3.0 * 8.0;
        if can_move && delta.abs() >= max_delta {
            self.camera_pos.y =
                max_delta * if delta < 0.0 { -1.0 } else { 1.0 } + self.pos.y.floor();
        }

        if let Some(pumpkin) = &mut self.carrying {
            pumpkin.pos = self.pos + vec2(4.0, -3.0);
        }
    }
    pub fn draw(&self, assets: &Assets) {
        let torso_animation = if self.carrying.is_some() {
            &self.carry_animation
        } else {
            &self.body_animation
        };
        draw_texture_ex(
            torso_animation.get_at_time(self.anim_frame as u32),
            self.pos.floor().x,
            self.pos.floor().y,
            WHITE,
            DrawTextureParams {
                flip_x: !self.facing_right,
                ..Default::default()
            },
        );
        let legs_animation = if self.velocity.length() > 0.0 {
            &self.walk_animation
        } else {
            &self.idle_animation
        };
        draw_texture_ex(
            legs_animation.get_at_time(self.anim_frame as u32),
            self.pos.floor().x,
            self.pos.floor().y,
            WHITE,
            DrawTextureParams {
                flip_x: !self.facing_right,
                ..Default::default()
            },
        );
        if let Some(pumpkin) = &self.carrying {
            pumpkin.draw(assets, &vec2(-999.0, -999.0));
        }
    }
}
