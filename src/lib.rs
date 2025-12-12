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

static DEBUG: bool = false;
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
        let dx = self.x as i32 - other.x as i32;
        let dy = self.y as i32 - other.y as i32;
        ((dx * dx + dy * dy) as f32).sqrt()
    }
    fn clamp_to_world(&mut self) {
        if self.x < 0 {
            self.x = 0;
        }
        if self.x >= WORLD_SIZE as i16 {
            self.x = (WORLD_SIZE - 1) as i16;
        }
        if self.y < 0 {
            self.y = 0;
        }
        if self.y >= WORLD_SIZE as i16 {
            self.y = (WORLD_SIZE - 1) as i16;
        }
    }
}

#[derive(PartialEq)]
enum Screen {
    Start,
    Game,
    GameOver,
    Transition,
}

struct GameMaster {
    rng: Rng,
    seed: u64,
    frame: u32,
    hp: u8,
    pos: Pos,
    dir: u8,
    world: MiniBitVec,
    drill_speed: u8,
    exit_loc: Pos,
    gold_locs: Vec<Pos>,
    gold_rained: usize,
    gold: usize,
    // Dificulty
    drone_limit: usize,
    fly_limit: usize,
    slider_limit: usize,
    drone_rte: u16,      // Frames between drone spawns
    rain_chance_rte: u8, // Rate of increase of rain chance
    rain_amount_rte: u8, // Rate of increase of rain amount
    //
    rain_locs: Vec<Pos>,
    drone_locs: Vec<Pos>,
    fly_locs: Vec<Pos>,
    slider_locs: Vec<Pos>,
    player_flags_last: u32,
    dmg_frames: u8,
    no_input_frames: u8,
    has_drilled: bool,
    has_gold: bool,
    is_drilling: bool,
    screen: Screen,
}
static mut GM: LazyLock<GameMaster> = LazyLock::new(|| GameMaster {
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
    drill_speed: 64,
    exit_loc: Pos { x: 0, y: 0 },
    gold_locs: Vec::new(),
    gold_rained: 0,
    gold: 0,
    // Difficulty
    drone_limit: 5,
    fly_limit: 5,
    slider_limit: 5,
    drone_rte: 200,
    rain_chance_rte: 100,
    rain_amount_rte: 200,
    //
    rain_locs: Vec::new(),
    drone_locs: Vec::new(),
    fly_locs: Vec::new(),
    slider_locs: Vec::new(),
    player_flags_last: BLIT_1BPP,
    dmg_frames: 0,
    no_input_frames: 0,
    has_drilled: false,
    has_gold: false,
    is_drilling: false,
    screen: Screen::Transition,
});
impl GameMaster {
    unsafe fn input_check(&mut self, check: u8) -> bool {
        if self.no_input_frames > 0 {
            return false;
        }
        let gamepad = *GAMEPAD1;
        (gamepad & check) != 0
    }
    unsafe fn input_check_any(&mut self) -> bool {
        if self.no_input_frames > 0 {
            return false;
        }
        let gamepad = *GAMEPAD1;
        gamepad != 0
    }

    fn world_get(&self, x: usize, y: usize) -> Option<bool> {
        let index = y * WORLD_SIZE + x;
        self.world.get(index)
    }
    fn world_set(&mut self, x: usize, y: usize, value: bool) {
        let index = y * WORLD_SIZE + x;
        // Clamp to world size
        if index >= self.world.len() {
            return;
        }
        self.world.set(index, value);
    }
    fn world_set_area(&mut self, x: usize, y: usize, w: usize, h: usize, value: bool) {
        for dy in 0..h {
            for dx in 0..w {
                let wx = x + dx;
                let wy = y + dy;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    self.world_set(wx, wy, value);
                }
            }
        }
    }
    unsafe fn drill_area(&mut self, x: usize, y: usize, w: usize, h: usize, chance: u8) {
        for dy in 0..h {
            for dx in 0..w {
                let wx = x + dx;
                let wy = y + dy;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    // 128 = 100% chance
                    if self.rng.i32(0..128) < chance as i32 {
                        self.world_set(wx, wy, false);
                        self.sfx_dig();
                    }
                }
            }
        }
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
    // TODO: This is a little hacky
    // It should just use new and cache saved values
    unsafe fn reset(&mut self, full: bool) {
        // Block input for N frames
        self.no_input_frames = 120;
        self.screen = Screen::Transition;
        // Reset game state
        self.frame = 0;
        self.pos = Pos { x: 76, y: 0 };
        self.world = MiniBitVec::new();
        self.exit_loc = Pos { x: 0, y: 0 };
        self.gold_locs.clear();
        self.gold_rained = 0;
        self.rain_locs.clear();
        self.player_flags_last = BLIT_1BPP;
        self.dmg_frames = 0;
        self.has_gold = false;
        self.is_drilling = false;
        self.dir = 0;
        self.drone_locs.clear();
        self.fly_locs.clear();
        self.slider_locs.clear();
        if full {
            self.screen = Screen::Start;
            self.gold = 0;
            self.hp = 10;
            self.seed = 0;
            self.has_drilled = false;
        }
    }

    #[allow(static_mut_refs)]
    unsafe fn sfx_dig(&mut self) {
        let max = 440 - self.pos.y as u32 * 2; // 160
        let f = self.rng.u32(120..max);
        tone(f, 0, 50, TONE_NOISE);
    }

    #[allow(static_mut_refs)]
    unsafe fn sfx_rain(&mut self, p: &Pos) {
        let f = self.rng.u32(440..880);
        let dist = p.distance(&self.pos) as u32;
        let vol = 5 + (if dist > 50 { 20 } else { 50 - dist });
        tone(f, 0, vol, TONE_PULSE2);
    }

    #[allow(static_mut_refs)]
    unsafe fn sfx_gold(&mut self) {
        let f = self.rng.u32(400..440);
        tone(f, 4, 128, TONE_PULSE1);
    }

    #[allow(static_mut_refs)]
    unsafe fn sfx_dmg(&mut self) {
        let f = self.rng.u32(200..220);
        tone(f, 4, 128, TONE_PULSE1);
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn gen_world(&mut self) {
        self.world = MiniBitVec::new();
        for y in 0..WORLD_SIZE {
            for _ in 0..WORLD_SIZE {
                let mut alive = y >= 24;
                if self.rng.i32(0..100) < 2 {
                    alive = false;
                }
                self.world.push(alive);
            }
        }
        // Generate some random gold locations
        for _ in 0..GOLD_COUNT {
            let x = self.rng.i16(0..(WORLD_SIZE as i16));
            let y = self.rng.i16(24..(WORLD_SIZE as i16));
            self.gold_locs.push(Pos::new(x, y));
        }
        // Exit location
        let exit_x = self.rng.i16(0..(WORLD_SIZE as i16));
        self.exit_loc = Pos::new(exit_x, 152);
        // Fly locations
        for _ in 0..self.fly_limit {
            let x = self.rng.i16(0..(WORLD_SIZE as i16));
            let y = self.rng.i16(24..(WORLD_SIZE as i16));
            self.fly_locs.push(Pos::new(x, y));
        }
        // Slider locations
        for _ in 0..self.slider_limit {
            let x = self.rng.i16(0..(WORLD_SIZE as i16));
            let y = self.rng.i16(24..(WORLD_SIZE as i16));
            self.slider_locs.push(Pos::new(x, y));
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn input_main(&mut self) {
        let pos_cache = self.pos;
        self.is_drilling = false;
        self.dir = 0;
        if self.input_check(BUTTON_1) {
            self.is_drilling = true;
        }
        if self.input_check(BUTTON_LEFT) {
            self.pos.x -= 1;
            self.dir = 1;
        }
        if self.input_check(BUTTON_RIGHT) {
            self.pos.x += 1;
            self.dir = 2;
        }
        if self.is_drilling && self.input_check(BUTTON_DOWN) {
            self.has_drilled = true;
            self.dir = 3;
            // Remove the 4 blocks under the smiley
            self.drill_area(
                (self.pos.x - 1) as usize,
                (self.pos.y + PLAYER_SIZE as i16) as usize,
                (PLAYER_SIZE + 2) as usize,
                1,
                self.drill_speed,
            );
            // for dy in 0..1 {
            //     for dx in 0..(PLAYER_SIZE + 2) as i16 {
            //         let wx = (self.pos.x + dx - 1) as usize;
            //         let wy = (self.pos.y + dy + PLAYER_SIZE as i16) as usize;
            //         if wx < WORLD_SIZE && wy < WORLD_SIZE {
            //             if self.rng.i32(0..100) < 50 {
            //                 self.world_set(wx, wy, false);
            //                 self.sfx_dig();
            //             }
            //         }
            //     }
            // }
        }

        if self.is_drilling && self.input_check(BUTTON_RIGHT) {
            self.has_drilled = true;
            // Remove the 4 blocks to the right of the smiley
            self.drill_area(
                (self.pos.x + PLAYER_SIZE as i16 - 1) as usize,
                (self.pos.y - 1) as usize,
                1,
                (PLAYER_SIZE + 1) as usize,
                self.drill_speed,
            );
            // for dy in 0..(PLAYER_SIZE + 1) as i16 {
            //     for dx in 0..1 {
            //         let wx = (self.pos.x - 1 + dx + PLAYER_SIZE as i16) as usize;
            //         let wy = (self.pos.y + dy - 1) as usize;
            //         if wx < WORLD_SIZE && wy < WORLD_SIZE {
            //             if self.rng.i32(0..100) < 50 {
            //                 self.world_set(wx, wy, false);
            //                 self.sfx_dig();
            //             }
            //         }
            //     }
            // }
        }
        if self.is_drilling && self.input_check(BUTTON_LEFT) {
            self.has_drilled = true;
            // Remove the 4 blocks to the left of the smiley
            self.drill_area(
                (self.pos.x) as usize,
                (self.pos.y - 1) as usize,
                1,
                (PLAYER_SIZE + 1) as usize,
                self.drill_speed,
            );
            // for dy in 0..(PLAYER_SIZE + 1) as i16 {
            //     for dx in 0..1 {
            //         let wx = (self.pos.x - dx) as usize;
            //         let wy = (self.pos.y + dy - 1) as usize;
            //         if wx < WORLD_SIZE && wy < WORLD_SIZE {
            //             if self.rng.i32(0..100) < 50 {
            //                 self.world_set(wx, wy, false);
            //                 self.sfx_dig();
            //             }
            //         }
            //     }
            // }
        }
        self.player_collide(pos_cache);
        let pos_cache = self.pos;
        self.pos.y += 1;
        if self.pos.y > (WORLD_SIZE - PLAYER_SIZE as usize) as i16 {
            self.pos.y = (WORLD_SIZE - PLAYER_SIZE as usize) as i16;
        }
        self.player_collide(pos_cache);
        self.player_wrap();
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn player_collisions(&mut self) {
        // Check for gold collection
        self.gold_locs.retain(|gold| {
            let collected = GM.pos.x < gold.x + 4
                && GM.pos.x + PLAYER_SIZE as i16 > gold.x
                && GM.pos.y < gold.y + 4
                && GM.pos.y + PLAYER_SIZE as i16 > gold.y;
            if collected {
                GM.sfx_gold();
                GM.gold += 1;
                false
            } else {
                true
            }
        });
        // Check for collisions with doors
        let door_collide = self.pos.x < self.exit_loc.x + 8
            && self.pos.x + PLAYER_SIZE as i16 > self.exit_loc.x
            && self.pos.y < self.exit_loc.y + 8
            && self.pos.y + PLAYER_SIZE as i16 > self.exit_loc.y;
        if door_collide {
            // Win the game (reset for now)
            self.reset(false);
            return;
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn update_rain(&mut self) {
        // Add rain
        let mut rain_chance = self.frame / self.rain_chance_rte as u32;
        if rain_chance > 100 {
            rain_chance = 100;
        }
        let mut rain_amount = self.frame / self.rain_amount_rte as u32;
        if rain_amount > 10 {
            rain_amount = 10;
        }
        if self.rain_locs.len() < RAIN_MAX {
            for _ in 0..rain_amount {
                if self.rng.i32(0..100) < rain_chance as i32 {
                    let x = self.rng.i16(0..(WORLD_SIZE as i16));
                    self.rain_locs.push(Pos::new(x, 0));
                }
            }
        }
        // Update rain
        for rain in &mut self.rain_locs {
            rain.y += 1;
        }
        // Check for collision with player
        let mut hits_player: Vec<usize> = Vec::new();
        for (i, rain) in self.rain_locs.iter().enumerate() {
            for py in 0..PLAYER_SIZE as i16 {
                for px in 0..PLAYER_SIZE as i16 {
                    let px_pos = self.pos.x + px;
                    let py_pos = self.pos.y + py;
                    if px_pos == rain.x && py_pos == rain.y {
                        hits_player.push(i);
                    }
                }
            }
        }
        for &i in hits_player.iter().rev() {
            self.rain_locs.remove(i);
            self.dmg_frames = DMG_FRAMES;
            self.hp = self.hp.saturating_sub(1);
            self.sfx_dmg();
        }
        // Check for collision with world
        let mut hits_world: Vec<usize> = Vec::new();
        for (i, rain) in self.rain_locs.iter().enumerate() {
            let wx = rain.x as usize;
            let wy = rain.y as usize;
            if wx < WORLD_SIZE && wy < WORLD_SIZE {
                if let Some(cell) = self.world_get(wx, wy) {
                    if cell {
                        hits_world.push(i);
                    }
                }
            }
        }
        for &i in hits_world.iter().rev() {
            self.world_set(
                self.rain_locs[i].x as usize,
                self.rain_locs[i].y as usize,
                false,
            );
            let rain = self.rain_locs[i].clone();
            self.sfx_rain(&rain);
            self.rain_locs.remove(i);
        }
        // Check for collision with gold
        let mut hits_gold: Vec<usize> = Vec::new();
        for (i, rain) in self.rain_locs.iter().enumerate() {
            let wx = rain.x as usize;
            let wy = rain.y as usize;
            self.gold_locs.retain(|gold| {
                let hit = gold.x >= wx as i16
                    && gold.x < (wx + 4) as i16
                    && gold.y >= wy as i16
                    && gold.y < (wy + 4) as i16;
                if hit {
                    tone(280, 1, 100, TONE_PULSE2);
                    self.gold_rained += 1;
                    hits_gold.push(i);
                }
                !hit
            });
        }
        for &i in hits_gold.iter().rev() {
            self.gold_locs.retain(|gold| {
                let wx = self.rain_locs[i].x as usize;
                let wy = self.rain_locs[i].y as usize;
                let hit = gold.x >= wx as i16
                    && gold.x < (wx + 4) as i16
                    && gold.y >= wy as i16
                    && gold.y < (wy + 4) as i16;
                !hit
            });
            // Remove a chunk of world where gold was
            self.world_set_area(
                self.rain_locs[i].x as usize,
                self.rain_locs[i].y as usize,
                8,
                8,
                false,
            );
            self.rain_locs.remove(i);
        }
        // Check out of bounds rain
        self.rain_locs.retain(|rain| rain.y < WORLD_SIZE as i16);
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn update_drones(&mut self) {
        // Add drones
        if self.frame % self.drone_rte as u32 == 0 && self.drone_locs.len() < self.drone_limit {
            let x = self.rng.i16(0..(WORLD_SIZE as i16));
            self.drone_locs.push(Pos::new(x, 0));
        }
        // Update drones
        if self.frame % 16 != 0 {
            return;
        }
        // Move towards player
        let mut trigger_sfx = false;
        let mut clear_locs: Vec<Pos> = Vec::new();
        for drone in &mut self.drone_locs {
            let dx = self.pos.x - drone.x;
            let dy = self.pos.y - drone.y;
            let dist = self.pos.distance(drone);
            if dist > 1.0 {
                let step_x = (dx as f32 / dist).round() as i16;
                let step_y = (dy as f32 / dist).round() as i16;
                drone.x += step_x;
                drone.y += step_y;
                drone.clamp_to_world();
            }
            // Check for collision with player
            for py in 0..PLAYER_SIZE as i16 {
                for px in 0..PLAYER_SIZE as i16 {
                    let px_pos = self.pos.x + px;
                    let py_pos = self.pos.y + py;
                    if px_pos == drone.x && py_pos == drone.y {
                        trigger_sfx = true;
                    }
                }
            }
            clear_locs.push(Pos::new(drone.x, drone.y));
            // Remove world blocks at drone position
            // TODO: Cant use self.world_set here as it would borrow self mutably
            // This should be changed to use a list of positions to clear after the loop
            // for dy in 0..4 {
            //     for dx in 0..6 {
            //         let wx = (drone.x + dx) as usize;
            //         let wy = (drone.y + dy) as usize;
            //         if wx < WORLD_SIZE && wy < WORLD_SIZE {
            //             clear_locs.push(Pos::new(wx as i16, wy as i16));
            //         }
            //     }
            // }
        }
        if trigger_sfx {
            self.sfx_dmg();
        }
        // Clear world blocks
        for loc in clear_locs {
            self.world_set_area(loc.x as usize, loc.y as usize, 8, 4, false);
            // if loc.x >= 0 && loc.y >= 0 {
            //     let wx = loc.x as usize;
            //     let wy = loc.y as usize;
            //     if wx < WORLD_SIZE && wy < WORLD_SIZE {
            //         self.world_set(wx, wy, false);
            //     }
            // }
        }
    }
    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn update_flies(&mut self) {
        // Update fly location
        if self.frame % 8 != 0 {
            return;
        }
        // Move randomly
        for fly in &mut self.fly_locs {
            let dir = self.rng.i32(0..4);
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
            fly.clamp_to_world();
        }
        // Check for collision with player
        let mut hits_player: Vec<usize> = Vec::new();
        for (i, fly) in self.fly_locs.iter().enumerate() {
            for py in 0..PLAYER_SIZE as i16 {
                for px in 0..PLAYER_SIZE as i16 {
                    let px_pos = self.pos.x + px;
                    let py_pos = self.pos.y + py;
                    if px_pos == fly.x && py_pos == fly.y {
                        hits_player.push(i);
                    }
                }
            }
        }
        for &i in hits_player.iter().rev() {
            self.fly_locs.remove(i);
            self.dmg_frames = DMG_FRAMES;
            self.hp = self.hp.saturating_sub(1);
            self.sfx_dmg();
        }
        // Check for collision with world
        let mut hits_world: Vec<usize> = Vec::new();
        for (i, fly) in self.fly_locs.iter().enumerate() {
            let wx = fly.x as usize;
            let wy = fly.y as usize;
            if wx < WORLD_SIZE && wy < WORLD_SIZE {
                if let Some(cell) = self.world_get(wx, wy) {
                    if cell {
                        hits_world.push(i);
                    }
                }
            }
        }
        for &i in hits_world.iter().rev() {
            let fly = self.fly_locs[i].clone();
            self.world_set_area(fly.x as usize, fly.y as usize, 8, 4, false);
            // for dy in 0..4 {
            //     for dx in 0..8 {
            //         let wx = (fly.x + dx) as usize;
            //         let wy = (fly.y + dy) as usize;
            //         if wx < WORLD_SIZE && wy < WORLD_SIZE {
            //             self.world_set(wx, wy, false);
            //         }
            //     }
            // }
            self.sfx_rain(&fly);
            // self.fly_locs.remove(i);
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn update_sliders(&mut self) {
        // Sliders move left and right only
        if self.frame % 8 != 0 {
            return;
        }
        // Move randomly
        for slider in &mut self.slider_locs {
            let dir = self.rng.i32(0..2);
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
        }
        // Check for collision with player
        let mut hits_player: Vec<usize> = Vec::new();
        for (i, slider) in self.slider_locs.iter().enumerate() {
            for py in 0..PLAYER_SIZE as i16 {
                for px in 0..PLAYER_SIZE as i16 {
                    let px_pos = self.pos.x + px;
                    let py_pos = self.pos.y + py;
                    if px_pos == slider.x && py_pos == slider.y {
                        hits_player.push(i);
                    }
                }
            }
        }
        for &i in hits_player.iter().rev() {
            self.slider_locs.remove(i);
            self.dmg_frames = DMG_FRAMES;
            self.hp = self.hp.saturating_sub(1);
            self.sfx_dmg();
        }
        // Check for collision with world
        let mut hits_world: Vec<usize> = Vec::new();
        for (i, slider) in self.slider_locs.iter().enumerate() {
            let wx = slider.x as usize;
            let wy = slider.y as usize;
            if wx < WORLD_SIZE && wy < WORLD_SIZE {
                if let Some(cell) = self.world_get(wx, wy) {
                    if cell {
                        hits_world.push(i);
                    }
                }
            }
        }
        for &i in hits_world.iter().rev() {
            let slider = self.slider_locs[i].clone();
            self.world_set_area(slider.x as usize, slider.y as usize, 8, 4, false);
            self.sfx_rain(&slider);
            // self.slider_locs.remove(i);
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    // NOTE: This is a VERY expensive operation
    // We need to split the world update over multiple frames or we will run out of memeory
    // The larger the split size the faster the world updates
    // This effects the speed of falling blocks
    // WARN: `split_size` must evenly divide `WORLD_SIZE`
    unsafe fn update_world(&mut self) {
        // If world blocks have less than 4 neighbors, they fall down
        let split_size = 4;
        let falling_limit = 160; // Max number of blocks to fall per update
        let world_split = WORLD_SIZE / split_size;
        let world_offset = (self.frame % split_size as u32) as usize;
        let start_y = world_offset * world_split;
        let end_y = start_y + world_split;
        let mut to_fall = Vec::new();
        for y in (start_y..end_y).rev() {
            for x in 1..WORLD_SIZE - 1 {
                if let Some(cell) = self.world_get(x, y) {
                    // Only check alive cells
                    if cell {
                        let mut neighbors = 0;
                        for oy in -1..=1 {
                            for ox in -1..=1 {
                                if ox == 0 && oy == 0 {
                                    continue;
                                }
                                if let Some(ncell) = self
                                    .world_get((x as i32 + ox) as usize, (y as i32 + oy) as usize)
                                {
                                    if ncell {
                                        neighbors += 1;
                                    }
                                }
                            }
                        }
                        if neighbors < 4 && to_fall.len() < falling_limit {
                            to_fall.push((x, y));
                        }
                    }
                }
            }
        }
        for (x, y) in to_fall {
            // Check for collision below
            if let Some(below) = self.world_get(x, y + 1) {
                if !below {
                    // Check for player collision
                    let mut collide_with_player = false;
                    for dy in 0..PLAYER_SIZE as i16 {
                        for dx in 0..PLAYER_SIZE as i16 {
                            let px = self.pos.x + dx;
                            let py = self.pos.y + dy;
                            if px == x as i16 && py == (y as i16 + 1) {
                                collide_with_player = true;
                            }
                        }
                    }
                    if !collide_with_player {
                        self.world_set(x, y, false);
                        self.world_set(x, y + 1, true);
                    }
                }
            }
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn main_logic(&mut self) {
        self.input_main();
        self.player_collisions();

        self.update_rain();
        self.update_drones();
        self.update_flies();
        self.update_sliders();
        self.update_world();

        // Check for game over
        if self.hp == 0 {
            // self.screen = Screen::GameOver;
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn render(&mut self) {
        // If took damage, change palette briefly
        if GM.dmg_frames > 0 {
            *PALETTE = PAL_DMG;
            GM.dmg_frames -= 1;
        } else {
            *PALETTE = PAL;
        }
        // Render the world
        for y in 0..WORLD_SIZE {
            for x in 0..WORLD_SIZE {
                if let Some(cell) = GM.world_get(x, y) {
                    if cell {
                        *DRAW_COLORS = 2;
                        rect(x as i32, y as i32, 1, 1);
                    }
                }
            }
        }
        *DRAW_COLORS = 4;

        // Render player
        let player_flags = match GM.dir {
            0 => GM.player_flags_last,
            1 => BLIT_1BPP | BLIT_FLIP_X,
            2 => BLIT_1BPP,
            _ => GM.player_flags_last,
        };
        GM.player_flags_last = player_flags;
        let player_frame = (GM.frame / 10) % 3;
        let mut player_sprite = match player_frame {
            0 => &SMILEY1,
            1 => &SMILEY2,
            2 => &SMILEY3,
            _ => &SMILEY1,
        };
        if GM.dir == 0 {
            player_sprite = &SMILEY1;
        }
        blit(
            player_sprite,
            GM.pos.x as i32,
            GM.pos.y as i32,
            8,
            PLAYER_SIZE as u32,
            player_flags,
        );

        // Render drill
        let drill_off = match GM.dir {
            0 => Pos::new(PLAYER_SIZE as i16, 0),
            1 => Pos::new(-(PLAYER_SIZE as i16), 0),
            2 => Pos::new(PLAYER_SIZE as i16, 0),
            3 => Pos::new(0, PLAYER_SIZE as i16),
            _ => Pos::new(PLAYER_SIZE as i16, 0),
        };
        let drill_flags = match GM.dir {
            0 => BLIT_1BPP,
            1 => BLIT_1BPP | BLIT_FLIP_X,
            2 => BLIT_1BPP,
            3 => BLIT_1BPP | BLIT_FLIP_Y | BLIT_FLIP_X | BLIT_ROTATE,
            _ => BLIT_1BPP,
        };
        let drill_show = match GM.dir {
            0 => false,
            1 => true,
            2 => true,
            3 => true,
            _ => false,
        };
        if drill_show && GM.is_drilling {
            let drill_frame = (GM.frame / 5) % 2;
            let drill_sprite = if drill_frame == 0 { &DRILL } else { &DRILL2 };
            blit(
                drill_sprite,
                (GM.pos.x + drill_off.x) as i32,
                (GM.pos.y + drill_off.y) as i32,
                8,
                PLAYER_SIZE as u32,
                drill_flags,
            );
        }

        // Render gold locations
        let gold_frame = (GM.frame / 15) % 2;
        let gold_sprite = if gold_frame == 0 { &GOLD1 } else { &GOLD2 };
        for gold in &GM.gold_locs {
            *DRAW_COLORS = 3;
            // rect(gold.x as i32, gold.y as i32, 2, 2);
            blit(gold_sprite, gold.x as i32, gold.y as i32, 8, 4, BLIT_1BPP);
        }

        // Render exit
        let door_frame = (GM.frame / 20) % 2;
        let door_sprite = if door_frame == 0 { &DOOR1 } else { &DOOR2 };
        *DRAW_COLORS = 1;
        rect(GM.exit_loc.x as i32, GM.exit_loc.y as i32, 8, 8);
        *DRAW_COLORS = 3;
        blit(
            door_sprite,
            GM.exit_loc.x as i32,
            GM.exit_loc.y as i32,
            8,
            8,
            BLIT_1BPP,
        );

        // Render rain
        for rain in &GM.rain_locs {
            *DRAW_COLORS = 4;
            rect(rain.x as i32, rain.y as i32, 1, 1);
        }
        // Render drones
        let drone_frame = (GM.frame / 10) % 2;
        let drone_sprite = if drone_frame == 0 { &DRONE1 } else { &DRONE2 };
        for drone in &GM.drone_locs {
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
        let fly_frame = (GM.frame / 10) % 2;
        let fly_sprite = if fly_frame == 0 { &FLY1 } else { &FLY2 };
        for fly in &GM.fly_locs {
            *DRAW_COLORS = 4;
            // rect(fly.x as i32, fly.y as i32, 4, 4);
            blit(fly_sprite, fly.x as i32, fly.y as i32, 8, 8, BLIT_1BPP);
        }

        // Render sliders
        let slider_frame = (GM.frame / 10) % 2;
        let slider_sprite = if slider_frame == 0 {
            &SLIDER1
        } else {
            &SLIDER2
        };
        for slider in &GM.slider_locs {
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
                if let Some(cell) = GM.world_get(x, y) {
                    if cell {
                        // Check if block above is empty
                        if y == 0 || GM.world_get(x, y - 1) == Some(false) {
                            *DRAW_COLORS = 3;
                            rect(x as i32, y as i32, 1, 1);
                        }
                    }
                }
            }
        }

        // Help text
        if GM.has_drilled == false {
            *DRAW_COLORS = 1;
            rect(45, 45, 75, 16);
            *DRAW_COLORS = 4;
            let help_text = "X TO DIG";
            text(help_text, 50, 50);
        }

        // Health
        for i in 0..GM.hp {
            *DRAW_COLORS = 3;
            // rect(45 + i as i32 * 6, 4, 4, 4);
            blit(&HEART, 55 + i as i32 * 10, 2, 8, 8, BLIT_1BPP);
        }
        // Gold collected
        text(GM.gold.to_string(), 4, 2);

        // Start screen
        if GM.screen == Screen::Start {
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
        if GM.screen == Screen::GameOver {
            *DRAW_COLORS = 1;
            // rect(0, 0, 160, 160);
            *DRAW_COLORS = 4;
            let over_text = "GAME OVER";
            *DRAW_COLORS = 3;
            text(over_text, 45, 60);
            *DRAW_COLORS = 4;
            text(over_text, 46, 61);
            *DRAW_COLORS = 2;
            text(GM.gold.to_string(), 10, 80);
        }
        // Transition screen
        if GM.screen == Screen::Transition {
            *DRAW_COLORS = 1;
            rect(0, 0, 160, 160);
            *DRAW_COLORS = 4;
            rect(0, 0, 160, 30);
            *DRAW_COLORS = 1;
            text("LEVEL UP!", 45, 12);
            *DRAW_COLORS = 4;
            text("BUY UPGRADES:", 30, 40);
            *DRAW_COLORS = 3;
            text("HEART PIECE", 30, 60);
            text("DRILL SPEED", 30, 80);
        }

        // Debug
        if DEBUG {
            let dbg_string = format!(
                "FR:{}\nDR:{}\nFL:{}\nSL:{}\nRN:{}",
                self.frame,
                self.drone_locs.len(),
                self.fly_locs.len(),
                self.slider_locs.len(),
                self.rain_locs.len()
            );
            text(dbg_string.as_str(), 100, 120);
            let psize = std::mem::size_of::<Pos>();
            let mut size = 0;
            size += self.drone_locs.capacity() * psize;
            size += self.fly_locs.capacity() * psize;
            size += self.slider_locs.capacity() * psize;
            size += self.world.data.capacity();
            size += self.rain_locs.capacity() * psize;
            text(&format!("MEM: {} B", size), 4, 20);
            (80, 0, self.pos.x as i32 + 4, self.pos.y as i32);
        }
    }
}

#[no_mangle]
#[allow(static_mut_refs)]
unsafe fn start() {
    *PALETTE = PAL;
    GM.world = MiniBitVec::new();
}

#[no_mangle]
#[allow(static_mut_refs)]
unsafe fn update() {
    // UPDATE
    if GM.screen == Screen::Start {
        GM.seed += 1; // Increment seed while on start screen
        if GM.input_check_any() {
            GM.screen = Screen::Game;
            // Seed random with current frame
            GM.rng = Rng::with_seed(GM.seed);
            GM.gen_world();
        }
    } else if GM.screen == Screen::Game {
        GM.main_logic();
        GM.frame += 1;
    } else if GM.screen == Screen::GameOver {
        *PALETTE = PAL_GAMEOVER;
    } else if GM.screen == Screen::Transition {
        if GM.input_check_any() {
            GM.screen = Screen::Game;
            GM.gen_world();
        }
    }
    GM.no_input_frames = GM.no_input_frames.saturating_sub(1);

    // DRAW
    GM.render();
}
