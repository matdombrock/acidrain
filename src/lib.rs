use std::sync::LazyLock;
#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
use fastrand::Rng;

use wasm4::*;

#[rustfmt::skip]
const SMILEY1: [u8; 8] = [
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
const SMILEY2: [u8; 8] = [
    0b11000011,
    0b10000001,
    0b10110111,
    0b10000001,
    0b00000000,
    0b00000000,
    0b10000001,
    0b11111001,
];
#[rustfmt::skip]
const SMILEY3: [u8; 8] = [
    0b11000011,
    0b10000001,
    0b10110111,
    0b10000001,
    0b00000000,
    0b00000000,
    0b10000001,
    0b10011111,
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

#[rustfmt::skip]
const DOOR1: [u8; 8] = [
    0b10000001,
    0b01111110,
    0b00000000,
    0b01111110,
    0b00000000,
    0b01111110,
    0b00000000,
    0b01111110,
];
#[rustfmt::skip]
const DOOR2: [u8; 8] = [
    0b10000001,
    0b01111110,
    0b01111110,
    0b00000000,
    0b01111110,
    0b00000000,
    0b01111110,
    0b00000000,
];
#[rustfmt::skip]
const HEART: [u8; 8] = [
    0b10011001,
    0b00011000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b10000001,
    0b11000011,
    0b11100111,
];
#[rustfmt::skip]
const GOLD1: [u8; 8] = [
    0b10011111,
    0b00101111,
    0b01101111,
    0b10011111,
    0b1111111,
    0b11111111,
    0b11111111,
    0b11111111,
];
#[rustfmt::skip]
const GOLD2: [u8; 8] = [
    0b10011111,
    0b01101111,
    0b01001111,
    0b10011111,
    0b11111111,
    0b11111111,
    0b11111111,
    0b11111111,
];
#[rustfmt::skip]
const DRONE1: [u8; 8] = [
    0b10000001,
    0b00100100,
    0b00000000,
    0b10111101,
    0b11111111,
    0b11111111,
    0b11111111,
    0b11111111,
];
#[rustfmt::skip]
const DRONE2: [u8; 8] = [
    0b10000001,
    0b00100100,
    0b00000000,
    0b10011001,
    0b11111111,
    0b11111111,
    0b11111111,
    0b11111111,
];
#[rustfmt::skip]
const FLY1: [u8; 8] = [
    0b11000011,
    0b11100111,
    0b11100111,
    0b11000011,
    0b11111111,
    0b11111111,
    0b11111111,
    0b11111111,
];
#[rustfmt::skip]
const FLY2: [u8; 8] = [
    0b11000011,
    0b00100100,
    0b00100100,
    0b11000011,
    0b11111111,
    0b11111111,
    0b11111111,
    0b11111111,
];
#[rustfmt::skip]
const SLIDER1: [u8; 8] = [
    0b11000011,
    0b01101101,
    0b10000000,
    0b11000011,
    0b11111111,
    0b11111111,
    0b11111111,
    0b11111111,
];
#[rustfmt::skip]
const SLIDER2: [u8; 8] = [
    0b11000011,
    0b10110110,
    0b00000001,
    0b11000010,
    0b11111111,
    0b11111111,
    0b11111111,
    0b11111111,
];
#[rustfmt::skip]
const LOGO: [u8; 16] = [
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
];

static WORLD_SIZE: usize = 160;
static PLAYER_SIZE: u8 = 8;
static GOLD_COUNT: usize = 64;
static RAIN_MAX: usize = 500;
static PAL: [u32; 4] = [0x001110, 0x506655, 0xD0FFDD, 0xEEFFE0];
static PAL_DMG: [u32; 4] = [0x221110, 0x506655, 0xD0FFDD, 0xEEFFE0];
static PAL_GAMEOVER: [u32; 4] = [0x551110, 0x506655, 0xD0FFDD, 0xEEFFE0];
static DMG_FRAMES: u8 = 16;

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
    fn distance(&self, other: &Pos) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        ((dx * dx + dy * dy) as f32).sqrt()
    }
}

#[derive(PartialEq)]
enum Screen {
    Start,
    Game,
    GameOver,
}

struct State {
    rng: Rng,
    seed: u64,
    frame: u32,
    hp: u8,
    pos: Pos,
    dir: u8,
    world: MiniBitVec,
    exit_loc: Pos,
    gold_locs: Vec<Pos>,
    gold_rained: usize,
    gold: usize,
    rain_locs: Vec<Pos>,
    rain_chance_rte: u8,
    rain_amount_rte: u8,
    drone_locs: Vec<Pos>,
    fly_locs: [Pos; 8],
    slider_locs: [Pos; 8],
    player_flags_last: u32,
    dmg_frames: u8,
    has_drilled: bool,
    has_gold: bool,
    is_drilling: bool,
    screen: Screen,
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
    unsafe fn clear_at_player(&mut self) {
        for dy in 0..PLAYER_SIZE as i16 {
            for dx in 0..PLAYER_SIZE as i16 {
                let wx = (self.pos.x + dx) as usize;
                let wy = (self.pos.y + dy) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    self.world_set(wx, wy, false);
                }
            }
        }
    }
    unsafe fn player_wrap(&mut self) {
        if self.pos.x < 0 {
            self.pos.x = (WORLD_SIZE - PLAYER_SIZE as usize) as i16;
            self.clear_at_player();
        }
        if self.pos.x > (WORLD_SIZE - PLAYER_SIZE as usize) as i16 {
            self.pos.x = 0;
            self.clear_at_player();
        }
    }
    #[allow(static_mut_refs)]
    unsafe fn reset(&mut self, full: bool) {
        // Reset game state
        ST.frame = 0;
        ST.pos = Pos { x: 76, y: 0 };
        ST.world = MiniBitVec::new();
        ST.exit_loc = Pos { x: 0, y: 0 };
        ST.gold_locs.clear();
        ST.gold_rained = 0;
        ST.rain_locs.clear();
        ST.player_flags_last = BLIT_1BPP;
        ST.dmg_frames = 0;
        ST.has_gold = false;
        ST.is_drilling = false;
        ST.screen = Screen::Start;
        ST.dir = 0;
        ST.drone_locs.clear();
        if full {
            ST.gold = 0;
            ST.hp = 10;
            ST.seed = 0;
            ST.has_drilled = false;
        }
    }
}

static mut ST: LazyLock<State> = LazyLock::new(|| State {
    rng: Rng::new(),
    seed: 0,
    frame: 0,
    hp: 10,
    pos: Pos { x: 76, y: 0 },
    dir: 0,
    world: MiniBitVec {
        data: Vec::new(),
        len: 0,
    },
    exit_loc: Pos { x: 0, y: 0 },
    gold_locs: Vec::new(),
    gold_rained: 0,
    gold: 0,
    rain_locs: Vec::new(),
    rain_chance_rte: 100,
    rain_amount_rte: 200,
    drone_locs: Vec::new(),
    fly_locs: [Pos { x: 0, y: 0 }; 8],
    slider_locs: [Pos { x: 0, y: 0 }; 8],
    player_flags_last: BLIT_1BPP,
    dmg_frames: 0,
    has_drilled: false,
    has_gold: false,
    is_drilling: false,
    screen: Screen::Start,
});

#[allow(static_mut_refs)]
unsafe fn sfx_dig() {
    let max = 440 - ST.pos.y as u32 * 2; // 160
    let f = ST.rng.u32(120..max);
    tone(f, 0, 50, TONE_NOISE);
}

#[allow(static_mut_refs)]
unsafe fn sfx_rain(p: &Pos) {
    let f = ST.rng.u32(440..880);
    let dist = ((ST.pos.x - p.x).abs() + (ST.pos.y - p.y).abs()) as u32;
    let vol = 5 + (if dist > 50 { 20 } else { 50 - dist });
    tone(f, 0, vol, TONE_PULSE2);
}

#[allow(static_mut_refs)]
unsafe fn sfx_gold() {
    let f = ST.rng.u32(400..440);
    tone(f, 4, 128, TONE_PULSE1);
}

#[allow(static_mut_refs)]
unsafe fn sfx_dmg() {
    let f = ST.rng.u32(200..220);
    tone(f, 4, 128, TONE_PULSE1);
}

#[no_mangle]
#[allow(static_mut_refs)]
unsafe fn start() {
    *PALETTE = PAL;
    ST.world = MiniBitVec::new();
}

#[no_mangle]
#[allow(static_mut_refs)]
unsafe fn gen_world() {
    for y in 0..WORLD_SIZE {
        for _ in 0..WORLD_SIZE {
            let mut alive = y >= 24;
            if ST.rng.i32(0..100) < 2 {
                alive = false;
            }
            ST.world.push(alive);
        }
    }
    // Generate some random gold locations
    for _ in 0..GOLD_COUNT {
        let x = ST.rng.i16(0..(WORLD_SIZE as i16));
        let y = ST.rng.i16(24..(WORLD_SIZE as i16));
        ST.gold_locs.push(Pos::new(x, y));
    }
    // Exit location
    let exit_x = ST.rng.i16(0..(WORLD_SIZE as i16));
    ST.exit_loc = Pos::new(exit_x, 152);
    // Fly locations
    for fly in &mut ST.fly_locs {
        let x = ST.rng.i16(0..(WORLD_SIZE as i16));
        let y = ST.rng.i16(24..(WORLD_SIZE as i16));
        *fly = Pos::new(x, y);
    }
    // Slider locations
    for slider in &mut ST.slider_locs {
        let x = ST.rng.i16(0..(WORLD_SIZE as i16));
        let y = ST.rng.i16(24..(WORLD_SIZE as i16));
        *slider = Pos::new(x, y);
    }
}

#[no_mangle]
#[allow(static_mut_refs)]
unsafe fn main_logic() {
    let pos_cache = ST.pos;
    ST.is_drilling = false;
    ST.dir = 0;
    let gamepad = *GAMEPAD1;
    if gamepad & BUTTON_1 != 0 {
        ST.is_drilling = true;
    }
    if gamepad & BUTTON_LEFT != 0 {
        ST.pos.x -= 1;
        ST.dir = 1;
    }
    if gamepad & BUTTON_RIGHT != 0 {
        ST.pos.x += 1;
        ST.dir = 2;
    }
    if ST.is_drilling && (gamepad & BUTTON_DOWN != 0) {
        ST.has_drilled = true;
        ST.dir = 3;
        // Remove the 4 blocks under the smiley
        for dy in 0..1 {
            for dx in 0..(PLAYER_SIZE + 2) as i16 {
                let wx = (ST.pos.x + dx - 1) as usize;
                let wy = (ST.pos.y + dy + PLAYER_SIZE as i16) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    if ST.rng.i32(0..100) < 50 {
                        ST.world_set(wx, wy, false);
                        sfx_dig();
                    }
                }
            }
        }
    }

    if ST.is_drilling && (gamepad & BUTTON_RIGHT != 0) {
        ST.has_drilled = true;
        // Remove the 4 blocks to the right of the smiley
        for dy in 0..(PLAYER_SIZE + 1) as i16 {
            for dx in 0..1 {
                let wx = (ST.pos.x - 1 + dx + PLAYER_SIZE as i16) as usize;
                let wy = (ST.pos.y + dy - 1) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    if ST.rng.i32(0..100) < 50 {
                        ST.world_set(wx, wy, false);
                        sfx_dig();
                    }
                }
            }
        }
    }
    if ST.is_drilling && (gamepad & BUTTON_LEFT != 0) {
        ST.has_drilled = true;
        // Remove the 4 blocks to the left of the smiley
        for dy in 0..(PLAYER_SIZE + 1) as i16 {
            for dx in 0..1 {
                let wx = (ST.pos.x - dx) as usize;
                let wy = (ST.pos.y + dy - 1) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    if ST.rng.i32(0..100) < 50 {
                        ST.world_set(wx, wy, false);
                        sfx_dig();
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
    ST.player_wrap();

    // Check for gold collection
    ST.gold_locs.retain(|gold| {
        let collected = ST.pos.x < gold.x + 4
            && ST.pos.x + PLAYER_SIZE as i16 > gold.x
            && ST.pos.y < gold.y + 4
            && ST.pos.y + PLAYER_SIZE as i16 > gold.y;
        if collected {
            sfx_gold();
            ST.gold += 1;
            false
        } else {
            true
        }
    });

    // Add drones
    if ST.frame % 200 == 0 && ST.drone_locs.len() < 5 {
        let x = ST.rng.i16(0..(WORLD_SIZE as i16));
        ST.drone_locs.push(Pos::new(x, 0));
    }
    // Update drones
    // Move towards player
    if ST.frame % 16 == 0 {
        for drone in &mut ST.drone_locs {
            let dx = ST.pos.x - drone.x;
            let dy = ST.pos.y - drone.y;
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist > 0.0 {
                let step_x = (dx as f32 / dist).round() as i16;
                let step_y = (dy as f32 / dist).round() as i16;
                drone.x += step_x;
                drone.y += step_y;
            }
            // Check for collision with player
            for py in 0..PLAYER_SIZE as i16 {
                for px in 0..PLAYER_SIZE as i16 {
                    let px_pos = ST.pos.x + px;
                    let py_pos = ST.pos.y + py;
                    if px_pos == drone.x && py_pos == drone.y {
                        sfx_dmg();
                    }
                }
            }
            // Remove world blocks at drone position
            for dy in 0..4 {
                for dx in 0..6 {
                    let wx = (drone.x + dx) as usize;
                    let wy = (drone.y + dy) as usize;
                    if wx < WORLD_SIZE && wy < WORLD_SIZE {
                        ST.world_set(wx, wy, false);
                    }
                }
            }
        }
    }

    // Add rain
    let mut rain_chance = ST.frame / ST.rain_chance_rte as u32;
    if rain_chance > 100 {
        rain_chance = 100;
    }
    let mut rain_amount = ST.frame / ST.rain_amount_rte as u32;
    if rain_amount > 10 {
        rain_amount = 10;
    }
    if ST.rain_locs.len() < RAIN_MAX {
        for _ in 0..rain_amount {
            if ST.rng.i32(0..100) < rain_chance as i32 {
                let x = ST.rng.i16(0..(WORLD_SIZE as i16));
                ST.rain_locs.push(Pos::new(x, 0));
            }
        }
    }
    // Update rain
    for rain in &mut ST.rain_locs {
        rain.y += 1;
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
                    sfx_rain(rain);
                    return false;
                }
            }
            // Check for player collision
            for dy in 0..PLAYER_SIZE as i16 {
                for dx in 0..PLAYER_SIZE as i16 {
                    let px = ST.pos.x + dx;
                    let py = ST.pos.y + dy;
                    if px == rain.x && py == rain.y {
                        rain_hits.push(Pos::new(wx as i16, wy as i16));
                        ST.dmg_frames = DMG_FRAMES;
                        ST.hp = ST.hp.saturating_sub(1);
                        sfx_dmg();
                        return false;
                    }
                }
            }
            // Remove gold if hit
            ST.gold_locs.retain(|gold| {
                let hit = gold.x >= wx as i16
                    && gold.x < (wx + 4) as i16
                    && gold.y >= wy as i16
                    && gold.y < (wy + 4) as i16;
                if hit {
                    tone(280, 1, 100, TONE_PULSE2);
                    ST.gold_rained += 1;
                    // Remove a chunk of world where gold was
                    for gy in 0..8 {
                        for gx in 0..8 {
                            let gwx = (gold.x + gx) as usize;
                            let gwy = (gold.y + gy) as usize;
                            if gwx < WORLD_SIZE && gwy < WORLD_SIZE {
                                ST.world_set(gwx, gwy, false);
                            }
                        }
                    }
                }
                !hit
            });
        }
        true
    });
    // Create small holes where rain hits
    for hit in rain_hits {
        for dy in 0..2 {
            for dx in 0..2 {
                let wx = (hit.x + dx) as usize;
                let wy = (hit.y + dy) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    ST.world_set(wx, wy, false);
                }
            }
        }
    }

    // Update fly location
    if ST.frame % 8 == 0 {
        for fly in &mut ST.fly_locs {
            let dir = ST.rng.i32(0..4);
            match dir {
                0 => {
                    fly.x += 1;
                    if fly.x >= WORLD_SIZE as i16 {
                        fly.x = 0;
                    }
                }
                1 => {
                    fly.x -= 1;
                    if fly.x < 0 {
                        fly.x = (WORLD_SIZE - 1) as i16;
                    }
                }
                2 => {
                    fly.y += 1;
                    if fly.y >= WORLD_SIZE as i16 {
                        fly.y = 0;
                    }
                }
                3 => {
                    fly.y -= 1;
                    if fly.y < 0 {
                        fly.y = (WORLD_SIZE - 1) as i16;
                    }
                }
                _ => {}
            }
            // Check for collision with player
            for py in 0..PLAYER_SIZE as i16 {
                for px in 0..PLAYER_SIZE as i16 {
                    let px_pos = ST.pos.x + px;
                    let py_pos = ST.pos.y + py;
                    if px_pos == fly.x && py_pos == fly.y {
                        sfx_dmg();
                    }
                }
            }
            // Destroy world blocks at fly position
            for dy in 0..4 {
                for dx in 0..4 {
                    let wx = (fly.x + dx) as usize;
                    let wy = (fly.y + dy) as usize;
                    if wx < WORLD_SIZE && wy < WORLD_SIZE {
                        ST.world_set(wx, wy, false);
                    }
                }
            }
        }
    }

    // Sliders move left and right only
    if ST.frame % 8 == 0 {
        for slider in &mut ST.slider_locs {
            let dir = ST.rng.i32(0..2);
            match dir {
                0 => {
                    slider.x += 4;
                    if slider.x >= WORLD_SIZE as i16 {
                        slider.x = 0;
                    }
                }
                1 => {
                    slider.x -= 4;
                    if slider.x < 0 {
                        slider.x = (WORLD_SIZE - 1) as i16;
                    }
                }
                _ => {}
            }
            // Check for collision with player
            for py in 0..PLAYER_SIZE as i16 {
                for px in 0..PLAYER_SIZE as i16 {
                    let px_pos = ST.pos.x + px;
                    let py_pos = ST.pos.y + py;
                    if px_pos == slider.x && py_pos == slider.y {
                        sfx_dmg();
                    }
                }
            }
            // Destroy world blocks at slider position
            for dy in 0..4 {
                for dx in 0..6 {
                    let wx = (slider.x + dx) as usize;
                    let wy = (slider.y + dy) as usize;
                    if wx < WORLD_SIZE && wy < WORLD_SIZE {
                        ST.world_set(wx, wy, false);
                    }
                }
            }
        }
    }

    // Check for collisions with doors
    let door_collide = ST.pos.x < ST.exit_loc.x + 8
        && ST.pos.x + PLAYER_SIZE as i16 > ST.exit_loc.x
        && ST.pos.y < ST.exit_loc.y + 8
        && ST.pos.y + PLAYER_SIZE as i16 > ST.exit_loc.y;
    if door_collide {
        // Win the game (reset for now)
        ST.reset(false);
        return;
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

    // If took damage, change palette briefly
    if ST.dmg_frames > 0 {
        *PALETTE = PAL_DMG;
        ST.dmg_frames -= 1;
    } else {
        *PALETTE = PAL;
    }
    // Check for game over
    if ST.hp == 0 {
        ST.screen = Screen::GameOver;
    }
}

#[no_mangle]
#[allow(static_mut_refs)]
unsafe fn update() {
    // UPDATE
    if ST.screen == Screen::Start {
        let gamepad = *GAMEPAD1;
        if gamepad != 0 {
            ST.screen = Screen::Game;
            // Seed random with current frame
            ST.rng = Rng::with_seed(ST.seed);
            gen_world();
        }
        ST.seed += 1;
    } else if ST.screen == Screen::Game {
        main_logic();
        ST.frame += 1;
    } else if ST.screen == Screen::GameOver {
        *PALETTE = PAL_GAMEOVER;
        let gamepad = *GAMEPAD1;
        // if gamepad != 0 {
        // }
    }

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

    // Render player
    let player_flags = match ST.dir {
        0 => ST.player_flags_last,
        1 => BLIT_1BPP | BLIT_FLIP_X,
        2 => BLIT_1BPP,
        _ => ST.player_flags_last,
    };
    ST.player_flags_last = player_flags;
    let player_frame = (ST.frame / 10) % 3;
    let mut player_sprite = match player_frame {
        0 => &SMILEY1,
        1 => &SMILEY2,
        2 => &SMILEY3,
        _ => &SMILEY1,
    };
    if ST.dir == 0 {
        player_sprite = &SMILEY1;
    }
    blit(
        player_sprite,
        ST.pos.x as i32,
        ST.pos.y as i32,
        8,
        PLAYER_SIZE as u32,
        player_flags,
    );

    // Render drill
    let drill_off = match ST.dir {
        0 => Pos::new(PLAYER_SIZE as i16, 0),
        1 => Pos::new(-(PLAYER_SIZE as i16), 0),
        2 => Pos::new(PLAYER_SIZE as i16, 0),
        3 => Pos::new(0, PLAYER_SIZE as i16),
        _ => Pos::new(PLAYER_SIZE as i16, 0),
    };
    let drill_flags = match ST.dir {
        0 => BLIT_1BPP,
        1 => BLIT_1BPP | BLIT_FLIP_X,
        2 => BLIT_1BPP,
        3 => BLIT_1BPP | BLIT_FLIP_Y | BLIT_FLIP_X | BLIT_ROTATE,
        _ => BLIT_1BPP,
    };
    let drill_show = match ST.dir {
        0 => false,
        1 => true,
        2 => true,
        3 => true,
        _ => false,
    };
    if drill_show && ST.is_drilling {
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
    let gold_frame = (ST.frame / 15) % 2;
    let gold_sprite = if gold_frame == 0 { &GOLD1 } else { &GOLD2 };
    for gold in &ST.gold_locs {
        *DRAW_COLORS = 3;
        // rect(gold.x as i32, gold.y as i32, 2, 2);
        blit(gold_sprite, gold.x as i32, gold.y as i32, 8, 4, BLIT_1BPP);
    }

    // Render exit
    let door_frame = (ST.frame / 20) % 2;
    let door_sprite = if door_frame == 0 { &DOOR1 } else { &DOOR2 };
    *DRAW_COLORS = 1;
    rect(ST.exit_loc.x as i32, ST.exit_loc.y as i32, 8, 8);
    *DRAW_COLORS = 3;
    blit(
        door_sprite,
        ST.exit_loc.x as i32,
        ST.exit_loc.y as i32,
        8,
        8,
        BLIT_1BPP,
    );

    // Render rain
    for rain in &ST.rain_locs {
        *DRAW_COLORS = 4;
        rect(rain.x as i32, rain.y as i32, 1, 1);
    }
    // Render drones
    let drone_frame = (ST.frame / 10) % 2;
    let drone_sprite = if drone_frame == 0 { &DRONE1 } else { &DRONE2 };
    for drone in &ST.drone_locs {
        *DRAW_COLORS = 4;
        // rect(drone.x as i32, drone.y as i32, 6, 4);
        blit(
            drone_sprite,
            drone.x as i32,
            drone.y as i32,
            8,
            8,
            BLIT_1BPP,
        );
    }

    // Render flies
    let fly_frame = (ST.frame / 10) % 2;
    let fly_sprite = if fly_frame == 0 { &FLY1 } else { &FLY2 };
    for fly in &ST.fly_locs {
        *DRAW_COLORS = 4;
        // rect(fly.x as i32, fly.y as i32, 4, 4);
        blit(fly_sprite, fly.x as i32, fly.y as i32, 8, 8, BLIT_1BPP);
    }

    // Render sliders
    let slider_frame = (ST.frame / 10) % 2;
    let slider_sprite = if slider_frame == 0 {
        &SLIDER1
    } else {
        &SLIDER2
    };
    for slider in &ST.slider_locs {
        *DRAW_COLORS = 4;
        // rect(slider.x as i32, slider.y as i32, 6, 4);
        blit(
            slider_sprite,
            slider.x as i32,
            slider.y as i32,
            8,
            8,
            BLIT_1BPP,
        );
    }

    // Highlight the top layer of blocks
    // TODO: This is probablly slow
    // Really only want to highlight visible to sky
    for x in 0..WORLD_SIZE {
        for y in 0..WORLD_SIZE {
            if let Some(cell) = ST.world_get(x, y) {
                if cell {
                    // Check if block above is empty
                    if y == 0 || ST.world_get(x, y - 1) == Some(false) {
                        *DRAW_COLORS = 3;
                        rect(x as i32, y as i32, 1, 1);
                    }
                }
            }
        }
    }

    // Help text
    if ST.has_drilled == false {
        *DRAW_COLORS = 1;
        rect(45, 45, 75, 16);
        *DRAW_COLORS = 4;
        let help_text = "X TO DIG";
        text(help_text, 50, 50);
    }

    // Health
    for i in 0..ST.hp {
        *DRAW_COLORS = 3;
        // rect(45 + i as i32 * 6, 4, 4, 4);
        blit(&HEART, 55 + i as i32 * 10, 2, 8, 8, BLIT_1BPP);
    }
    // Gold collected
    text(ST.gold.to_string(), 4, 2);

    // Start screen
    if ST.screen == Screen::Start {
        *DRAW_COLORS = 1;
        rect(0, 0, 160, 160);
        *DRAW_COLORS = 4;
        let title_text = "ACID\nRAIN";
        let instr_text = "PRESS\nX\nBUTTON";
        *DRAW_COLORS = 3;
        text(title_text, 45, 60);
        *DRAW_COLORS = 4;
        text(title_text, 45, 61);
        *DRAW_COLORS = 2;
        text(instr_text, 45, 80);
    }
    // Game over screen
    if ST.screen == Screen::GameOver {
        *DRAW_COLORS = 1;
        // rect(0, 0, 160, 160);
        *DRAW_COLORS = 4;
        let over_text = "GAME OVER";
        *DRAW_COLORS = 3;
        text(over_text, 45, 60);
        *DRAW_COLORS = 4;
        text(over_text, 46, 61);
        *DRAW_COLORS = 2;
        text(ST.gold.to_string(), 10, 80);
    }

    // Debug
    // text(ST.rain_locs.len().to_string(), 120, 2);
    // text(ST.hp.to_string(), 120, 8);
    // line(80, 0, ST.pos.x as i32 + 4, ST.pos.y as i32);
}
