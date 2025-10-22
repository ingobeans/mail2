use crate::assets::{Chunk, Pumpkin};
use macroquad::prelude::*;

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

pub fn update_physicsbody(
    pos: Vec2,
    velocity: &mut Vec2,
    delta_time: f32,
    collision_tiles: &Vec<Chunk>,
    one_way_tiles: &Vec<Chunk>,
) -> (Vec2, bool) {
    let mut new = pos + *velocity * delta_time;

    let tile_x = pos.x / 8.0;
    let tile_y = pos.y / 8.0;

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

    let chunks: Vec<&Chunk> = collision_tiles
        .iter()
        .filter(|f| chunks_pos.contains(&(f.x, f.y)))
        .collect();

    let one_way_chunks: Vec<&Chunk> = one_way_tiles
        .iter()
        .filter(|f| chunks_pos.contains(&(f.x, f.y)))
        .collect();

    let mut on_ground = false;
    for (tx, ty) in tiles_y {
        let tile = get_tile(&chunks, tx as i16, ty as i16);
        if tile != 0 {
            let c = if velocity.y < 0.0 {
                tile_y.floor() * 8.0
            } else {
                on_ground = true;
                tile_y.ceil() * 8.0
            };
            new.y = c;
            velocity.y = 0.0;
            break;
        }

        // handle single way platforms
        if velocity.y > 0.0
            && ty.trunc() > tile_y.trunc()
            && get_tile(&one_way_chunks, tx as i16, ty as i16) != 0
        {
            new.y = tile_y.ceil() * 8.0;
            velocity.y = 0.0;
            on_ground = true;
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

    let chunks: Vec<&Chunk> = collision_tiles
        .iter()
        .filter(|f| chunks_pos.contains(&(f.x, f.y)))
        .collect();

    for (tx, ty) in tiles_x {
        let tile = get_tile(&chunks, tx as i16, ty as i16);
        if tile != 0 {
            let c = if velocity.x < 0.0 {
                tile_x.floor() * 8.0
            } else {
                tile_x.ceil() * 8.0
            };
            new.x = c;
            velocity.x = 0.0;
            break;
        }
    }
    (new, on_ground)
}

pub fn collide_with_pumpkins(
    mut pos: Vec2,
    velocity: &mut Vec2,
    pumpkins: &Vec<Pumpkin>,
) -> (Vec2, bool) {
    let mut on_ground = false;
    for pumpkin in pumpkins {
        let center_pos = pos + 4.0;
        let inside = (pumpkin.pos.x - 4.0..pumpkin.pos.x + 8.0 + 4.0).contains(&center_pos.x)
            && (pumpkin.pos.y - 4.0..pumpkin.pos.y + 8.0 + 4.0).contains(&center_pos.y);
        if inside {
            let delta_x = pos.x - pumpkin.pos.x;
            let delta_y = center_pos.y - pumpkin.pos.y;

            if delta_y < 0.0 {
                pos.y = pumpkin.pos.y - 4.0 - 4.0;
                if velocity.y > 0.0 {
                    velocity.y = 0.0;
                    on_ground = true;
                }
            } else if delta_x.abs() > delta_y.abs() {
                if delta_x < 0.0 {
                    pos.x = pumpkin.pos.x - 4.0 - 4.0;
                    if velocity.x > 0.0 {
                        velocity.x = 0.0;
                    }
                } else {
                    pos.x = pumpkin.pos.x + 7.0 + 4.0 - 4.0;
                    if velocity.x < 0.0 {
                        velocity.x = 0.0;
                    }
                }
            }
            break;
        }
    }
    (pos, on_ground)
}
