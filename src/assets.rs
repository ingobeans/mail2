use asefile::AsepriteFile;
use hashmap_macro::hashmap;
use image::EncodableLayout;
use macroquad::prelude::*;

use crate::{physics::update_physicsbody, utils::*};

pub struct Assets {
    font: Spritesheet,
    tileset: Spritesheet,
    pub poi: Animation,
}
impl Default for Assets {
    fn default() -> Self {
        Self {
            font: Spritesheet::new(
                load_ase_texture(include_bytes!("../assets/font.ase"), None),
                4.0,
            ),
            tileset: Spritesheet::new(
                load_ase_texture(include_bytes!("../assets/tileset.ase"), None),
                8.0,
            ),
            poi: Animation::from_file(include_bytes!("../assets/poi.ase")),
        }
    }
}
impl Assets {
    pub fn draw_text(&self, text: &str, mut x: f32, mut y: f32) -> (f32, f32) {
        let original_x = x;
        let original_y = y;
        let hardcoded = hashmap!(':'=>36,'.'=>37,'-'=>38,'%'=>39,'+'=>40,'/'=>41,'H'=>42,'('=>43,')'=>44,'!'=>45,'?'=>46);
        let mut start_of_line = true;

        for char in text.chars() {
            if char == '\n' {
                start_of_line = true;
                y += 5.0;
                x = original_x;
                continue;
            } else if char == ' ' {
                if start_of_line {
                    continue;
                }
                x += 4.0;
                continue;
            }
            let code = char as u8;

            let index = if let Some(value) = hardcoded.get(&char) {
                *value
            } else if code.is_ascii_lowercase() {
                code - b'a'
            } else if code.is_ascii_digit() {
                code - b'0' + 26
            } else {
                continue;
            };
            start_of_line = false;
            self.font
                .draw_sprite(x + 2.0, y + 2.0, index as f32, 0.0, None);

            x += 4.0
        }

        gl_use_default_material();
        (x - original_x, y - original_y)
    }
}
fn load_ase_texture(bytes: &[u8], layer: Option<u32>) -> Texture2D {
    let img = AsepriteFile::read(bytes).unwrap();
    let img = if let Some(layer) = layer {
        img.layer(layer).frame(0).image()
    } else {
        img.frame(0).image()
    };
    let new = Image {
        width: img.width() as u16,
        height: img.height() as u16,
        bytes: img.as_bytes().to_vec(),
    };
    let texture = Texture2D::from_image(&new);
    texture.set_filter(FilterMode::Nearest);
    texture
}

pub struct Spritesheet {
    pub texture: Texture2D,
    pub sprite_size: f32,
}
impl Spritesheet {
    pub fn new(texture: Texture2D, sprite_size: f32) -> Self {
        Self {
            texture,
            sprite_size,
        }
    }
    /// Same as `draw_tile`, except centered
    pub fn draw_sprite(
        &self,
        screen_x: f32,
        screen_y: f32,
        tile_x: f32,
        tile_y: f32,
        params: Option<&DrawTextureParams>,
    ) {
        self.draw_tile(
            screen_x - self.sprite_size / 2.0,
            screen_y - self.sprite_size / 2.0,
            tile_x,
            tile_y,
            params,
        );
    }
    /// Draws a single tile from the spritesheet
    pub fn draw_tile(
        &self,
        screen_x: f32,
        screen_y: f32,
        tile_x: f32,
        tile_y: f32,
        params: Option<&DrawTextureParams>,
    ) {
        let mut p = params.cloned().unwrap_or(DrawTextureParams::default());
        p.dest_size = p
            .dest_size
            .or(Some(Vec2::new(self.sprite_size, self.sprite_size)));
        p.source = p.source.or(Some(Rect {
            x: tile_x * self.sprite_size,
            y: tile_y * self.sprite_size,
            w: self.sprite_size,
            h: self.sprite_size,
        }));
        draw_texture_ex(&self.texture, screen_x, screen_y, WHITE, p);
    }
}

pub struct Animation {
    frames: Vec<(Texture2D, u32)>,
    pub total_length: u32,
}
impl Animation {
    pub fn from_file(bytes: &[u8]) -> Self {
        let ase = AsepriteFile::read(bytes).unwrap();
        let mut frames = Vec::new();
        let mut total_length = 0;
        for index in 0..ase.num_frames() {
            let frame = ase.frame(index);
            let img = frame.image();
            let new = Image {
                width: img.width() as u16,
                height: img.height() as u16,
                bytes: img.as_bytes().to_vec(),
            };
            let duration = frame.duration();
            total_length += duration;
            let texture = Texture2D::from_image(&new);
            frames.push((texture, duration));
        }
        Self {
            frames,
            total_length,
        }
    }
    pub fn get_at_time(&self, mut time: u32) -> &Texture2D {
        time %= self.total_length;
        for (texture, length) in self.frames.iter() {
            if time >= *length {
                time -= length;
            } else {
                return texture;
            }
        }
        panic!()
    }
}

pub struct Pumpkin {
    pub pos: Vec2,
    pub velocity: Vec2,
    pub on_ground: bool,
}
impl Pumpkin {
    pub fn update(&mut self, delta_time: f32, collision_tiles: &[Chunk], one_way_tiles: &[Chunk]) {
        self.velocity.y += GRAVITY * delta_time;
        self.velocity.x -=
            self.velocity.x * if self.on_ground { GROUND_FRICTION } else { 0.0 } * delta_time;

        if self.velocity.x.abs() <= 2.0 {
            self.velocity.x = 0.0;
            self.pos = self.pos.round();
        }
        (self.pos, self.on_ground) = update_physicsbody(
            self.pos,
            &mut self.velocity,
            delta_time,
            collision_tiles,
            one_way_tiles,
        );
    }
    pub fn within_reach(&self, player_pos: &Vec2, player_grounded: bool) -> bool {
        if !player_grounded {
            return false;
        }
        let weighted_dist =
            (self.pos.x - player_pos.x).powi(2) + (self.pos.y - player_pos.y).powi(2) * 1.5;
        weighted_dist <= PUMPKIN_PICKUP_DIST.powi(2)
    }
    pub fn draw(&self, assets: &Assets, player_pos: &Vec2, player_grounded: bool) {
        let tile_y = if self.within_reach(player_pos, player_grounded) {
            3.0
        } else {
            2.0
        };
        assets.tileset.draw_sprite(
            self.pos.floor().x + 4.0,
            self.pos.floor().y + 4.0,
            0.0,
            tile_y,
            None,
        );
    }
}

pub struct World {
    pub collision: Vec<Chunk>,
    pub one_way_collision: Vec<Chunk>,
    pub details: Vec<Chunk>,
    pub background: Vec<Chunk>,
    pub interactable: Vec<Chunk>,

    pub pumpkins: Vec<Pumpkin>,

    pub x_min: i16,
    pub x_max: i16,
    pub y_min: i16,
    pub y_max: i16,
}
#[expect(dead_code)]
impl World {
    pub fn get_interactable_spawn(&self, tile_index: i16) -> Option<Vec2> {
        for chunk in self.interactable.iter() {
            for (i, tile) in chunk.tiles.iter().enumerate() {
                if *tile == tile_index + 1 {
                    return Some(Vec2::new(
                        (i as i16 % 16 + chunk.x) as f32 * 8.0,
                        (i as i16 / 16 + chunk.y) as f32 * 8.0,
                    ));
                }
            }
        }
        None
    }
    pub fn set_collision_tile(&mut self, x: i16, y: i16, tile: i16) {
        let cx = ((x as f32 / 16.0).floor() * 16.0) as i16;
        let cy = ((y as f32 / 16.0).floor() * 16.0) as i16;

        let chunk = self
            .collision
            .iter_mut()
            .find(|f| f.x == cx && f.y == cy)
            .unwrap();
        chunk.tiles[(x - chunk.x + (y - chunk.y) * 16) as usize] = tile;
    }
}
impl Default for World {
    fn default() -> Self {
        let xml = include_str!("../assets/world/world.tmx");
        let collision = get_layer(xml, "Collision");
        let one_way_collision = get_layer(xml, "OneWayCollision");
        let detail = get_layer(xml, "Detail");
        let interactable = get_layer(xml, "Interactable");
        let background = get_layer(xml, "Background");
        let mut world = World {
            collision: get_all_chunks(collision),
            one_way_collision: get_all_chunks(one_way_collision),
            details: get_all_chunks(detail),
            background: get_all_chunks(background),
            interactable: get_all_chunks(interactable),
            x_min: 999,
            y_min: 999,
            y_max: -999,
            x_max: -999,
            pumpkins: Vec::new(),
        };

        // define x y min and max
        for layer in [
            &world.collision,
            &world.one_way_collision,
            &world.details,
            &world.background,
            &world.interactable,
        ] {
            for chunk in layer {
                if chunk.x < world.x_min {
                    world.x_min = chunk.x;
                }
                if chunk.y < world.y_min {
                    world.y_min = chunk.y;
                }
                if chunk.x > world.x_max {
                    world.x_max = chunk.x;
                }
                if chunk.y > world.y_max {
                    world.y_max = chunk.y;
                }
            }
        }

        // populate pumpkins array
        for chunk in &world.interactable {
            for (index, tile) in chunk.tiles.iter().enumerate() {
                let x = (index % 16) as i16 + chunk.x;
                let y = (index / 16) as i16 + chunk.y;
                if *tile == 64 + 1 {
                    world.pumpkins.push(Pumpkin {
                        pos: vec2((x * 8) as f32, (y * 8) as f32),
                        velocity: Vec2::ZERO,
                        on_ground: true,
                    });
                }
            }
        }

        world
    }
}

pub struct Chunk {
    pub x: i16,
    pub y: i16,
    pub tiles: Vec<i16>,
}
impl Chunk {
    pub fn tile_at(&self, x: usize, y: usize) -> Option<i16> {
        if x > 16 {
            return None;
        }
        self.tiles.get(x + y * 16).cloned()
    }
    pub fn draw(&self, assets: &Assets) {
        for (index, tile) in self.tiles.iter().enumerate() {
            if *tile == 0 {
                continue;
            }
            let tile = *tile - 1;
            let x = index % 16;
            let y = index / 16;
            assets.tileset.draw_tile(
                (self.x * 8) as f32 + (x * 8) as f32,
                (self.y * 8) as f32 + (y * 8) as f32,
                (tile % 32) as f32,
                (tile / 32) as f32,
                None,
            );
        }
    }
}

fn get_all_chunks(xml: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut xml = xml.to_string();
    while let Some((current, remains)) = xml.split_once("</chunk>") {
        let new = parse_chunk(current);
        chunks.push(new);
        xml = remains.to_string();
    }

    chunks
}

fn get_layer<'a>(xml: &'a str, layer: &str) -> &'a str {
    let split = format!(" name=\"{layer}");
    xml.split_once(&split)
        .unwrap()
        .1
        .split_once(">")
        .unwrap()
        .1
        .split_once("</layer>")
        .unwrap()
        .0
}

fn parse_chunk(xml: &str) -> Chunk {
    let (tag, data) = xml
        .split_once("<chunk ")
        .unwrap()
        .1
        .split_once(">")
        .unwrap();

    let x = tag
        .split_once("x=\"")
        .unwrap()
        .1
        .split_once("\"")
        .unwrap()
        .0
        .parse()
        .unwrap();
    let y = tag
        .split_once("y=\"")
        .unwrap()
        .1
        .split_once("\"")
        .unwrap()
        .0
        .parse()
        .unwrap();

    let mut split = data.split(',');

    let mut chunk = vec![0; 16 * 16];
    for item in &mut chunk {
        let a = split.next().unwrap().trim();
        *item = a.parse().unwrap()
    }
    Chunk { x, y, tiles: chunk }
}
