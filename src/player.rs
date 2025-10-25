use macroquad::prelude::*;

use crate::{
    assets::*,
    physics::{collide_with_pumpkins, update_physicsbody},
    utils::*,
};

pub struct Player {
    pub pos: Vec2,
    pub camera_pos: Vec2,
    pub velocity: Vec2,
    pub anim_frame: f32,
    pub facing_right: bool,
    pub on_ground: bool,
    pub jump_frames: f32,

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
        if !noclip && !self.on_ground {
            if forces.x * self.velocity.x > 0.0 {
                forces.x *= 0.02;
            }
        }

        let interacted = is_key_pressed(KeyCode::E) || is_mouse_button_pressed(MouseButton::Left);

        if interacted && self.carrying.is_none() {
            if self.velocity.y == 0.0 {
                let nearby_pumpkin = world
                    .pumpkins
                    .iter()
                    .position(|f| f.within_reach(&self.pos, self.on_ground));
                if let Some(pumpkin) = nearby_pumpkin {
                    let pumpkin = world.pumpkins.remove(pumpkin);
                    self.carrying = Some(pumpkin);
                }
            }
        } else if interacted {
            let mut pumpkin = self.carrying.take().unwrap();
            let input_dir = get_input_axis();
            let mut dir = Vec2::ZERO;
            if input_dir.y < 0.0 && input_dir.x != 0.0 {
                dir = input_dir;
            } else if input_dir.y == 0.0 {
                dir.x = if self.facing_right { 1.0 } else { -1.0 };
            } else if input_dir.y > 0.0 && input_dir.x != 0.0 {
                dir.x = input_dir.x;
            }
            pumpkin.velocity = dir * 3.0 * 60.0;
            world.pumpkins.push(pumpkin);
        }

        if self.on_ground {
            self.jump_frames = 0.0;
        }
        if can_move
            && is_key_down(KeyCode::Space)
            && (self.on_ground || (self.jump_frames > 0.0 && self.jump_frames < 0.5))
        {
            if self.jump_frames == 0.0 {
                self.velocity.y -= 3.0 * 60.0;
            } else {
                //self.velocity.y -= 60.0 * 10.0 * delta_time;
                //forces.y -= 60.0 * 10.0;
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
            self.velocity += forces * 3.0 * delta_time;

            self.velocity = self.velocity.lerp(Vec2::ZERO, GROUND_FRICTION * delta_time);

            self.pos += self.velocity * delta_time;
            self.camera_pos = self.pos.floor();
            return;
        }

        self.velocity += forces * delta_time;
        let on_pumpkin;
        self.velocity.x = self.velocity.x.clamp(-MAX_VELOCITY, MAX_VELOCITY);
        self.velocity.x -= self.velocity.x
            * delta_time
            * if self.on_ground {
                if forces.x == 0.0 {
                    GROUND_FRICTION * 3.0
                } else {
                    GROUND_FRICTION
                }
            } else {
                AIR_DRAG
            };

        (self.pos, on_pumpkin) =
            collide_with_pumpkins(self.pos, &mut self.velocity, &world.pumpkins);
        (self.pos, self.on_ground) = update_physicsbody(
            self.pos,
            &mut self.velocity,
            delta_time,
            &world.collision,
            &world.one_way_collision,
        );
        self.on_ground |= on_pumpkin;

        if self.velocity.x.abs() <= 2.0 && forces.x == 0.0 {
            self.velocity.x = 0.0;
        }
        self.camera_pos.x = self.pos.x.floor();
        let delta = self.camera_pos.y - self.pos.y.floor();
        let max_delta = 3.0 * 8.0;
        if can_move && delta.abs() >= max_delta {
            self.camera_pos.y =
                max_delta * if delta < 0.0 { -1.0 } else { 1.0 } + self.pos.y.floor();
        }

        if let Some(pumpkin) = &mut self.carrying {
            pumpkin.pos = self.pos + vec2(0.0, -7.0);
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
            pumpkin.draw(assets, &vec2(0.0, 0.0), false);
        }
    }
}
