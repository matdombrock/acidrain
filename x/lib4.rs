#[cfg(feature = "buddy-alloc")]
use std::sync::LazyLock; // Add this import
mod alloc;
mod wasm4;
use core::ptr;
use fastrand::Rng;

use wasm4::*;

#[rustfmt::skip]
const SMILEY: [u8; 8] = [
    0b11000011,
    0b10000001,
    0b10110111,
    0b10000001,
    0b00000000,
    0b00000000,
    0b10000001,
    0b10011001,
];
#[rustfmt::skip]
const DRILL: [u8; 8] = [
    0b11111111,
    0b11111111,
    0b10111111,
    0b10001111,
    0b00000111,
    0b10011111,
    0b11111111,
    0b11111111,
];

#[rustfmt::skip]
const DRILL2: [u8; 8] = [
    0b11111111,
    0b11111111,
    0b10011111,
    0b00000111,
    0b10001111,
    0b10111111,
    0b11111111,
    0b11111111,
];

static WORLD_SIZE: usize = 160;
static PLAYER_SIZE: u8 = 8;
static GOLD_COUNT: usize = 32;

pub struct MiniBitVec {
    data: Vec<u8>,
    len: usize,
}

impl MiniBitVec {
    pub fn new() -> Self {
        MiniBitVec {
            data: Vec::new(),
            len: 0,
        }
    }

    pub fn push(&mut self, value: bool) {
        let byte_index = self.len / 8;
        let bit_index = self.len % 8;

        if bit_index == 0 {
            self.data.push(0);
        }

        if value {
            self.data[byte_index] |= 1 << bit_index;
        }

        self.len += 1;
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        if index >= self.len {
            return None;
        }
        let byte_index = index / 8;
        let bit_index = index % 8;
        let byte = self.data[byte_index];
        Some((byte & (1 << bit_index)) != 0)
    }

    pub fn set(&mut self, index: usize, value: bool) {
        if index >= self.len {
            return;
        }
        let byte_index = index / 8;
        let bit_index = index % 8;
        if value {
            self.data[byte_index] |= 1 << bit_index;
        } else {
            self.data[byte_index] &= !(1 << bit_index);
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

#[derive(Copy, Clone)]
struct Pos {
    x: i16,
    y: i16,
}
impl Pos {
    fn new(x: i16, y: i16) -> Self {
        Pos { x, y }
    }
}

struct State {
    rng: Rng,
    frame: u32,
    pos: Pos,
    world: MiniBitVec,
    gold_locs: Vec<Pos>,
    gold_rained: usize,
    rain_locs: Vec<Pos>,
    player_flags_last: u32,
}
impl State {
    fn world_get(&self, x: usize, y: usize) -> Option<bool> {
        let index = y * WORLD_SIZE + x;
        self.world.get(index)
    }
    fn world_set(&mut self, x: usize, y: usize, value: bool) {
        let index = y * WORLD_SIZE + x;
        self.world.set(index, value);
    }
    unsafe fn player_collide(&mut self, cache: Pos) -> bool {
        // Collision with world
        let mut collided = false;
        for dy in 0..PLAYER_SIZE {
            for dx in 0..PLAYER_SIZE {
                let wx = (self.pos.x + dx as i16) as usize;
                let wy = (self.pos.y + dy as i16) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    if let Some(cell) = self.world_get(wx, wy) {
                        if cell {
                            collided = true;
                        }
                    }
                }
            }
        }
        if collided {
            self.pos = cache;
        }
        collided
    }
}

static mut ST: LazyLock<State> = LazyLock::new(|| State {
    rng: Rng::with_seed(1),
    frame: 0,
    pos: Pos { x: 0, y: 0 },
    world: MiniBitVec {
        data: Vec::new(),
        len: 0,
    },
    gold_locs: Vec::new(),
    gold_rained: 0,
    rain_locs: Vec::new(),
    player_flags_last: BLIT_1BPP,
});

#[no_mangle]
#[allow(static_mut_refs)]
unsafe fn start() {
    *PALETTE = [0x001110, 0x506655, 0xD0FFDD, 0xEEFFE0];
    ST.world = MiniBitVec::new();
    for y in 0..WORLD_SIZE {
        for _ in 0..WORLD_SIZE {
            let alive = y >= 16;
            ST.world.push(alive);
        }
    }
    // Generate some random gold locations
    for _ in 0..GOLD_COUNT {
        let x = ST.rng.i16(0..(WORLD_SIZE as i16));
        let y = ST.rng.i16(16..(WORLD_SIZE as i16));
        ST.gold_locs.push(Pos::new(x, y));
    }
}

#[no_mangle]
#[allow(static_mut_refs)]
unsafe fn update() {
    // UPDATE

    let pos_cache = ST.pos;
    let mut dir: u8 = 0; // 0=no,1=left,2=right,3=down
    let mut is_drilling = false;

    let gamepad = *GAMEPAD1;
    if gamepad & BUTTON_1 != 0 {
        is_drilling = true;
    }
    if gamepad & BUTTON_LEFT != 0 {
        ST.pos.x -= 1;
        dir = 1;
    }
    if gamepad & BUTTON_RIGHT != 0 {
        ST.pos.x += 1;
        dir = 2;
    }
    if is_drilling && (gamepad & BUTTON_DOWN != 0) {
        dir = 3;
        // Remove the 4 blocks under the smiley
        for dy in 0..1 {
            for dx in 0..(PLAYER_SIZE + 2) as i16 {
                let wx = (ST.pos.x + dx - 1) as usize;
                let wy = (ST.pos.y + dy + PLAYER_SIZE as i16) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    if ST.rng.i32(0..100) < 50 {
                        ST.world_set(wx, wy, false);
                    }
                }
            }
        }
    }

    if is_drilling && (gamepad & BUTTON_RIGHT != 0) {
        // Remove the 4 blocks to the right of the smiley
        for dy in 0..(PLAYER_SIZE + 1) as i16 {
            for dx in 0..1 {
                let wx = (ST.pos.x - 1 + dx + PLAYER_SIZE as i16) as usize;
                let wy = (ST.pos.y + dy - 1) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    if ST.rng.i32(0..100) < 50 {
                        ST.world_set(wx, wy, false);
                    }
                }
            }
        }
    }
    if is_drilling && (gamepad & BUTTON_LEFT != 0) {
        // Remove the 4 blocks to the left of the smiley
        for dy in 0..(PLAYER_SIZE + 1) as i16 {
            for dx in 0..1 {
                let wx = (ST.pos.x - dx) as usize;
                let wy = (ST.pos.y + dy - 1) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    if ST.rng.i32(0..100) < 50 {
                        ST.world_set(wx, wy, false);
                    }
                }
            }
        }
    }
    ST.player_collide(pos_cache);
    let pos_cache = ST.pos;
    ST.pos.y += 1;
    if ST.pos.y > (WORLD_SIZE - PLAYER_SIZE as usize) as i16 {
        ST.pos.y = (WORLD_SIZE - PLAYER_SIZE as usize) as i16;
    }
    ST.player_collide(pos_cache);

    // Check for gold collection
    ST.gold_locs.retain(|gold| {
        let collected = ST.pos.x < gold.x + 2
            && ST.pos.x + PLAYER_SIZE as i16 > gold.x
            && ST.pos.y < gold.y + 2
            && ST.pos.y + PLAYER_SIZE as i16 > gold.y;
        !collected
    });

    // Add rain
    let mut rain_chance = ST.frame / 100;
    if rain_chance > 100 {
        rain_chance = 100;
    }
    let mut rain_amount = ST.frame / 200;
    if rain_amount > 10 {
        rain_amount = 10;
    }
    for _ in 0..rain_amount {
        if ST.rng.i32(0..100) < rain_chance as i32 {
            let x = ST.rng.i16(0..(WORLD_SIZE as i16));
            ST.rain_locs.push(Pos::new(x, 0));
        }
    }
    // Update rain
    for rain in &mut ST.rain_locs {
        rain.y += 1;
        if ST.rng.i32(0..100) < 10 {
            // Slight horizontal movement
            let dir = ST.rng.i32(0..3) - 1;
            rain.x += dir as i16;
            if rain.x < 0 {
                rain.x = 0;
            }
            if rain.x >= WORLD_SIZE as i16 {
                rain.x = (WORLD_SIZE - 1) as i16;
            }
        }
    }
    // Remove rain that is out of bounds or hit the ground
    let mut rain_hits = Vec::new();
    ST.rain_locs.retain(|rain| {
        if rain.y >= WORLD_SIZE as i16 {
            return false;
        }
        let wx = rain.x as usize;
        let wy = rain.y as usize;
        if wx < WORLD_SIZE && wy < WORLD_SIZE {
            if let Some(cell) = ST.world_get(wx, wy) {
                if cell {
                    rain_hits.push(Pos::new(wx as i16, wy as i16));
                    return false;
                }
            }
        }
        true
    });

    // Create small holes where rain hits
    // Also remove gold if rain hits it
    for hit in rain_hits {
        for dy in 0..2 {
            for dx in 0..2 {
                let wx = (hit.x + dx) as usize;
                let wy = (hit.y + dy) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    ST.world_set(wx, wy, false);
                    // Remove gold if hit
                    ST.gold_locs.retain(|gold| {
                        !(gold.x >= wx as i16
                            && gold.x < (wx + 2) as i16
                            && gold.y >= wy as i16
                            && gold.y < (wy + 2) as i16)
                    });
                }
            }
        }
    }

    // If world blocks have less than 4 neighbors, they fall down
    let mut to_fall = Vec::new();
    for y in (1..WORLD_SIZE - 1).rev() {
        for x in 1..WORLD_SIZE - 1 {
            if let Some(cell) = ST.world_get(x, y) {
                if cell {
                    let mut neighbors = 0;
                    for oy in -1..=1 {
                        for ox in -1..=1 {
                            if ox == 0 && oy == 0 {
                                continue;
                            }
                            if let Some(ncell) =
                                ST.world_get((x as i32 + ox) as usize, (y as i32 + oy) as usize)
                            {
                                if ncell {
                                    neighbors += 1;
                                }
                            }
                        }
                    }
                    if neighbors < 4 {
                        to_fall.push((x, y));
                    }
                }
            }
        }
    }
    for (x, y) in to_fall {
        if ST.frame % 6 == 0 {
            // Check for collision below
            if let Some(below) = ST.world_get(x, y + 1) {
                if !below {
                    // Check for player collision
                    let mut collide_with_player = false;
                    for dy in 0..PLAYER_SIZE as i16 {
                        for dx in 0..PLAYER_SIZE as i16 {
                            let px = ST.pos.x + dx;
                            let py = ST.pos.y + dy;
                            if px == x as i16 && py == (y as i16 + 1) {
                                collide_with_player = true;
                            }
                        }
                    }
                    if !collide_with_player {
                        ST.world_set(x, y, false);
                        ST.world_set(x, y + 1, true);
                    }
                }
            }
        }
    }

    ST.frame += 1;

    // DRAW

    // Render the world
    for y in 0..WORLD_SIZE {
        for x in 0..WORLD_SIZE {
            if let Some(cell) = ST.world_get(x, y) {
                if cell {
                    *DRAW_COLORS = 2;
                    rect(x as i32, y as i32, 1, 1);
                }
            }
        }
    }
    *DRAW_COLORS = 4;

    let player_flags = match dir {
        0 => ST.player_flags_last,
        1 => BLIT_1BPP | BLIT_FLIP_X,
        2 => BLIT_1BPP,
        _ => ST.player_flags_last,
    };
    ST.player_flags_last = player_flags;
    blit(
        &SMILEY,
        ST.pos.x as i32,
        ST.pos.y as i32,
        8,
        PLAYER_SIZE as u32,
        player_flags,
    );

    let drill_off = match dir {
        0 => Pos::new(PLAYER_SIZE as i16, 0),
        1 => Pos::new(-(PLAYER_SIZE as i16), 0),
        2 => Pos::new(PLAYER_SIZE as i16, 0),
        3 => Pos::new(0, PLAYER_SIZE as i16),
        _ => Pos::new(PLAYER_SIZE as i16, 0),
    };
    let drill_flags = match dir {
        0 => BLIT_1BPP,
        1 => BLIT_1BPP | BLIT_FLIP_X,
        2 => BLIT_1BPP,
        3 => BLIT_1BPP | BLIT_FLIP_Y | BLIT_FLIP_X | BLIT_ROTATE,
        _ => BLIT_1BPP,
    };
    let drill_show = match dir {
        0 => false,
        1 => true,
        2 => true,
        3 => true,
        _ => false,
    };
    if drill_show && is_drilling {
        let drill_frame = (ST.frame / 5) % 2;
        let drill_sprite = if drill_frame == 0 { &DRILL } else { &DRILL2 };
        blit(
            drill_sprite,
            (ST.pos.x + drill_off.x) as i32,
            (ST.pos.y + drill_off.y) as i32,
            8,
            PLAYER_SIZE as u32,
            drill_flags,
        );
    }
    // Render gold locations
    for gold in &ST.gold_locs {
        *DRAW_COLORS = 3;
        rect(gold.x as i32, gold.y as i32, 2, 2);
    }
    let gold = GOLD_COUNT - ST.gold_locs.len();
    text(gold.to_string(), 4, 2);

    // Render rain
    for rain in &ST.rain_locs {
        *DRAW_COLORS = 3;
        rect(rain.x as i32, rain.y as i32, 1, 1);
    }
}
