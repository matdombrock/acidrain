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
    0b11111111,
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
const GOLDLRG1: [u8; 8] = [
    0b10000001,
    0b01111110,
    0b01001110,
    0b01001110,
    0b01111110,
    0b01111110,
    0b01111110,
    0b10000001,
];
#[rustfmt::skip]
const GOLDLRG2: [u8; 8] = [
    0b10000001,
    0b01111110,
    0b01111110,
    0b01111110,
    0b01110010,
    0b01110010,
    0b01111110,
    0b10000001,
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
const LOGO_A: [u8; 32] = [
    0b11110000, 0b00001111,
    0b11000000, 0b00000011,
    0b00000000, 0b00000000,
    0b00000011, 0b11000000,
    0b00000011, 0b11000000,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b00000011, 0b11000000,
    0b00000011, 0b11000000,
    0b00000011, 0b11000000,
    0b00000011, 0b11000000,
    0b00000011, 0b11000000,
    0b10000011, 0b11000001,
];
#[rustfmt::skip]
const LOGO_C: [u8; 32] = [
    0b11000000, 0b00000001,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000111, 0b11100000,
    0b10000111, 0b11111111,
    0b10000111, 0b11111111,
    0b10000111, 0b11100000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b11000000, 0b00000001,
];
#[rustfmt::skip]
const LOGO_I: [u8; 32] = [
    0b10000000, 0b00000001,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b10000000, 0b00000001,
    0b00000000, 0b00000000,
    0b10000000, 0b00000001,
    0b11111000, 0b00011111,
    0b11111000, 0b00011111,
    0b11111000, 0b00011111,
    0b11111000, 0b00011111,
    0b11111000, 0b00011111,
    0b10000000, 0b00000001,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b00000000, 0b00000000,
    0b10000000, 0b00000001,
];
#[rustfmt::skip]
const LOGO_D: [u8; 32] = [
    0b10000000, 0b00000001,
    0b11000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000001, 0b11111000,
    0b10000001, 0b11111000,
    0b10000001, 0b11111000,
    0b10000001, 0b11111000,
    0b10000001, 0b11111000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b11000000, 0b00000001,
];
#[rustfmt::skip]
const LOGO_R: [u8; 32] = [
    0b10000000, 0b00000011,
    0b01000000, 0b00000001,
    0b00000000, 0b00000001,
    0b00000000, 0b11100001,
    0b00000000, 0b11100001,
    0b00000000, 0b00000001,
    0b00000000, 0b00000001,
    0b00000000, 0b00000001,
    0b00000000, 0b00000001,
    0b00000000, 0b00000011,
    0b00000000, 0b00000001,
    0b00000000, 0b11000001,
    0b00000000, 0b11000001,
    0b00000000, 0b11000001,
    0b00000000, 0b11000001,
    0b10000000, 0b11000001,
];
#[rustfmt::skip]
const LOGO_N: [u8; 32] = [
    0b10000000, 0b11100001,
    0b01000000, 0b11100000,
    0b00000000, 0b11100000,
    0b00000000, 0b11100000,
    0b00000000, 0b11100000,
    0b00000001, 0b00100000,
    0b00000001, 0b00000000,
    0b00000001, 0b10000000,
    0b00000001, 0b10000000,
    0b00000001, 0b10000001,
    0b00000001, 0b10000000,
    0b00000001, 0b11000000,
    0b00000001, 0b11000000,
    0b00000001, 0b11000000,
    0b00000001, 0b11000000,
    0b10000001, 0b11000000,
];
static DEBUG: bool = false;
static WORLD_SIZE: usize = 160;
static PLAYER_SIZE: u8 = 8;
static RAIN_MAX: usize = 500;
static PAL: [u32; 4] = [0x001110, 0x506655, 0xD0FFDD, 0xEEFFE0];
static PAL_DMG: [u32; 4] = [0x221110, 0x506655, 0xD0FFDD, 0xEEFFE0];
static PAL_GAMEOVER: [u32; 4] = [0x221110, 0x506655, 0xD0FFDD, 0xEEFFE0];
static DMG_FRAMES: u8 = 16;
static NO_INPUT_FRAMES: u8 = 120;
static MAX_LVL: usize = 8;

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
    GameEnd,
    Shop,
    Transition,
}

#[derive(Copy, Clone)]
struct LVlSettings {
    drone_limit: usize,
    fly_limit: usize,
    slider_limit: usize,
    drone_rte: u16,
    rain_chance_rte: u16, // Higher is less chance
    rain_amount_rte: u16, // Higher is less amount
    gold_amt: usize,
}
impl LVlSettings {
    fn new() -> Self {
        Self {
            drone_limit: 0,
            fly_limit: 0,
            slider_limit: 0,
            drone_rte: 100,
            rain_chance_rte: 100,
            rain_amount_rte: 200,
            gold_amt: 10,
        }
    }
}

const LVLS: [LVlSettings; MAX_LVL] = [
    // Zero is not used
    LVlSettings {
        drone_limit: 0,
        fly_limit: 0,
        slider_limit: 0,
        drone_rte: 100,
        rain_chance_rte: 1000,
        rain_amount_rte: 1000,
        gold_amt: 8,
    },
    // This is the first real level
    LVlSettings {
        drone_limit: 0,
        fly_limit: 0,
        slider_limit: 0,
        drone_rte: 100,
        rain_chance_rte: 400,
        rain_amount_rte: 600,
        gold_amt: 8,
    },
    LVlSettings {
        drone_limit: 0,
        fly_limit: 3,
        slider_limit: 0,
        drone_rte: 250,
        rain_chance_rte: 300,
        rain_amount_rte: 300,
        gold_amt: 24,
    },
    LVlSettings {
        drone_limit: 0,
        fly_limit: 4,
        slider_limit: 3,
        drone_rte: 200,
        rain_chance_rte: 200,
        rain_amount_rte: 300,
        gold_amt: 32,
    },
    LVlSettings {
        drone_limit: 4,
        fly_limit: 5,
        slider_limit: 4,
        drone_rte: 150,
        rain_chance_rte: 100,
        rain_amount_rte: 140,
        gold_amt: 48,
    },
    LVlSettings {
        drone_limit: 5,
        fly_limit: 6,
        slider_limit: 5,
        drone_rte: 120,
        rain_chance_rte: 60,
        rain_amount_rte: 120,
        gold_amt: 64,
    },
    LVlSettings {
        drone_limit: 6,
        fly_limit: 7,
        slider_limit: 6,
        drone_rte: 100,
        rain_chance_rte: 50,
        rain_amount_rte: 100,
        gold_amt: 64,
    },
    LVlSettings {
        drone_limit: 7,
        fly_limit: 8,
        slider_limit: 7,
        drone_rte: 80,
        rain_chance_rte: 40,
        rain_amount_rte: 80,
        gold_amt: 64,
    },
];

struct GameMaster {
    rng: Rng,
    seed: u64,
    frame: u32,
    lvl: usize,
    hp: u8,
    pos: Pos,
    dir: u8,
    world: MiniBitVec,
    drill_speed: u8,
    exit_loc: Pos,
    gold_locs: Vec<Pos>,
    gold_rained: usize,
    gold: u16,
    drill_heat_max: u16,
    drill_heat: u16,
    drill_overheat: bool,
    diff: LVlSettings,
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
    cost_heart: u16,
    cost_drill_speed: u16,
    cost_drill_cool: u16,
    purchased: u8,
}
impl GameMaster {
    fn new() -> Self {
        Self {
            rng: Rng::new(),
            seed: 0,
            frame: 0,
            lvl: 1, // Start at 1
            hp: 8,
            pos: Pos { x: 48, y: 0 },
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
            drill_heat_max: 256,
            drill_heat: 0,
            drill_overheat: false,
            diff: LVlSettings::new(),
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
            screen: Screen::Start,
            cost_heart: 8,
            cost_drill_speed: 16,
            cost_drill_cool: 16,
            purchased: 0, // None, shop, drill speed, drill cool
        }
    }
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
                if wx < WORLD_SIZE - 1 && wy < WORLD_SIZE - 1 {
                    self.world_set(wx, wy, value);
                }
            }
        }
    }
    unsafe fn drill_area(&mut self, x: usize, y: usize, w: usize, h: usize, chance: u8) {
        // Prevent overflow and out-of-bounds
        if x.checked_add(w).map_or(true, |end_x| end_x > WORLD_SIZE)
            || y.checked_add(h).map_or(true, |end_y| end_y > WORLD_SIZE)
        {
            // Handle error or early return
            return;
        }

        for dy in 0..h {
            for dx in 0..w {
                let wx = x + dx;
                let wy = y + dy;
                if self.rng.i32(0..128) < chance as i32 {
                    self.world_set(wx, wy, false);
                    self.sfx_dig();
                }
            }
        }
    }
    unsafe fn player_collide_world(&mut self, cache: Pos) -> bool {
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

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn player_collide_misc(&mut self) {
        // Check for gold collection
        self.gold_locs.retain(|gold| {
            let collected = GM.pos.x < gold.x + 4
                && GM.pos.x + PLAYER_SIZE as i16 > gold.x
                && GM.pos.y < gold.y + 4
                && GM.pos.y + PLAYER_SIZE as i16 > gold.y;
            if collected {
                GM.sfx_gold();
                GM.drill_heat = GM.drill_heat.saturating_sub(GM.drill_heat_max / 10);
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
            self.screen = Screen::Shop;
            self.no_input_frames = NO_INPUT_FRAMES;
            return;
        }
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
    unsafe fn world_reset(&mut self, full: bool) {
        // Block input for N frames
        self.no_input_frames = 0;
        self.screen = Screen::Shop;
        // Reset game state
        self.frame = 0;
        self.pos = Pos { x: 48, y: 0 };
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
        self.drill_heat = 0;
        self.drill_overheat = false;
        if full {
            // Unused and not up to date
            self.screen = Screen::Start;
            self.gold = 0;
            self.hp = 10;
            self.seed = 0;
            self.has_drilled = false;
            self.drill_heat_max = 100;
        }
    }

    fn draw_gold(&mut self, x: i32, y: i32, amt: u16) {
        let frame = (self.frame / 16) % 2;
        if frame == 0 {
            blit(&GOLDLRG1, x, y, 8, 8, BLIT_1BPP);
        } else {
            blit(&GOLDLRG2, x, y, 8, 8, BLIT_1BPP);
        }
        text(&format!("{}", amt), x + 10, y);
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

    #[allow(static_mut_refs)]
    unsafe fn sfx_drill_overheat(&mut self) {
        tone(150, 60, 128, TONE_NOISE);
    }

    #[allow(static_mut_refs)]
    unsafe fn sfx_drill_warn(&mut self) {
        tone(840, 1, 128, TONE_TRIANGLE);
    }

    #[allow(static_mut_refs)]
    unsafe fn sfx_buy(&mut self) {
        tone(600, 2, 128, TONE_TRIANGLE);
    }

    #[allow(static_mut_refs)]
    unsafe fn sfx_deny(&mut self) {
        tone(400, 2, 128, TONE_TRIANGLE);
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
        for _ in 0..self.diff.gold_amt {
            let x = self.rng.i16(0..(WORLD_SIZE as i16));
            let y = self.rng.i16(24..(WORLD_SIZE as i16));
            self.gold_locs.push(Pos::new(x, y));
        }
        // Exit location
        let exit_x = self.rng.i16(0..(WORLD_SIZE as i16));
        self.exit_loc = Pos::new(exit_x, 152);
        self.world_set_area(
            (self.exit_loc.x - 4) as usize,
            (self.exit_loc.y - 2) as usize,
            16,
            12,
            false,
        );
        // Fly locations
        for _ in 0..self.diff.fly_limit {
            let x = self.rng.i16(0..(WORLD_SIZE as i16));
            let y = self.rng.i16(24..(WORLD_SIZE as i16));
            self.fly_locs.push(Pos::new(x, y));
        }
        // Slider locations
        for _ in 0..self.diff.slider_limit {
            let x = self.rng.i16(0..(WORLD_SIZE as i16));
            let y = self.rng.i16(24..(WORLD_SIZE as i16));
            self.slider_locs.push(Pos::new(x, y));
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn input_main(&mut self) {
        let pos_cache = self.pos;
        let mut drill_down = false;
        self.is_drilling = false;
        self.dir = 0;
        if self.input_check(BUTTON_1) {
            if !self.drill_overheat {
                drill_down = true;
            }
        }
        if self.input_check(BUTTON_LEFT) {
            self.pos.x -= 1;
            self.dir = 1;
        }
        if self.input_check(BUTTON_RIGHT) {
            self.pos.x += 1;
            self.dir = 2;
        }
        if drill_down && self.input_check(BUTTON_DOWN) {
            self.is_drilling = true;
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
        }

        if drill_down && self.input_check(BUTTON_RIGHT) {
            self.is_drilling = true;
            self.has_drilled = true;
            // Remove the 4 blocks to the right of the smiley
            self.drill_area(
                (self.pos.x + PLAYER_SIZE as i16 - 1) as usize,
                (self.pos.y - 1) as usize,
                1,
                (PLAYER_SIZE + 1) as usize,
                self.drill_speed,
            );
        }
        if drill_down && self.input_check(BUTTON_LEFT) {
            self.is_drilling = true;
            self.has_drilled = true;
            // Remove the 4 blocks to the left of the smiley
            self.drill_area(
                (self.pos.x) as usize,
                (self.pos.y - 1) as usize,
                1,
                (PLAYER_SIZE + 1) as usize,
                self.drill_speed,
            );
        }
        self.player_collide_world(pos_cache);
        let pos_cache = self.pos;
        self.pos.y += 1;
        if self.pos.y > (WORLD_SIZE - PLAYER_SIZE as usize) as i16 {
            self.pos.y = (WORLD_SIZE - PLAYER_SIZE as usize) as i16;
        }
        self.player_collide_world(pos_cache);
        self.player_wrap();
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn update_drill(&mut self) {
        if self.is_drilling {
            self.drill_heat = self.drill_heat.saturating_add(1);
        } else if self.drill_overheat {
            // Slower cooldown when overheated
            self.drill_heat = self.drill_heat.saturating_sub(1);
        } else {
            self.drill_heat = self.drill_heat.saturating_sub(2);
        }
        if self.drill_heat > (self.drill_heat_max as f32 * 0.75) as u16 {
            if self.frame % 8 == 0 {
                self.sfx_drill_warn();
            }
        }
        if self.drill_heat >= self.drill_heat_max {
            self.drill_overheat = true;
            self.sfx_drill_overheat();
        }
        // Release overheat when cooled down
        if self.drill_heat == 0 {
            self.drill_overheat = false;
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn update_rain(&mut self) {
        // Add rain
        let mut rain_chance = self.frame / self.diff.rain_chance_rte as u32;
        if rain_chance > 100 {
            rain_chance = 100;
        }
        let mut rain_amount = self.frame / self.diff.rain_amount_rte as u32;
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
            for (_, gold) in self.gold_locs.iter().enumerate() {
                let hit = gold.x >= wx as i16
                    && gold.x < (wx + 4) as i16
                    && gold.y >= wy as i16
                    && gold.y < (wy + 4) as i16;
                if hit {
                    hits_gold.push(i);
                    self.gold_rained += 1;
                }
            }
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
        if self.frame % self.diff.drone_rte as u32 == 0
            && self.drone_locs.len() < self.diff.drone_limit
        {
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
            let dir = self.rng.i32(0..6);
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
                _ => {
                    // Flies trend up
                    // So use the last two cases to go up
                    fly.y -= 1;
                    if fly.y < 0 {
                        fly.y = (WORLD_SIZE - 1) as i16;
                    }
                }
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
        self.player_collide_misc();

        self.update_drill();

        self.update_rain();
        self.update_drones();
        self.update_flies();
        self.update_sliders();
        self.update_world();

        // Check for game over
        if self.hp == 0 {
            self.screen = Screen::GameOver;
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn shop_logic(&mut self) {
        fn cont(gm: &mut GameMaster) {
            gm.lvl += 1;
            if gm.lvl >= MAX_LVL {
                gm.lvl = 0;
                gm.screen = Screen::GameEnd;
            }
            gm.no_input_frames = NO_INPUT_FRAMES;
            gm.screen = Screen::Transition;
        }
        if self.purchased > 0 && self.input_check_any() {
            self.purchased = 0;
            self.no_input_frames = NO_INPUT_FRAMES;
        }
        if self.input_check(BUTTON_UP) {
            // Buy heart piece
            self.no_input_frames = NO_INPUT_FRAMES;
            if self.gold >= self.cost_heart && self.hp < 8 {
                self.gold = self.gold.saturating_sub(self.cost_heart);
                self.hp += 1;
                self.purchased = 1;
                self.sfx_buy();
            } else {
                self.sfx_deny();
                self.dmg_frames = DMG_FRAMES;
            }
        } else if self.input_check(BUTTON_LEFT) {
            // Buy drill speed
            self.no_input_frames = NO_INPUT_FRAMES;
            if self.gold >= self.cost_drill_speed && self.drill_speed < 128 {
                self.gold = self.gold.saturating_sub(self.cost_drill_speed);
                self.drill_speed += 8;
                self.purchased = 2;
                self.sfx_buy();
            } else {
                self.sfx_deny();
                self.dmg_frames = DMG_FRAMES;
            }
        } else if self.input_check(BUTTON_RIGHT) {
            // Buy drill cooling
            self.no_input_frames = NO_INPUT_FRAMES;
            if self.gold >= self.cost_drill_cool && self.drill_heat_max < 1024 {
                self.gold = self.gold.saturating_sub(self.cost_drill_cool);
                self.drill_heat_max += 64;
                self.purchased = 3;
                self.sfx_buy();
            } else {
                self.sfx_deny();
                self.dmg_frames = DMG_FRAMES;
            }
        } else if self.input_check(BUTTON_DOWN) {
            cont(self);
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn render_start(&mut self) {
        if GM.screen == Screen::Start {
            *DRAW_COLORS = 1;
            rect(0, 0, 160, 160);
            for x in 0..5 as i32 {
                for y in 0..8 {
                    *DRAW_COLORS = (x as u32 + y as u32 * self.frame / 100) as u16 % 3 + 0;
                    text("ACID", x * 32, y * 20);
                    *DRAW_COLORS = (x as u32 + (y as u32 * 3) * self.frame / 128) as u16 % 3 + 0;
                    text("RAIN", x * 32, 10 + y * 20);
                }
            }
            *DRAW_COLORS = 1;
            for x in 0..5 as i32 {
                for y in 0..8 {
                    text("ACID", 1 + x * 32, 1 + y * 20);
                    text("RAIN", 1 + x * 32, 1 + 10 + y * 20);
                }
            }
            if self.frame % 512 > 20 {
                *DRAW_COLORS = 1;
                for y in 0..80 {
                    hline(0, y * 2, 160);
                }
            }
            for i in 0..160 {
                let sina = (self.frame as f32 / 320.).sin() * 2.0;
                let sin = ((self.frame as f32 / 10.) + (i as f32 / (4. + sina))).sin();
                let y = (sin * 8.0 + 140.0) as i32;
                *DRAW_COLORS = 2;
                rect(i as i32, y, 1, (160 - y) as u32);
            }
            *DRAW_COLORS = 2;
            text("MATHIEU/\nDOMBROCK\n2025////", 12, 50);
            *DRAW_COLORS = 3;
            if (self.frame / 30) % 2 == 0 {
                *DRAW_COLORS = 4;
            }
            text(b"PRESS \x80 TO START", 16, 105);
            *DRAW_COLORS = 2;
            let x = 10;
            let y = 10;
            blit(&LOGO_A, x, y, 16, 16, BLIT_1BPP);
            blit(&LOGO_C, x + 16 * 1, y, 16, 16, BLIT_1BPP);
            blit(&LOGO_I, x + 16 * 2, y, 16, 16, BLIT_1BPP);
            blit(&LOGO_D, x + 16 * 3, y, 16, 16, BLIT_1BPP);
            blit(&LOGO_R, x, y + 18, 16, 16, BLIT_1BPP);
            blit(&LOGO_A, x + 16 * 1, y + 18, 16, 16, BLIT_1BPP);
            blit(&LOGO_I, x + 16 * 2, y + 18, 16, 16, BLIT_1BPP);
            blit(&LOGO_N, x + 16 * 3, y + 18, 16, 16, BLIT_1BPP);
            let x = 12;
            let y = 12;
            *DRAW_COLORS = 4;
            blit(&LOGO_A, x, y, 16, 16, BLIT_1BPP);
            blit(&LOGO_C, x + 16 * 1, y, 16, 16, BLIT_1BPP);
            blit(&LOGO_I, x + 16 * 2, y, 16, 16, BLIT_1BPP);
            blit(&LOGO_D, x + 16 * 3, y, 16, 16, BLIT_1BPP);
            blit(&LOGO_R, x, y + 18, 16, 16, BLIT_1BPP);
            blit(&LOGO_A, x + 16 * 1, y + 18, 16, 16, BLIT_1BPP);
            blit(&LOGO_I, x + 16 * 2, y + 18, 16, 16, BLIT_1BPP);
            blit(&LOGO_N, x + 16 * 3, y + 18, 16, 16, BLIT_1BPP);
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn render_shop(&mut self) {
        if GM.screen == Screen::Shop {
            *DRAW_COLORS = 1;
            rect(0, 0, 160, 160);
            *DRAW_COLORS = 3;
            rect(0, 0, 160, 80);
            *DRAW_COLORS = 1;
            for x in 0..160 {
                let sina = (GM.frame as f32 / 320.).sin() * 2.0;
                let sin = ((GM.frame as f32 / 10.) + (x as f32 / (4. + sina))).sin();
                let y = (sin * 8.0 + 32.0) as i32;
                rect(x as i32, y, 1, (160 - y) as u32);
            }
            *DRAW_COLORS = 1;
            let sy = (self.frame as f32 / 8.).sin() * 2.0;
            text("UPGRADES!", 50, 6 + sy as i32);
            self.draw_gold(50, 14 + sy as i32, self.gold);
            *DRAW_COLORS = 3;
            vline(115, 45, 80);
            // Up
            text(b"\x86HEART PIECE", 12, 50);
            self.draw_gold(120, 50, self.cost_heart);
            text(format!("{}/8", self.hp), 20, 60);
            // Left
            text(b"\x84DRILL SPEED", 12, 80);
            self.draw_gold(120, 80, self.cost_drill_speed);
            text(format!("{}/128", self.drill_speed), 20, 90);
            // Righ
            text(b"\x85DRILL COOLR", 12, 110);
            self.draw_gold(120, 110, self.cost_drill_cool);
            text(format!("{}/1024", self.drill_heat_max), 20, 120);
            // Down
            *DRAW_COLORS = 4;
            hline(0, 135, 160);
            text(b"\x87NEXT  LEVEL", 30, 145);

            // Purchased
            if self.purchased > 0 {
                *DRAW_COLORS = 1;
                rect(0, 45, 160, 120);
                *DRAW_COLORS = 4;
                text("PURCHASED!", 12, 60);
                match self.purchased {
                    1 => {
                        text("HEART PIECE", 12, 70);
                        text(format!("{}/{}", self.hp, 8), 12, 80);
                    }
                    2 => {
                        text("DRILL SPEED", 12, 70);
                        text(format!("{}/{}", self.drill_speed, 128), 12, 80);
                    }
                    3 => {
                        text("DRILL COOLR", 12, 70);
                        text(format!("{}/{}", self.drill_heat_max, 1024), 12, 80);
                    }
                    _ => {}
                }
                *DRAW_COLORS = 3;
                text(b"\x80TO CONTINUE", 12, 110);
            }
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn render_transition(&mut self) {
        if GM.screen == Screen::Transition {
            *DRAW_COLORS = 1;
            rect(0, 0, 160, 160);
            *DRAW_COLORS = 4;
            let trans_text = format!("LEVEL {}", self.lvl);
            text(trans_text, 50, 60);
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn render_gameover(&mut self) {
        if GM.screen == Screen::GameOver {
            *PALETTE = PAL_GAMEOVER;
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
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn render_gameend(&mut self) {
        if GM.screen == Screen::GameEnd {
            *PALETTE = PAL_GAMEOVER;
            *DRAW_COLORS = 1;
            // rect(0, 0, 160, 160);
            *DRAW_COLORS = 4;
            let over_text = "GAME END";
            *DRAW_COLORS = 3;
            text(over_text, 45, 60);
            *DRAW_COLORS = 4;
            text(over_text, 46, 61);
            *DRAW_COLORS = 2;
            text(GM.gold.to_string(), 10, 80);
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn draw_no_input(&mut self) {
        if GM.no_input_frames > 0 {
            *DRAW_COLORS = 2;
            rect(0, 0, 160, 2);
            rect(0, 158, 160, 2);
            rect(0, 0, 2, 160);
            rect(158, 0, 2, 160);
            GM.no_input_frames -= 1;
        }
    }

    #[no_mangle]
    #[allow(static_mut_refs)]
    unsafe fn render(&mut self) {
        // Always run palette change first
        // If took damage, change palette briefly
        if GM.dmg_frames > 0 {
            *PALETTE = PAL_DMG;
            GM.dmg_frames -= 1;
        } else {
            *PALETTE = PAL;
        }

        if GM.screen != Screen::Game {
            return;
        }

        // Health
        for i in 0..GM.hp {
            *DRAW_COLORS = 3;
            // rect(45 + i as i32 * 6, 4, 4, 4);
            blit(&HEART, 76 + i as i32 * 10, 2, 8, 8, BLIT_1BPP);
        }
        // Gold collected
        // text(GM.gold.to_string(), 4, 2);
        self.draw_gold(4, 2, GM.gold);

        // Heat bar
        *DRAW_COLORS = 2;
        let heat_bar_width = 80;
        let heat_width = (GM.drill_heat as u32 * heat_bar_width) / (GM.drill_heat_max as u32);
        rect(76, 12, heat_bar_width, 4);
        *DRAW_COLORS = 3;
        if GM.drill_overheat {
            let c = (GM.frame / 10) % 2;
            *DRAW_COLORS = (c + 3) as u16;
        }
        rect(76, 12, heat_width, 4);
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
        if GM.has_drilled == false && GM.lvl == 1 {
            *DRAW_COLORS = 1;
            rect(50, 50, 60, 24);
            *DRAW_COLORS = 4;
            let help_text = b"\x84\x85\x87+\x80\nTO DIG";
            text(help_text, 57, 53);
        }
    }
}

static mut GM: LazyLock<GameMaster> = LazyLock::new(|| GameMaster::new());

#[no_mangle]
#[allow(static_mut_refs)]
unsafe fn start() {
    *PALETTE = PAL;
    GM.world = MiniBitVec::new();
}

#[no_mangle]
#[allow(static_mut_refs)]
unsafe fn update() {
    // TODO: Frame inc can prob happen everywhere
    // UPDATE
    if GM.screen == Screen::Start {
        GM.seed += 1; // Increment seed while on start screen
        if GM.input_check_any() {
            // Seed random with current frame
            GM.rng = Rng::with_seed(GM.seed);
            GM.no_input_frames = NO_INPUT_FRAMES;
            GM.screen = Screen::Transition;
        }
        GM.frame += 1;
    } else if GM.screen == Screen::Game {
        GM.main_logic();
        GM.frame += 1;
    } else if GM.screen == Screen::GameOver {
        // *PALETTE = PAL_GAMEOVER;
    } else if GM.screen == Screen::Shop {
        GM.shop_logic();
        GM.frame += 1;
    } else if GM.screen == Screen::Transition {
        if GM.input_check_any() {
            GM.world_reset(false);
            GM.diff = LVLS[GM.lvl];
            GM.gen_world();
            GM.screen = Screen::Game;
        }
    }
    GM.no_input_frames = GM.no_input_frames.saturating_sub(1);

    // DRAW
    GM.render();
    // NOTE: Other screens only render if active
    // Start screen
    GM.render_start();
    // Game over screen
    GM.render_gameover();
    // Shop screen
    GM.render_shop();
    // Transition screen
    GM.render_transition();
    // Game end screen
    GM.render_gameend();
    // No input overlay
    GM.draw_no_input();
    // Debug
    if DEBUG {
        let dbg_string = format!(
            "FR:{}\nDR:{}\nFL:{}\nSL:{}\nRN:{}",
            GM.frame,
            GM.drone_locs.len(),
            GM.fly_locs.len(),
            GM.slider_locs.len(),
            GM.rain_locs.len()
        );
        text(dbg_string.as_str(), 100, 120);
        let psize = std::mem::size_of::<Pos>();
        let mut size = 0;
        size += GM.drone_locs.capacity() * psize;
        size += GM.fly_locs.capacity() * psize;
        size += GM.slider_locs.capacity() * psize;
        size += GM.world.data.capacity();
        size += GM.rain_locs.capacity() * psize;
        text(&format!("MEM: {} B", size), 4, 20);
        (80, 0, GM.pos.x as i32 + 4, GM.pos.y as i32);
    }
}
