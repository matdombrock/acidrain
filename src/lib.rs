/*
* ACIDRAIN
* Mathieu Dombrock 2025
* GPLv3 License
*
* This code is not super well written.
* There are many things that could be optimized or cleaned up.
* There are abstractions that should exist but don't.
* There are abstractions that do exist but probably shouldn't.
* Loops through ent lists are inconsistent (sometimes index based, sometimes iterator based).
* There may be memory leaks though I've put a lot of work into avoiding them.
* A lot of this is unsafe code due to the nature of WASM4.
*
* The types used on some variables are not ideal. For example, a lot of things use `usize` which is
* pretty large for this platform. We could instead store u8 or u16 in many places and cast to usize
* as needed.
*
* A major optimization that could be done is to use a more efficient data structure for storing
* world positions. Positions are currently stored as Vec<Pos>. However, the world is stored as a
* MiniBitVec. This means that we could store positions as indices into
* the MiniBitVec, which would save a lot of memory and potentially speed up lookups.
*
* That being said, it works, and should be pretty bug free.
*
*/

use std::sync::LazyLock;
#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
use fastrand::Rng;

use wasm4::*;

static GRID: bool = false;
static WORLD_SIZE: usize = 160;
static PLAYER_SIZE: u8 = 8;
static RAIN_MAX: usize = 500;
static DMG_FRAMES: u8 = 16;
static NO_INPUT_FRAMES: u8 = 120;
static NO_INPUT_FRAMES_SH: u8 = 48;
static POWERUP_FRAMES: u16 = 600;
static MAX_LVL: usize = 8;
static MAX_DIFF: u8 = 8;
static MAX_HP: u8 = 8;
static DOOR_TIMER: u16 = 128;
static DIRT_START: u8 = 24;
static MUSIC_ENABLED: bool = true;
static INVINCIBLE: bool = false;

// Color palettes
static PAL_OG: [u32; 4] = [0x001105, 0x506655, 0xA0FFA5, 0xB0FFB5]; // OG
static PAL_CLREV: [u32; 4] = [0xd0d058, 0xa0a840, 0x708028, 0x405010]; // Classic Rev
static PAL_CL: [u32; 4] = [0x405010, 0x708028, 0xa0a840, 0xd0d058]; // Classic
static PAL_MOON: [u32; 4] = [0x191b1a, 0x294257, 0xa0a840, 0xd0d058]; // Moonlight
static PAL_MOLD: [u32; 4] = [0x191b1a, 0x294257, 0x579c9a, 0x99c9b3]; // BlueMold
static PAL_NYMPH: [u32; 4] = [0x2c2137, 0x446176, 0x3fac95, 0xa1ef8c]; // Nymph
static PAL_SWAMP: [u32; 4] = [0x3b252e, 0x593a5f, 0x4d7d65, 0xd1ada1]; // Forgotten Swamp
static PAL_STAR: [u32; 4] = [0x674577, 0x64b9ca, 0xffa3d6, 0xffebe5]; // Star pop
static PAL_GBN: [u32; 4] = [0x060601, 0x0b3e08, 0x489a0d, 0xdaf222]; // GB Night
static PAL_ML: [u32; 4] = [0x211f1f, 0x372c38, 0x7a7272, 0xababab]; // Mono logo
static PAL_TECH: [u32; 4] = [0x1d2938, 0x2a616e, 0x13b37e, 0x07ef5c]; // Technobike
static PAL_DMG: [u32; 4] = [0x221111, 0x551111, 0xDD1111, 0xFF1111];
static PALS: [[u32; 4]; 11] = [
    PAL_GBN, PAL_NYMPH, PAL_MOON, PAL_STAR, PAL_MOLD, PAL_SWAMP, PAL_ML, PAL_TECH, PAL_OG, PAL_CL,
    PAL_CLREV,
];

static INTRO_TEXT: &str = r#"
Its getting harder
out here every day.

Every dig, the rain
comes stronger, 
faster, 
and with more bite.

I don't know how 
much longer they 
expect us to keep
this up.

I was just happy
to retire with some
skin left...

I wanna know kid,
have you ever seen
the rain?

- ACIDMINER #1336
"#;

// Sprite data
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
const SMILEYDEAD: [u8; 8] = [
    0b11111111,
    0b11110011,
    0b10100000,
    0b00110100,
    0b00000101,
    0b00110101,
    0b00100000,
    0b10000000,
];

#[rustfmt::skip]
const DRILL1: [u8; 8] = [
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
const DRILLD1: [u8; 8] = [
    0b01011111,
    0b10011111,
    0b01001111,
    0b11001111,
    0b11111111,
    0b11111111,
    0b11111111,
    0b11111111,
];
#[rustfmt::skip]
const DRILLD2: [u8; 8] = [
    0b01011111,
    0b10111111,
    0b00001111,
    0b11001111,
    0b11111111,
    0b11111111,
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
const SEEKER1: [u8; 8] = [
    0b00100100,
    0b01000010,
    0b10000001,
    0b11011011,
    0b11000011,
    0b10000001,
    0b01000010,
    0b00100100,
];
#[rustfmt::skip]
const SEEKER2: [u8; 8] = [
    0b10100101,
    0b00000000,
    0b10000001,
    0b11011011,
    0b11000011,
    0b10000001,
    0b00000000,
    0b10100101,
];
#[rustfmt::skip]
const BOMBER1: [u8; 8] = [
    0b11111111,
    0b11101111,
    0b11110111,
    0b11100111,
    0b10000001,
    0b01100110,
    0b01100110,
    0b10000001,
];
#[rustfmt::skip]
const BOMBER2: [u8; 8] = [
    0b11011111,
    0b11110111,
    0b11101111,
    0b11100111,
    0b10000001,
    0b01100110,
    0b00000000,
    0b10000001,
];
#[rustfmt::skip]
const PU1: [u8; 8] = [
    0b10000001,
    0b01111110,
    0b01111110,
    0b01100110,
    0b00000000,
    0b01111110,
    0b01111110,
    0b00000000,
];
#[rustfmt::skip]
const PU2: [u8; 8] = [
    0b10000001,
    0b01111110,
    0b01111110,
    0b01100110,
    0b00000000,
    0b01111110,
    0b01111110,
    0b00000000,
];

#[rustfmt::skip]
const EXC: [u8; 8] = [
    0b11000011,
    0b11000011,
    0b11000011,
    0b11100111,
    0b11100111,
    0b11111111,
    0b11100111,
    0b11100111,
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
    0b00010011, 0b11000100,
    0b10010011, 0b11010101,
];
#[rustfmt::skip]
const LOGO_C: [u8; 32] = [
    0b11000000, 0b00000001,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b00000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000011, 0b11110000,
    0b10000011, 0b11110000,
    0b00000011, 0b11111111,
    0b10000011, 0b11111111,
    0b10000011, 0b11110000,
    0b10000011, 0b11110000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000000, 0b00000000,
    0b00000000, 0b00001000,
    0b11001000, 0b10011001,
];
#[rustfmt::skip]
const LOGO_I: [u8; 32] = [
    0b11000000, 0b00000011,
    0b10000000, 0b00000001,
    0b10000000, 0b00000000,
    0b00000000, 0b00000001,
    0b00000000, 0b00000000,
    0b10000000, 0b00000001,
    0b11111000, 0b00011111,
    0b11111000, 0b00011111,
    0b11111000, 0b00011111,
    0b11111000, 0b00011111,
    0b11111000, 0b00011111,
    0b10000000, 0b00000001,
    0b00000000, 0b00000000,
    0b00000000, 0b00000001,
    0b10000001, 0b00001000,
    0b11010011, 0b01001011,
];
#[rustfmt::skip]
const LOGO_D: [u8; 32] = [
    0b10000000, 0b00000001,
    0b11000000, 0b00000000,
    0b00000000, 0b00000000,
    0b10000000, 0b00000000,
    0b00000000, 0b00000000,
    0b10000000, 0b00000000,
    0b10000001, 0b11111000,
    0b10000001, 0b11111000,
    0b10000001, 0b11111000,
    0b10000001, 0b11111000,
    0b10000001, 0b11111000,
    0b10000000, 0b00000000,
    0b00000000, 0b00000000,
    0b10000000, 0b00000000,
    0b00000000, 0b10000000,
    0b11001000, 0b11001001,
];
#[rustfmt::skip]
const LOGO_R: [u8; 32] = [
    0b10000000, 0b00000011,
    0b01000000, 0b00000001,
    0b00000000, 0b00000001,
    0b00000000, 0b11110001,
    0b00000000, 0b11110000,
    0b00000000, 0b00000001,
    0b00000000, 0b00000001,
    0b00000000, 0b00000000,
    0b00000000, 0b00000001,
    0b00000000, 0b00000011,
    0b00000000, 0b00000001,
    0b00000000, 0b11000001,
    0b00000000, 0b11000000,
    0b00000000, 0b11000001,
    0b01001000, 0b11000100,
    0b10001100, 0b11000101,
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
    0b00001001, 0b11010000,
    0b10101001, 0b11010010,
];

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
        let byte_index = index / 8;
        if byte_index >= self.data.len() {
            return None;
        }
        let bit_index = index % 8;
        let byte = self.data[byte_index];
        Some((byte & (1 << bit_index)) != 0)
    }

    pub fn set(&mut self, index: usize, value: bool) {
        let byte_index = index / 8;
        if byte_index >= self.data.len() {
            return;
        }
        let bit_index = index % 8;
        if value {
            self.data[byte_index] |= 1 << bit_index;
        } else {
            self.data[byte_index] &= !(1 << bit_index);
        }
    }
    pub fn len_bits(&self) -> usize {
        self.len * 8
    }
}

#[derive(Copy, Clone, PartialEq)]
// NOTE: Cant use unsigned because we need negative
// Cant use i8 because 160x160
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
    Intro,
    Start,
    Game,
    GameOver,
    Shop,
    Transition,
}

#[derive(Copy, Clone)]
struct LVlSettings {
    drone_limit: usize,
    fly_limit: usize,
    slider_limit: usize,
    seeker_limit: usize,
    bomber_limit: usize,
    drone_rte: u16,
    rain_chance_rte: u16, // Higher is less chance
    rain_amount_rte: u16, // Higher is less amount
    rain_acidity: u8,
    gold_amt: usize,
    text: &'static [u8],
}
impl LVlSettings {
    fn new() -> Self {
        Self {
            drone_limit: 0,
            fly_limit: 0,
            slider_limit: 0,
            seeker_limit: 0,
            bomber_limit: 0,
            drone_rte: 100,
            rain_chance_rte: 100,
            rain_amount_rte: 200,
            rain_acidity: 50,
            gold_amt: 10,
            text: b"",
        }
    }
    fn apply_difficulty(&mut self, difficulty: u8) {
        self.drone_limit *= difficulty as usize;
        self.fly_limit *= difficulty as usize;
        self.slider_limit *= difficulty as usize;
        self.drone_rte = self.drone_rte.saturating_div(difficulty as u16);
        self.rain_chance_rte = self.rain_chance_rte.saturating_div(difficulty as u16);
        self.rain_amount_rte = self.rain_amount_rte.saturating_div(difficulty as u16);
    }
}

const LVLS: [LVlSettings; MAX_LVL] = [
    // Zero is not used
    // NOTE: IDK why I built it like this
    LVlSettings {
        drone_limit: 0,
        fly_limit: 0,
        slider_limit: 0,
        seeker_limit: 0,
        bomber_limit: 0,
        drone_rte: 100,
        rain_chance_rte: 1000,
        rain_amount_rte: 1000,
        rain_acidity: 0,
        gold_amt: 8,
        text: b"\x84\x87\x85a",
    },
    // This is the first real level
    LVlSettings {
        drone_limit: 0,
        fly_limit: 2,
        slider_limit: 0,
        seeker_limit: 0,
        bomber_limit: 2,
        drone_rte: 100,
        rain_chance_rte: 600,
        rain_amount_rte: 600,
        rain_acidity: 0,
        gold_amt: 8,
        text: b"\x84\x87\x85
MOVE
\x84\x87\x86\x85+\x80
DRILL",
    },
    LVlSettings {
        drone_limit: 0,
        fly_limit: 3,
        slider_limit: 0,
        seeker_limit: 0,
        bomber_limit: 3,
        drone_rte: 250,
        rain_chance_rte: 300,
        rain_amount_rte: 300,
        rain_acidity: 5,
        gold_amt: 24,
        text: b"Its's all
down from
here...",
    },
    LVlSettings {
        drone_limit: 0,
        fly_limit: 4,
        slider_limit: 3,
        seeker_limit: 0,
        bomber_limit: 3,
        drone_rte: 200,
        rain_chance_rte: 200,
        rain_amount_rte: 300,
        rain_acidity: 10,
        gold_amt: 32,
        text: b"THE END?",
    },
    LVlSettings {
        drone_limit: 4,
        fly_limit: 5,
        slider_limit: 4,
        seeker_limit: 0,
        bomber_limit: 2,
        drone_rte: 150,
        rain_chance_rte: 100,
        rain_amount_rte: 140,
        rain_acidity: 20,
        gold_amt: 48,
        text: b"THE END?",
    },
    LVlSettings {
        drone_limit: 2,
        fly_limit: 6,
        slider_limit: 5,
        seeker_limit: 3,
        bomber_limit: 2,
        drone_rte: 120,
        rain_chance_rte: 60,
        rain_amount_rte: 120,
        rain_acidity: 30,
        gold_amt: 64,
        text: b"THE END?",
    },
    LVlSettings {
        drone_limit: 6,
        fly_limit: 7,
        slider_limit: 6,
        seeker_limit: 3,
        bomber_limit: 4,
        drone_rte: 100,
        rain_chance_rte: 50,
        rain_amount_rte: 100,
        rain_acidity: 40,
        gold_amt: 64,
        text: b"THE END?",
    },
    LVlSettings {
        drone_limit: 7,
        fly_limit: 8,
        slider_limit: 7,
        seeker_limit: 4,
        bomber_limit: 4,
        drone_rte: 80,
        rain_chance_rte: 40,
        rain_amount_rte: 80,
        rain_acidity: 60,
        gold_amt: 64,
        text: b"THE END?",
    },
];

#[derive(Copy, Clone, PartialEq, Debug)]
enum PowerUp {
    None,
    SuperDrill,
    Invincible,
    Magnet,
}
const POWERUP_TYPES: [PowerUp; 3] = [PowerUp::SuperDrill, PowerUp::Invincible, PowerUp::Magnet];

struct GameMaster {
    rng: Rng,
    seed: u64,
    frame: u32,
    lvl: usize,
    difficulty: u8,
    hp: u8,
    player_pos: Pos,
    dir: u8, // 0=none,1=left,2=right,3=down,4=left+down,5=right+down
    world: MiniBitVec,
    door_loc: Pos,
    powerup_loc: Pos,
    powerup_taken: bool,
    powerup_cur: PowerUp,
    powerup_frames: u16,
    gold_locs: Vec<Pos>,
    gold: u16,
    drill_speed: u8,
    drill_heat_max: u16,
    drill_heat: u16,
    drill_overheat: bool,
    cur_lvl_data: LVlSettings,
    rain_locs: Vec<Pos>,
    drone_locs: Vec<Pos>,
    fly_locs: Vec<Pos>,
    slider_locs: Vec<Pos>,
    seeker_locs: Vec<Pos>,
    bomber_locs: Vec<Pos>,
    bomber_times: Vec<u16>,
    wind_speed: i8,
    player_flags_last: u32,
    dmg_frames: u8,
    no_input_frames: u8,
    has_drilled: bool,
    is_drilling: bool,
    screen: Screen,
    cost_heart: u16,
    cost_drill_speed: u16,
    cost_drill_cool: u16,
    purchased: u8,
    auto_drill: bool,
    gameover_acc: u8,
    pal_index: usize,
    last_dmg_from: String,
    door_timer: u16,
}
impl GameMaster {
    fn new() -> Self {
        Self {
            rng: Rng::new(),
            seed: 0,
            frame: 0,
            lvl: 0,
            difficulty: 1,
            hp: 4,
            player_pos: Pos { x: 48, y: 0 },
            dir: 0,
            world: MiniBitVec {
                data: Vec::new(),
                len: 0,
            },
            door_loc: Pos { x: 0, y: 0 },
            powerup_loc: Pos { x: 0, y: 0 },
            powerup_taken: false,
            powerup_cur: PowerUp::None,
            powerup_frames: 0,
            gold_locs: Vec::new(),
            gold: 0,
            drill_speed: 48,
            drill_heat_max: 256,
            drill_heat: 0,
            drill_overheat: false,
            cur_lvl_data: LVlSettings::new(),
            rain_locs: Vec::new(),
            drone_locs: Vec::new(),
            fly_locs: Vec::new(),
            slider_locs: Vec::new(),
            seeker_locs: Vec::new(),
            bomber_locs: Vec::new(),
            bomber_times: Vec::new(),
            wind_speed: 0,
            player_flags_last: BLIT_1BPP,
            dmg_frames: 0,
            no_input_frames: 0,
            has_drilled: false,
            is_drilling: false,
            screen: Screen::Intro,
            cost_heart: 8,
            cost_drill_speed: 16,
            cost_drill_cool: 16,
            purchased: 0, // None, heart, drill speed, drill cool
            auto_drill: true,
            gameover_acc: 0,
            pal_index: 0,
            last_dmg_from: String::new(),
            door_timer: 0,
        }
    }

    fn input_check(&mut self, check: u8) -> bool {
        if self.no_input_frames > 0 {
            return false;
        }
        let gamepad = unsafe { *GAMEPAD1 };
        (gamepad & check) != 0
    }

    fn input_check_any(&mut self) -> bool {
        if self.no_input_frames > 0 {
            return false;
        }
        let gamepad = unsafe { *GAMEPAD1 };
        gamepad != 0
    }

    fn input_main(&mut self) {
        let pos_cache = self.player_pos;
        let mut drill_on = false;
        if self.input_check(BUTTON_1) || self.auto_drill {
            if !self.drill_overheat {
                drill_on = true;
            }
        }
        self.dir = 0;
        let mut lr = 0;
        if self.input_check(BUTTON_LEFT) {
            self.player_pos.x -= 1;
            self.dir = 1;
            lr = 1;
        }
        if self.input_check(BUTTON_RIGHT) {
            self.player_pos.x += 1;
            self.dir = 2;
            lr = 2;
        }
        self.is_drilling = false;
        if drill_on && self.input_check(BUTTON_DOWN) {
            self.is_drilling = true;
            self.has_drilled = true;
            self.dir = 3 + lr;
            // Remove the 4 blocks under the smiley
            self.world_drill_area(
                (self.player_pos.x - 1) as usize,
                (self.player_pos.y + PLAYER_SIZE as i16) as usize,
                (PLAYER_SIZE + 2) as usize,
                1,
                self.drill_speed,
            );
        } else if drill_on && self.input_check(BUTTON_UP) {
            self.is_drilling = true;
            self.has_drilled = true;
            self.dir = 6 + lr;
            // Remove the 4 blocks above the smiley
            self.world_drill_area(
                (self.player_pos.x - 1) as usize,
                (self.player_pos.y - 5) as usize,
                (PLAYER_SIZE + 2) as usize,
                5,
                self.drill_speed,
            );
        }

        if drill_on && self.input_check(BUTTON_RIGHT) {
            self.is_drilling = true;
            self.has_drilled = true;
            // Remove the 4 blocks to the right of the smiley
            self.world_drill_area(
                (self.player_pos.x + PLAYER_SIZE as i16 - 1) as usize,
                (self.player_pos.y - 1) as usize,
                1,
                (PLAYER_SIZE + 1) as usize,
                self.drill_speed,
            );
        }
        if drill_on && self.input_check(BUTTON_LEFT) {
            self.is_drilling = true;
            self.has_drilled = true;
            // Remove the 4 blocks to the left of the smiley
            self.world_drill_area(
                (self.player_pos.x) as usize,
                (self.player_pos.y - 1) as usize,
                1,
                (PLAYER_SIZE + 1) as usize,
                self.drill_speed,
            );
        }
        self.player_collide_world(pos_cache);
        let pos_cache = self.player_pos;
        self.player_pos.y += 1;
        if self.player_pos.y > (WORLD_SIZE - PLAYER_SIZE as usize) as i16 {
            self.player_pos.y = (WORLD_SIZE - PLAYER_SIZE as usize) as i16;
        }
        self.player_collide_world(pos_cache);
        self.player_wrap();
    }

    fn world_reset(&mut self) {
        let gold = self.gold;
        let rng = self.rng.clone();
        let lvl = self.lvl;
        let difficulty = self.difficulty;
        let auto_drill = self.auto_drill;
        let hp = self.hp;
        let drill_speed = self.drill_speed;
        let drill_heat_max = self.drill_heat_max;
        let pal_index = self.pal_index;

        *self = GameMaster::new();

        self.gold = gold;
        self.rng = rng;
        self.lvl = lvl;
        self.difficulty = difficulty;
        self.auto_drill = auto_drill;
        self.hp = hp;
        self.drill_speed = drill_speed;
        self.drill_heat_max = drill_heat_max;
        self.pal_index = pal_index;
    }

    fn world_gen(&mut self) {
        self.world = MiniBitVec::new();
        trace("World");
        for y in 0..WORLD_SIZE {
            for _ in 0..WORLD_SIZE {
                let mut alive = y >= DIRT_START as usize;
                if self.rng.i32(0..100) < 2 {
                    alive = false;
                }
                self.world.push(alive);
            }
        }
        // Generate some random gold locations
        trace("Gold");
        for _ in 0..self.cur_lvl_data.gold_amt {
            let x = self.rng.i16(0..(WORLD_SIZE as i16));
            let y = self.rng.i16(DIRT_START as i16..(WORLD_SIZE as i16));
            self.gold_locs.push(Pos::new(x, y));
        }
        // Exit location
        trace("Exit");
        let exit_x = self.rng.i16(4..(WORLD_SIZE as i16 - 12));
        trace(format!("Exit: {}", exit_x));
        self.door_loc = Pos::new(exit_x, 152);
        self.world_set_area(
            (self.door_loc.x as usize).saturating_sub(4),
            (self.door_loc.y as usize).saturating_sub(2),
            16,
            14,
            false,
        );
        // Powerup location
        trace("Powerup");
        let pu_x = self.rng.i16(4..(WORLD_SIZE as i16 - 12));
        // Only spawn at higher y
        let pu_y = self.rng.i16(DIRT_START as i16..(DIRT_START as i16 + 64));
        self.powerup_loc = Pos::new(pu_x, pu_y);
        self.world_set_area(
            (self.powerup_loc.x as usize).saturating_sub(4),
            (self.powerup_loc.y as usize).saturating_sub(2),
            16,
            10,
            false,
        );

        // Enemy locations
        fn spawn_loc(rng: &mut Rng) -> Pos {
            let min_x = 0;
            let max_x = WORLD_SIZE as i16 - 8;
            let min_y = DIRT_START as i16 + 12;
            let max_y = WORLD_SIZE as i16 - 8;
            let x = rng.i16(min_x..max_x);
            let y = rng.i16(min_y..max_y);
            Pos::new(x, y)
        }
        // Fly locations
        trace("Flies");
        for _ in 0..self.cur_lvl_data.fly_limit {
            self.fly_locs.push(spawn_loc(&mut self.rng));
        }
        // Slider locations
        trace("Sliders");
        for _ in 0..self.cur_lvl_data.slider_limit {
            self.slider_locs.push(spawn_loc(&mut self.rng));
        }
        // Seeker locations
        trace("Seekers");
        for _ in 0..self.cur_lvl_data.seeker_limit {
            self.seeker_locs.push(spawn_loc(&mut self.rng));
        }
        // Bomber locations
        trace("Bombers");
        for _ in 0..self.cur_lvl_data.bomber_limit {
            self.bomber_locs.push(spawn_loc(&mut self.rng));
            self.bomber_times.push(0);
        }
        // Wind speed
        self.wind_speed = self.rng.i8(5..95);
    }

    fn world_get(&self, x: usize, y: usize) -> Option<bool> {
        let index = WORLD_SIZE.saturating_mul(y).saturating_add(x);
        self.world.get(index)
    }

    fn world_set(&mut self, x: usize, y: usize, value: bool) {
        let index = WORLD_SIZE.saturating_mul(y).saturating_add(x);
        self.world.set(index, value);
    }

    fn world_set_area(&mut self, x: usize, y: usize, w: usize, h: usize, value: bool) {
        for dy in 0..h {
            for dx in 0..w {
                let wx = x.saturating_add(dx);
                let wy = y.saturating_add(dy);
                self.world_set(wx, wy, value);
            }
        }
    }

    fn world_set_circle(&mut self, x: usize, y: usize, r: usize, value: bool) {
        let r_sq = (r * r) as i32;
        let cx = x as i32;
        let cy = y as i32;
        let r_i = r as i32;
        for dy in -r_i..=r_i {
            for dx in -r_i..=r_i {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= r_sq {
                    let wx = cx + dx;
                    let wy = cy + dy;
                    if wx >= 0
                        && wy >= 0
                        && (wx as usize) < WORLD_SIZE
                        && (wy as usize) < WORLD_SIZE
                    {
                        self.world_set(wx as usize, wy as usize, value);
                    }
                }
            }
        }
    }

    // Like set area but with chance
    fn world_drill_area(&mut self, x: usize, y: usize, w: usize, h: usize, chance: u8) {
        // Prevent overflow and out-of-bounds
        if x.checked_add(w).map_or(true, |end_x| end_x > WORLD_SIZE)
            || y.checked_add(h).map_or(true, |end_y| end_y > WORLD_SIZE)
        {
            // NOTE: This is a pretty buggy code section
            // It used to return early, but that caused issues when drilling near edges
            // So now we modify w and h to fit within bounds
            // Problems here will cause out-of-bounds panics

            // Modify w and h to fit
            let w = if x >= WORLD_SIZE { 0 } else { WORLD_SIZE - x };
            let h = if y >= WORLD_SIZE { 0 } else { WORLD_SIZE - y };
            if w == 0 || h == 0 {
                return;
            }
        }

        let mut sfx = false;
        for dy in 0..h {
            for dx in 0..w {
                let wx = x + dx;
                let wy = y + dy;
                if self.rng.i32(0..128) < chance as i32 || self.powerup_cur == PowerUp::SuperDrill {
                    self.world_set(wx, wy, false);
                    sfx = true;
                }
            }
        }

        if sfx {
            self.sfx_drill();
        }
    }

    fn collides(&self, origin_a: &Pos, size_a: &Pos, origin_b: &Pos, size_b: &Pos) -> bool {
        !(origin_a.x + size_a.x <= origin_b.x
            || origin_a.x >= origin_b.x + size_b.x
            || origin_a.y + size_a.y <= origin_b.y
            || origin_a.y >= origin_b.y + size_b.y)
    }

    fn collides_player(&self, pos: &Pos, size: &Pos) -> bool {
        let player_size = Pos::new(PLAYER_SIZE as i16, PLAYER_SIZE as i16);
        self.collides(&self.player_pos, &player_size, pos, size)
    }

    fn collides_world(&self, pos: &Pos, size: &Pos) -> bool {
        for dy in 0..size.y {
            for dx in 0..size.x {
                let wx = (pos.x + dx) as usize;
                let wy = (pos.y + dy) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    if let Some(cell) = self.world_get(wx, wy) {
                        if cell {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn world_clear_at_player(&mut self) {
        for dy in 0..PLAYER_SIZE as i16 {
            for dx in 0..PLAYER_SIZE as i16 {
                let wx = (self.player_pos.x + dx) as usize;
                let wy = (self.player_pos.y + dy) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    self.world_set(wx, wy, false);
                }
            }
        }
    }

    fn player_collide_world(&mut self, cache: Pos) -> bool {
        // Collision with world
        let mut collided = 0;
        for dy in 0..PLAYER_SIZE {
            for dx in 0..PLAYER_SIZE {
                let wx = (self.player_pos.x + dx as i16) as usize;
                let wy = (self.player_pos.y + dy as i16) as usize;
                if wx < WORLD_SIZE && wy < WORLD_SIZE {
                    if let Some(cell) = self.world_get(wx, wy) {
                        if cell {
                            collided += 1;
                        }
                    }
                }
            }
        }
        if collided > 2 {
            self.player_pos = cache;
            return false;
        }
        true
    }

    fn player_collide_misc(&mut self) {
        // NOTE: Player only checks against non-logical/non-moving ents here
        // Other ents check their own collisions

        // Check for gold collection
        unsafe {
            #[allow(static_mut_refs)]
            // TODO: THIS IS BAD
            self.gold_locs.retain(|gold| {
                let collided = GM.collides_player(gold, &Pos { x: 4, y: 4 });
                if collided {
                    GM.sfx_gold();
                    GM.drill_heat = GM.drill_heat.saturating_sub(GM.drill_heat_max / 10);
                    GM.gold += 1;
                    false
                } else {
                    true
                }
            });
        }
        // Check for collisions with doors
        let door_collide = self.collides_player(&self.door_loc, &Pos { x: 8, y: 8 });
        if door_collide {
            self.door_timer = self.door_timer.saturating_add(1);
            self.sfx_door();
            // Watch for game over, only allow level change if alive
            if self.door_timer == DOOR_TIMER && self.hp > 0 {
                self.next_level();
            }
        } else {
            self.door_timer = 0;
        }
        // Check for powerup collection
        if !self.powerup_taken {
            let pu_collide = self.collides_player(&self.powerup_loc, &Pos { x: 8, y: 8 });
            if pu_collide {
                self.powerup_taken = true;
                self.sfx_ok();
                // Random powerup
                let pu_index = self.rng.u32(0..POWERUP_TYPES.len() as u32) as usize;
                self.powerup_cur = POWERUP_TYPES[pu_index].clone();
                // 10 seconds at 60fps
                self.powerup_frames = POWERUP_FRAMES;
                // Give 1 HP if invincible
                if self.powerup_cur == PowerUp::Invincible {
                    self.hp += 1;
                    if self.hp > MAX_HP {
                        self.hp = MAX_HP;
                    }
                }
            }
        }
    }

    fn player_wrap(&mut self) {
        if self.player_pos.x < 0 {
            self.player_pos.x = (WORLD_SIZE - PLAYER_SIZE as usize) as i16;
            self.world_clear_at_player();
        }
        if self.player_pos.x > (WORLD_SIZE - PLAYER_SIZE as usize) as i16 {
            self.player_pos.x = 0;
            self.world_clear_at_player();
        }
    }

    fn player_dmg(&mut self, from: &str) {
        if self.powerup_cur == PowerUp::Invincible {
            return;
        }
        self.dmg_frames = DMG_FRAMES;
        self.hp = self.hp.saturating_sub(1);
        self.sfx_dmg();
        trace(format!("DMG FROM: {}: HP={}", from, self.hp));
        self.last_dmg_from = from.to_string();
    }

    fn render_gold_text(&mut self, x: i32, y: i32, amt: u16) {
        let frame = (self.frame / 16) % 2;
        if frame == 0 {
            blit(&GOLDLRG1, x, y, 8, 8, BLIT_1BPP);
        } else {
            blit(&GOLDLRG2, x, y, 8, 8, BLIT_1BPP);
        }
        text(&format!("{}", amt), x + 10, y);
    }

    fn sfx_rain(&mut self, p: &Pos) {
        let f = self.rng.u32(440..880);
        let dist = p.distance(&self.player_pos) as u32;
        let vol = 50 + (if dist > 50 { 20 } else { 50 - dist });
        tone(f, 1, vol, TONE_PULSE2);
    }

    fn sfx_door(&mut self) {
        let add = self.door_timer as u32 * 8;
        let f = (self.frame as f32 / 3.14).sin() * 100.0 + 120.0 + add as f32;
        tone(f as u32, 32, 100, TONE_PULSE2);
    }

    fn sfx_gold(&mut self) {
        let f = self.rng.u32(500..540);
        tone(f | (900 << 16), 4, 128, TONE_PULSE1);
    }

    fn sfx_dmg(&mut self) {
        let f = self.rng.u32(200..220);
        tone(f * 2 | (f << 16), 8, 128, TONE_PULSE1);
    }

    fn sfx_drill(&mut self) {
        let max = 440 - self.player_pos.y as u32 * 2; // 160
        let f = self.rng.u32(120..max);
        tone(f, 1, 45, TONE_NOISE);
    }

    fn sfx_drill_overheat(&mut self) {
        tone(150 | (220 << 16), 120, 128, TONE_NOISE);
    }

    fn sfx_drill_warn(&mut self) {
        // let f = (self.drill_heat as f32 / self.drill_heat_max as f32) * 300.;
        // tone(650 + f as u32, 1, 128, TONE_TRIANGLE);
        let f = self.rng.u32(440..880);
        tone(f | (1000 << 16), 4, 100, TONE_TRIANGLE);
    }

    fn sfx_explode(&mut self) {
        tone(200 | (500 << 16), 60, 128, TONE_NOISE);
    }

    fn sfx_ok(&mut self) {
        tone(400 | (600 << 16), 4, 128, TONE_PULSE1);
    }

    fn sfx_deny(&mut self) {
        tone(400, 2, 128, TONE_PULSE1);
    }

    fn sfx_screen_change(&mut self) {
        tone(166 | (220 << 16), 8, 128, TONE_PULSE2);
    }

    fn next_level(&mut self) {
        self.lvl += 1;
        // Check if we just completed the last level
        if self.lvl >= MAX_LVL - 1 {
            self.screen_set(Screen::GameOver);
            return;
        }
        self.world_reset();
        self.cur_lvl_data = LVLS[self.lvl];
        self.cur_lvl_data.apply_difficulty(self.difficulty);
        self.world_gen();
        self.screen_set(Screen::Shop);
        return;
    }

    fn up_drill(&mut self) {
        if self.is_drilling {
            self.drill_heat = self.drill_heat.saturating_add(1);
        } else if self.drill_overheat {
            // Slower cooldown when overheated
            self.drill_heat = self.drill_heat.saturating_sub(1);
        } else {
            self.drill_heat = self.drill_heat.saturating_sub(2);
        }
        if self.drill_heat > (self.drill_heat_max as f32 * 0.7) as u16 && !self.drill_overheat {
            if self.frame % 8 == 0 {
                self.sfx_drill_warn();
            }
        }
        if self.drill_heat >= self.drill_heat_max && self.powerup_cur != PowerUp::SuperDrill {
            self.drill_overheat = true;
            self.sfx_drill_overheat();
        }
        // Release overheat when cooled down
        if self.drill_heat == 0 {
            self.drill_overheat = false;
        }
    }

    // Basic rain update (no collisions)
    fn up_rain_pos(&mut self, chance: u32, rate: u32, max: usize, wind: u8) {
        // Add rain
        let mut rain_chance = self.frame / chance;
        if rain_chance > 100 {
            rain_chance = 100;
        }
        let mut rain_amount = self.frame / rate as u32;
        if rain_amount > 4 {
            rain_amount = 4;
        }
        if self.rain_locs.len() < max - rain_amount as usize {
            for _ in 0..rain_amount {
                if self.rng.i32(0..100) < rain_chance as i32 {
                    let x = self.rng.i16(0..(WORLD_SIZE as i16));
                    self.rain_locs.push(Pos::new(x, 0));
                }
            }
        }
        // Move rain
        for rain in &mut self.rain_locs {
            rain.y += 2;
            if self.rng.i32(0..100) < wind as i32 {
                rain.x += 1;
            }
        }
        // Check out of bounds rain
        self.rain_locs.retain(|rain| rain.y < WORLD_SIZE as i16 + 1);
        self.rain_locs
            .retain(|rain| rain.x >= 0 && rain.x < WORLD_SIZE as i16);
    }

    // Rain collisions
    fn up_rain_col(&mut self) {
        self.up_rain_pos(
            self.cur_lvl_data.rain_chance_rte as u32,
            self.cur_lvl_data.rain_amount_rte as u32,
            RAIN_MAX,
            self.wind_speed as u8,
        );

        // Check for collision with player
        let mut hits_player: Vec<usize> = Vec::new();
        for (i, rain) in self.rain_locs.iter().enumerate() {
            let collides = self.collides_player(rain, &Pos::new(2, 2));
            if collides {
                hits_player.push(i);
            }
        }
        for &i in hits_player.iter().rev() {
            self.rain_locs.remove(i);
            self.player_dmg("rain");
        }
        // Clear world blocks
        for i in (0..self.rain_locs.len()).rev() {
            let rain = self.rain_locs[i].clone();
            let hit = self.collides_world(&Pos::new(rain.x, rain.y - 1), &Pos::new(1, 2));
            if hit {
                // Remove rain if it hits the world
                if self.rng.u8(0..100) > self.cur_lvl_data.rain_acidity {
                    self.sfx_rain(&rain);
                    self.rain_locs.remove(i);
                }
            }
            self.world_set_area(rain.x as usize, rain.y as usize - 1, 1, 2, false);
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
                }
            }
        }
        for &i in hits_gold.iter().rev() {
            if i < self.rain_locs.len() {
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
                self.world_set_circle(
                    self.rain_locs[i].x as usize,
                    self.rain_locs[i].y as usize,
                    4,
                    false,
                );
                self.rain_locs.remove(i);
            }
        }
    }

    fn up_drones(&mut self) {
        // Add drones
        if self.frame % self.cur_lvl_data.drone_rte as u32 == 0
            && self.drone_locs.len() < self.cur_lvl_data.drone_limit
        {
            let x = self.rng.i16(0..(WORLD_SIZE as i16));
            self.drone_locs.push(Pos::new(x, 0));
        }
        // Check for collision with player
        let mut hits_player: Vec<usize> = Vec::new();
        for (i, drone) in self.drone_locs.iter().enumerate() {
            let collides = self.collides_player(drone, &Pos::new(8, 4));
            if collides {
                hits_player.push(i);
            }
        }
        for &i in hits_player.iter().rev() {
            self.drone_locs.remove(i);
            self.player_dmg("drone");
        }
        // Update drones every N frames
        if self.frame % 16 != 0 {
            return;
        }
        // Move towards player
        for drone in &mut self.drone_locs {
            let dx = self.player_pos.x - drone.x;
            let dy = self.player_pos.y - drone.y;
            let dist = self.player_pos.distance(drone);
            if dist > 1. {
                let step_x = (dx as f32 / dist).round() as i16;
                let step_y = (dy as f32 / dist).round() as i16;
                drone.x += step_x;
                drone.y += step_y;
                drone.clamp_to_world();
            }
        }
        // Clear world blocks
        for i in 0..self.drone_locs.len() {
            let drone = self.drone_locs[i].clone();
            self.world_set_area(drone.x as usize, drone.y as usize, 8, 4, false);
        }
    }

    fn up_flies(&mut self) {
        // Check for collision with player
        let mut hits_player: Vec<usize> = Vec::new();
        for (i, fly) in self.fly_locs.iter().enumerate() {
            let collides = self.collides_player(fly, &Pos::new(8, 4));
            if collides {
                hits_player.push(i);
            }
        }
        for &i in hits_player.iter().rev() {
            self.fly_locs.remove(i);
            self.player_dmg("fly");
        }
        // Only move every N frames
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
        // Check for collision with world
        for i in 0..self.fly_locs.len() {
            let fly = &self.fly_locs[i];
            self.world_set_area(fly.x as usize, fly.y as usize, 8, 4, false);
        }
    }

    fn up_sliders(&mut self) {
        // Check for collision with player
        let mut hits_player: Vec<usize> = Vec::new();
        for (i, slider) in self.slider_locs.iter().enumerate() {
            let collides = self.collides_player(slider, &Pos::new(8, 4));
            if collides {
                hits_player.push(i);
            }
        }
        for &i in hits_player.iter().rev() {
            self.slider_locs.remove(i);
            self.player_dmg("slider");
        }
        // Only move every N frames
        if self.frame % 8 != 0 {
            return;
        }
        // Sliders move left and right only
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
        // Check for collision with world
        for i in 0..self.slider_locs.len() {
            let slider = &self.slider_locs[i];
            self.world_set_area(slider.x as usize, slider.y as usize, 8, 4, false);
        }
    }

    // Seekers move towards the player when the player gets close
    fn up_seekers(&mut self) {
        // Check for collision with player
        let mut hits_player: Vec<usize> = Vec::new();
        for (i, seeker) in self.seeker_locs.iter().enumerate() {
            let collides = self.collides_player(seeker, &Pos::new(8, 8));
            if collides {
                hits_player.push(i);
            }
        }
        for &i in hits_player.iter().rev() {
            self.seeker_locs.remove(i);
            self.player_dmg("seeker");
        }
        // Only move every N frames
        if self.frame % 16 != 0 {
            return;
        }
        // Move towards player if close enough
        for seeker in &mut self.seeker_locs {
            let dx = self.player_pos.x - seeker.x;
            let dy = self.player_pos.y - seeker.y;
            let dist = self.player_pos.distance(seeker);
            if dist < 64. && dist > 1. {
                let step_x = (dx as f32 / dist).round() as i16;
                let step_y = (dy as f32 / dist).round() as i16;
                seeker.x += step_x;
                seeker.y += step_y;
                seeker.clamp_to_world();
            }
        }
        // Check for collision with world
        for i in 0..self.seeker_locs.len() {
            let seeker = &self.seeker_locs[i];
            self.world_set_area(seeker.x as usize, seeker.y as usize, 8, 8, false);
        }
    }

    fn up_bombers(&mut self) {
        // Check for collision with player
        let mut hits: Vec<usize> = Vec::new();
        let hit_dist = 24.;
        for (i, bomber) in self.bomber_locs.iter().enumerate() {
            let dist = self.player_pos.distance(bomber);
            if dist < hit_dist {
                hits.push(i);
            }
        }
        for &i in hits.iter().rev() {
            if self.bomber_times[i] == 0 {
                self.bomber_times[i] = 64; // Start countdown
            }
        }
        for time in &mut self.bomber_times {
            if *time > 0 {
                *time = time.saturating_sub(1);
            }
        }
        // Bombers explode when timer reaches 1 (0 is safe state)
        let mut to_explode: Vec<usize> = Vec::new();
        for (i, &time) in self.bomber_times.iter().enumerate() {
            if time == 1 {
                to_explode.push(i);
            }
        }
        // TODO: There may be a bug here
        // Bombers indexes may shift when removing bombers
        // Should use a bombers_alive Vec<bool> instead
        for &i in to_explode.iter().rev() {
            let bomber = self.bomber_locs[i].clone();
            // Clear area around bomber
            self.world_set_circle(bomber.x as usize, bomber.y as usize, 16, false);
            let bomb_offset = Pos::new(bomber.x - 16, bomber.y - 16);
            let hit_player = self.collides_player(&bomb_offset, &Pos::new(32, 32));
            if hit_player {
                self.player_dmg("bomber");
            }
            self.bomber_locs.remove(i);
            self.bomber_times.remove(i);
            self.sfx_explode();
            let drops = self.rng.i16(2..5);
            for _ in 0..drops {
                let pos = Pos::new(
                    bomber.x + self.rng.i16(-16..17),
                    bomber.y + self.rng.i16(-16..17),
                );
                self.gold_locs.push(pos);
            }
        }

        // Bombers fall down
        if self.frame % 8 != 0 {
            return;
        }
        for i in 0..self.bomber_locs.len() {
            let bomber = self.bomber_locs[i].clone();
            let collides = self.collides_world(&Pos::new(bomber.x, bomber.y + 8), &Pos::new(8, 1));
            if !collides {
                self.bomber_locs[i].y += 1;
                self.bomber_locs[i].clamp_to_world();
            }
        }
    }

    // NOTE: This is a VERY expensive operation
    // We need to split the world update over multiple frames or we will run out of memeory
    // The larger the split size the faster the world updates
    // This effects the speed of falling blocks
    // WARN: `split_size` must evenly divide `WORLD_SIZE`
    fn up_world(&mut self) {
        // If world blocks have less than 4 neighbors, they fall down
        let split_size = 4;
        let falling_limit = 160; // Max number of blocks to fall per update
        let world_split = WORLD_SIZE / split_size;
        let world_offset = (self.frame % split_size as u32) as usize;
        let start_y = world_offset * world_split;
        let end_y = start_y + world_split;
        let mut to_fall = Vec::new();
        // NOTE: It's technically possible for to_fall to just hold the index insead of a position
        // This would cut the to_fall size in half
        // But for clarity we will keep it as positions for now
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
                            let px = self.player_pos.x + dx;
                            let py = self.player_pos.y + dy;
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

    fn up_gold(&mut self) {
        if self.frame % 4 != 0 {
            return;
        }
        // Magnet powerup effect
        let mut mag_list = Vec::new();
        if (self.powerup_cur == PowerUp::Magnet) && (self.powerup_frames > 0) {
            // Move gold towards player
            for (i, gold) in &mut self.gold_locs.iter_mut().enumerate() {
                let dx = self.player_pos.x - gold.x;
                let dy = self.player_pos.y - gold.y;
                let dist = self.player_pos.distance(gold);
                // TODO: Gold moving is jank
                if dist < 64. && dist > 1. {
                    let step_x = (dx as f32 / dist).round() as i16;
                    let step_y = (dy as f32 / dist).round() as i16;
                    gold.x += step_x;
                    gold.y += step_y;
                    gold.clamp_to_world();
                    mag_list.push(i);
                }
            }
            // Clear world blocks where gold is
            let gold_positions: Vec<_> =
                self.gold_locs.iter().map(|gold| (gold.x, gold.y)).collect();
            for (x, y) in gold_positions {
                self.world_set_area(x as usize, y as usize, 4, 4, false);
            }
        }
        // Gold falls
        let mut to_fall: Vec<usize> = Vec::new();
        for (i, gold) in &mut self.gold_locs.iter().enumerate() {
            if mag_list.contains(&i) {
                continue; // Skip magnet-affected gold
            }
            let below_x = gold.x as usize;
            let below_y = (gold.y + 4) as usize;
            if below_x < WORLD_SIZE && below_y < WORLD_SIZE {
                if let Some(cell) = self.world_get(below_x, below_y) {
                    if !cell {
                        to_fall.push(i);
                    }
                }
            }
        }
        for gold in to_fall {
            let g = &mut self.gold_locs[gold];
            g.y += 1;
            g.clamp_to_world();
        }
    }

    fn up_powerup(&mut self) {
        // Powerups fall down
        if self.frame % 8 != 0 {
            return;
        }
        let pu = self.powerup_loc.clone();
        let collides = self.collides_world(&Pos::new(pu.x, pu.y + 8), &Pos::new(8, 1));
        if !collides {
            self.powerup_loc.y += 1;
            self.powerup_loc.clamp_to_world();
        }
    }

    fn up_music(&mut self) {
        if !MUSIC_ENABLED {
            return;
        }
        if self.screen == Screen::Game {
            return;
        }
        fn p1(note: u32, vol: u32) {
            tone(note, 4, vol, TONE_PULSE1 | TONE_NOTE_MODE);
        }
        fn p2(note: u32, vol: u32) {
            tone(note, 4, vol, TONE_PULSE2 | TONE_NOTE_MODE);
        }
        fn p3(note: u32, vol: u32) {
            tone(note, 4, vol, TONE_TRIANGLE | TONE_NOTE_MODE);
        }
        fn p4(note: u32, vol: u32) {
            tone(note, 4, vol, TONE_NOISE | TONE_NOTE_MODE);
        }
        if self.screen == Screen::Intro {
            // Simple intro jingle
            let beat = (self.frame / 4) % 32;
            let beat_long = (self.frame / 4) % 64;
            if self.frame % 512 < 256 {
                match beat {
                    0 => p1(60, 100),
                    2 => p1(67, 100),
                    4 => p1(72, 100),
                    8 => p1(74, 100),
                    12 => p1(72, 100),
                    16 => p1(67, 100),
                    20 => p1(65, 100),
                    24 => p1(64, 100),
                    _ => p1(0, 0),
                }
                match beat {
                    0 => p3(50, 80),
                    4 => p3(55, 80),
                    8 => p3(57, 80),
                    12 => p3(55, 80),
                    16 => p3(50, 80),
                    20 => p3(48, 80),
                    24 => p3(47, 80),
                    _ => p3(0, 0),
                }
                match beat_long {
                    0..32 => p2(60, 40),
                    32..64 => p2(65, 40),
                    _ => p4(0, 0),
                }
            } else {
                match beat {
                    0 => p1(72, 100),
                    2 => p1(74, 100),
                    4 => p1(75, 100),
                    8 => p1(77, 100),
                    12 => p1(75, 100),
                    16 => p1(74, 100),
                    20 => p1(72, 100),
                    24 => p1(70, 100),
                    _ => p1(0, 0),
                }
                match beat {
                    0 => p3(55, 80),
                    4 => p3(57, 80),
                    8 => p3(59, 80),
                    12 => p3(57, 80),
                    16 => p3(55, 80),
                    20 => p3(53, 80),
                    24 => p3(52, 80),
                    _ => p3(0, 0),
                }
                match beat_long {
                    0..32 => p2(67, 40),
                    _ => p4(0, 0),
                }
            }
            return;
        }
        // Shop music
        if self.screen == Screen::Shop {
            let beat = (self.frame / 4) % 32;
            if self.frame % 512 < 256 {
                match beat {
                    0 => p2(60, 80),
                    4 => p2(64, 80),
                    8 => p2(67, 80),
                    12 => p2(69, 80),
                    16 => p2(67, 80),
                    20 => p2(64, 80),
                    24 => p2(62, 80),
                    _ => p2(0, 0),
                }
                match beat {
                    0 => p3(50, 60),
                    8 => p3(55, 60),
                    16 => p3(57, 60),
                    24 => p3(55, 60),
                    _ => p3(0, 0),
                }
            } else {
                match beat {
                    0 => p2(67, 80),
                    4 => p2(69, 80),
                    8 => p2(71, 80),
                    12 => p2(72, 80),
                    16 => p2(71, 80),
                    20 => p2(69, 80),
                    24 => p2(67, 80),
                    _ => p2(0, 0),
                }
                match beat {
                    0 => p3(55, 60),
                    8 => p3(57, 60),
                    16 => p3(59, 60),
                    24 => p3(57, 60),
                    _ => p3(0, 0),
                }
            }
            return;
        }
        // Transition music
        if self.screen == Screen::Transition {
            let beat = (self.frame / 4) % 16;
            if self.frame % 256 < 128 {
                match beat {
                    0 => p1(72, 100),
                    4 => p1(75, 100),
                    8 => p1(79, 100),
                    12 => p1(75, 100),
                    _ => p1(0, 0),
                }
                match beat {
                    0 => p3(55, 80),
                    8 => p3(60, 80),
                    _ => p3(0, 0),
                }
            } else {
                match beat {
                    0 => p1(79, 100),
                    4 => p1(75, 100),
                    8 => p1(72, 100),
                    12 => p1(75, 100),
                    _ => p1(0, 0),
                }
                match beat {
                    0 => p3(60, 80),
                    8 => p3(55, 80),
                    _ => p3(0, 0),
                }
            }
            return;
        }
        // Main game music
        let beat = (self.frame / 4) % 32;
        let beat_short = (self.frame / 4) % 16;
        let beat_long = (self.frame / 4) % 64;
        if self.frame % 1024 < 256 {
            // match beat_short {
            //     0 => p3(70, 128),
            //     2 => p3(75, 128),
            //     4 => p3(70, 128),
            //     6 => p3(73, 128),
            //     8 => p3(75, 128),
            //     10 => p3(70, 128),
            //     12 => p3(75, 128),
            //     14 => p3(70, 128),
            //     16 => p3(73, 128),
            //     18 => p3(72, 128),
            //     _ => p3(0, 0),
            // }
            if beat % 4 == 0 {
                p3(70, beat * 4);
            } else {
                p3(0, 0);
            }
            match beat_long {
                0..8 => p1(58, 80),
                16..24 => p1(61, 80),
                32..48 => p1(65, 80),
                52..58 => p1(61, 80),
                60..64 => p1(58, 80),
                _ => p1(0, 0),
            }
            match beat_long {
                0..8 => p2(58 - 24, 100),
                16..24 => p2(61 - 24, 100),
                32..48 => p2(63 - 24, 100),
                52..58 => p2(61 - 24, 100),
                60 => p2(58 - 12, 100),
                _ => p2(0, 0),
            }
            if beat % 8 == 0 {
                p4(100, 32);
            } else {
                p4(0, 0);
            }
        } else if self.frame % 1024 < 512 {
            // match beat_short {
            //     0 => p3(70, 128),
            //     2 => p3(75, 128),
            //     4 => p3(70, 128),
            //     6 => p3(73, 128),
            //     8 => p3(75, 128),
            //     10 => p3(70, 128),
            //     12 => p3(75, 128),
            //     14 => p3(70, 128),
            //     16 => p3(73, 128),
            //     18 => p3(72, 128),
            //     _ => p3(0, 0),
            // }
            if beat % 2 == 0 {
                p3(70, beat * 4);
            } else {
                p3(0, 0);
            }
            match beat_short {
                0 => p1(70 + 12, 90),
                2 => p1(75 + 12, 90),
                4 => p1(70 + 12, 90),
                6 => p1(73 + 12, 90),
                8 => p1(75 + 12, 90),
                10 => p1(70 + 12, 90),
                12 => p1(75 + 12, 90),
                14 => p1(70 + 12, 90),
                16 => p1(73 + 12, 90),
                18 => p1(72 + 12, 90),
                _ => p1(0, 0),
            }
            match beat_long {
                0..8 => p1(58, 80),
                16..24 => p1(61, 80),
                32..48 => p1(68, 80),
                52..58 => p1(61, 80),
                60..64 => p1(58, 80),
                _ => p1(0, 0),
            }
            match beat_long {
                0..8 => p2(58 - 12, 100),
                16..24 => p2(61 - 12, 100),
                32..48 => p2(66 - 12, 100),
                52..58 => p2(65 - 12, 100),
                60 => p2(58 - 12, 100),
                _ => p2(0, 0),
            }
            if beat % 4 == 0 {
                p4(100, 32);
            } else {
                p4(0, 0);
            }
        } else if self.frame % 1024 < 768 {
            let notes = [70, 75, 75, 75, 75, 75, 73, 77, 77 + 12];
            let mut off = beat_long as usize / 8;
            off = off.min(notes.len() - 2);
            // if off >= notes.len() - 2 {
            //     off = notes.len() - 2;
            // }
            match beat_short {
                0 => p1(70, 80),
                2 => p1(75, 80),
                4 => p1(70, 80),
                6 => p1(73, 80),
                8 => p1(75, 80),
                10..16 => p1(
                    notes[self.rng.u32(off as u32..notes.len() as u32) as usize],
                    80,
                ),
                // 10 => p1(70, 80),
                // 12 => p1(75, 80),
                // 14 => p1(70, 80),
                // 16 => p1(73, 80),
                // 18 => p1(72, 80),
                _ => p1(0, 0),
            }
            match beat_long {
                0..8 => p2(58 - 24, 100),
                10..12 => p2(58 - 12, 100),
                16..24 => p2(61 - 24, 100),
                26..28 => p2(61 - 12, 100),
                32..48 => p2(63 - 24, 100),
                52..58 => p2(68 - 24, 100),
                60..62 => p2(70 - 24, 100),
                _ => p2(0, 0),
            }
            match beat {
                0 => p3(70, 80),
                2 => p3(70, 80),
                4 => p3(75, 80),
                6 => p3(75, 80),
                8 => p3(77, 80),
                10 => p3(77, 80),
                12 => p3(75, 80),
                14 => p3(75, 80),
                16 => p3(70, 80),
                20 => p3(68, 80),
                24 => p3(67, 80),
                28 => p3(63, 80),
                _ => p3(0, 0),
            }
            if beat % 4 == 0 {
                p4(100, 32);
            } else if beat % 2 == 0 && beat_long > 32 {
                p4(120, 48);
            } else {
                p4(0, 0);
            }
        } else {
            match beat_short {
                0 => p1(70, 80),
                2 => p1(75, 80),
                4 => p1(70, 80),
                6 => p1(73, 80),
                8 => p1(75, 80),
                10 => p1(70, 80),
                12 => p1(75, 80),
                14 => p1(70, 80),
                16 => p1(73, 80),
                18 => p1(72, 80),
                _ => p1(0, 0),
            }
            match beat_long {
                0..8 => p1(58, 80),
                16..24 => p1(61, 80),
                32..48 => p1(63, 80),
                52..58 => p1(61, 80),
                60..64 => p1(58, 80),
                _ => p1(0, 0),
            }
            match beat_long {
                0..8 => p2(58 - 24, 80),
                10..12 => p2(58 - 12, 80),
                16..24 => p2(61 - 24, 80),
                26..28 => p2(61 - 12, 80),
                32..34 => p2(63 - 24, 80),
                36..38 => p2(63 - 12, 80),
                52..58 => p2(68 - 24, 80),
                60..62 => p2(70 - 24, 80),
                _ => p2(0, 0),
            }
            match beat {
                0 => p3(70, 80),
                2 => p3(70, 80),
                4 => p3(75, 80),
                6 => p3(75, 80),
                8 => p3(77, 80),
                10 => p3(77, 80),
                12 => p3(75, 80),
                14 => p3(75, 80),
                16 => p3(70, 80),
                20 => p3(68, 80),
                24 => p3(67, 80),
                28 => p3(63, 80),
                _ => p3(0, 0),
            }
            if beat % 4 == 0 {
                p4(100, 32);
            } else if beat % 2 == 0 && beat_long > 32 {
                p4(120, 48);
            } else {
                p4(0, 0);
            }
        }

        // let beat = (self.frame / 2) % 64;
        // match beat {
        //     0 => p1(70 - 24, 100),
        //     1 => p1(70 - 24, 100),
        //     16 => p1(70 - 24, 100),
        //     17 => p1(70 - 24, 100),
        //     _ => p1(0, 0),
        // }
        // match beat {
        //     0 => p3(70, 100),
        //     1 => p3(75, 100),
        //     4 => p3(77, 100),
        //     _ => p3(0, 0),
        // }
        // let beat_noise = (self.frame / 8) % 4;
        // if beat_noise == 0 && (self.frame / 512) % 2 == 1 {
        //     p4(120, 48);
        // } else if beat_noise == 2 && (self.frame / 1024) % 2 == 1 {
        //     p4(120, 32);
        // } else {
        //     p4(0, 0);
        // }
        // let notes = [0, 70, 73, 75, 77];
        // let beat_rng = (self.frame / 8) % 32;
        // let note = notes[self.rng.i32(0..notes.len() as i32) as usize];
        // match beat_rng {
        //     2 => p2(note, 80),
        //     4 => p2(75 - 12, 80),
        //     8 => p2(note, 80),
        //     10 => p2(73 - 12, 80),
        //     12 => p2(70 - 12, 80),
        //     18 => p2(note, 80),
        //     20 => p2(75 - 12, 80),
        //     24 => p2(note, 80),
        //     26 => p2(73 - 12, 80),
        //     28 => p2(70 - 12, 80),
        //     31 => p2(68 - 12, 80),
        //     _ => p2(0, 0),
        // }
    }

    fn screen_set(&mut self, screen: Screen) {
        // Clear rain locs because they persist between screens
        // and we might not do a world reset on them
        self.rain_locs.clear();
        self.frame = 0;
        if screen != Screen::Game {
            self.no_input_frames = NO_INPUT_FRAMES;
        }
        self.sfx_screen_change();
        self.screen = screen;
    }

    fn up_sc_intro(&mut self) {
        if self.screen != Screen::Intro {
            return;
        }
        if self.input_check_any() {
            self.screen_set(Screen::Start);
        }
        if self.frame > 1024 {
            self.screen_set(Screen::Start);
            return;
        }
        self.up_rain_pos(10, 20, RAIN_MAX / 2, 2);
    }

    fn up_sc_start(&mut self) {
        if self.screen != Screen::Start {
            return;
        }
        self.seed += 1; // Increment seed while on start screen
        if self.input_check(BUTTON_2) {
            self.no_input_frames = NO_INPUT_FRAMES_SH;
            self.pal_index += 1;
            if self.pal_index >= PALS.len() {
                self.pal_index = 0;
            }
            self.sfx_ok();
        }
        if self.input_check(BUTTON_1) {
            // Seed random with current frame
            self.rng = Rng::with_seed(self.seed);
            trace(format!("set seed: {}", self.seed));
            self.next_level();
            // Override next level screen set to transition
            self.screen_set(Screen::Transition);
        }
        if self.input_check(BUTTON_UP) {
            self.no_input_frames = NO_INPUT_FRAMES_SH;
            if self.difficulty < MAX_DIFF {
                self.difficulty = self.difficulty + 1;
                if self.difficulty >= MAX_DIFF {
                    self.difficulty = 0;
                }
            }
            self.sfx_ok();
        }
        if self.input_check(BUTTON_DOWN) {
            self.no_input_frames = NO_INPUT_FRAMES_SH;
            self.auto_drill = !self.auto_drill;
            self.sfx_ok();
        }
        self.up_rain_pos(50, 60, RAIN_MAX / 2, 5);
    }

    fn up_sc_main(&mut self) {
        if self.screen != Screen::Game {
            return;
        }
        self.input_main();
        self.player_collide_misc();

        self.up_drill();

        self.up_rain_col();
        self.up_drones();
        self.up_flies();
        self.up_sliders();
        self.up_seekers();
        self.up_bombers();
        self.up_powerup();
        self.up_gold();
        self.up_world();

        // Powerup frames countdown
        self.powerup_frames = self.powerup_frames.saturating_sub(1);
        if self.powerup_frames == 0 {
            self.powerup_cur = PowerUp::None;
        }

        // Check for game over
        if self.hp == 0 && !INVINCIBLE {
            self.gameover_acc += 1;
            if self.gameover_acc > 120 {
                self.gameover_acc = 120;
                self.screen_set(Screen::GameOver);
            }
        }
    }

    fn up_sc_transition(&mut self) {
        if self.screen != Screen::Transition {
            return;
        }
        if self.input_check_any() {
            self.screen_set(Screen::Game);
        }
    }

    fn up_sc_shop(&mut self) {
        if self.screen != Screen::Shop {
            return;
        }
        if self.purchased > 0 && self.input_check_any() {
            self.purchased = 0;
            self.no_input_frames = NO_INPUT_FRAMES_SH;
        }
        if self.input_check(BUTTON_UP) {
            // Buy heart piece
            self.no_input_frames = NO_INPUT_FRAMES_SH;
            if self.gold >= self.cost_heart && self.hp < MAX_HP {
                self.gold = self.gold.saturating_sub(self.cost_heart);
                self.hp += 1;
                self.purchased = 1;
                self.sfx_ok();
            } else {
                self.sfx_deny();
                self.dmg_frames = DMG_FRAMES;
            }
        } else if self.input_check(BUTTON_LEFT) {
            // Buy drill speed
            self.no_input_frames = NO_INPUT_FRAMES_SH;
            if self.gold >= self.cost_drill_speed && self.drill_speed < 128 {
                self.gold = self.gold.saturating_sub(self.cost_drill_speed);
                self.drill_speed += 8;
                self.purchased = 2;
                self.sfx_ok();
            } else {
                self.sfx_deny();
                self.dmg_frames = DMG_FRAMES;
            }
        } else if self.input_check(BUTTON_RIGHT) {
            // Buy drill cooling
            self.no_input_frames = NO_INPUT_FRAMES_SH;
            if self.gold >= self.cost_drill_cool && self.drill_heat_max < 1024 {
                self.gold = self.gold.saturating_sub(self.cost_drill_cool);
                self.drill_heat_max += 64;
                self.purchased = 3;
                self.sfx_ok();
            } else {
                self.sfx_deny();
                self.dmg_frames = DMG_FRAMES;
            }
        } else if self.input_check(BUTTON_DOWN) {
            self.screen_set(Screen::Transition);
        }
        self.up_rain_pos(100, 80, RAIN_MAX / 2, 2);
    }

    fn up_sc_gameover(&mut self) {
        if self.screen != Screen::GameOver {
            return;
        }
        if self.input_check_any() {
            *self = GameMaster::new();
            self.no_input_frames = NO_INPUT_FRAMES;
            return;
        }
    }

    fn colors_set(&mut self, c: u16) {
        unsafe { *DRAW_COLORS = c };
    }

    fn color_flash(&mut self, ca: u16, cb: u16, duration: u32) {
        self.colors_set(ca);
        if self.frame % duration < duration / 2 {
            self.colors_set(cb);
        }
    }

    fn palette_set(&mut self, pal: [u32; 4]) {
        unsafe {
            *PALETTE = pal;
        }
    }

    // Only works on 8x8 sprites
    fn sprite_frame(&self, fps: u8, frames: Vec<[u8; 8]>) -> [u8; 8] {
        let frame_index = (self.frame / (60 / fps as u32)) % (frames.len() as u32);
        frames[frame_index as usize]
    }

    fn render_logo_acid(&mut self, x: i32, y: i32) {
        blit(&LOGO_A, x, y, 16, 16, BLIT_1BPP);
        blit(&LOGO_C, x + 16 * 1, y, 16, 16, BLIT_1BPP);
        blit(&LOGO_I, x + 16 * 2, y, 16, 16, BLIT_1BPP);
        blit(&LOGO_D, x + 16 * 3, y, 16, 16, BLIT_1BPP);
    }
    fn render_logo_rain(&mut self, x: i32, y: i32) {
        blit(&LOGO_R, x, y, 16, 16, BLIT_1BPP);
        blit(&LOGO_A, x + 16 * 1, y, 16, 16, BLIT_1BPP);
        blit(&LOGO_I, x + 16 * 2, y, 16, 16, BLIT_1BPP);
        blit(&LOGO_N, x + 16 * 3, y, 16, 16, BLIT_1BPP);
    }

    fn render_rain(&mut self) {
        for i in 0..self.rain_locs.len() {
            self.colors_set(2);
            if self.rng.i32(0..2) == 0 {
                self.colors_set(3);
            }
            let rain = &self.rain_locs[i];
            rect(rain.x as i32, rain.y as i32, 1, 2);
        }
    }

    fn render_no_input(&mut self) {
        if self.no_input_frames > 0 {
            self.colors_set(2);
            rect(0, 0, 160, 2);
            rect(0, 158, 160, 2);
            rect(0, 0, 2, 160);
            rect(158, 0, 2, 160);
            self.no_input_frames -= 1;
        }
    }

    fn render_sc_intro(&mut self) {
        if self.screen != Screen::Intro {
            return;
        }
        self.colors_set(1);
        rect(0, 0, 160, 160);
        self.colors_set(3);
        text(INTRO_TEXT, 4, 48 - (self.frame as i32 / 8));
        self.colors_set(1);
        // rect(0, 0, 160, 32);
        // Sine
        for i in 0..160 {
            let sina = (self.frame as f32 / 320.).sin() * 2.0;
            let sin = ((self.frame as f32 / 16.) + (i as f32 / (4. + sina))).sin();
            let y = (sin * 8.0 + 130.0) as i32;
            self.colors_set((i + (self.frame as u16 / 32)) % 2 + 1);
            rect(i as i32, 0, 1, (160 - y) as u32);
        }
        self.render_rain();
        self.colors_set(3);
        self.render_logo_acid(14, 4);
        self.render_logo_rain(84, 4);
        self.colors_set(4);
        self.render_logo_acid(14, 6);
        self.render_logo_rain(84, 6);

        self.colors_set(2);
        rect(0, 144, 160, 16);
        self.colors_set(1);
        rect(2, 146, 156, 12);
        self.colors_set(2);
        if self.frame < 256 {
            text("WASM4 RUST GPLv3", 18, 148);
        } else {
            text("PRESS ANY BUTTON", 18, 148);
        }
    }

    fn render_sc_start(&mut self) {
        if self.screen != Screen::Start {
            return;
        }

        self.colors_set(1);
        rect(0, 0, 160, 160);

        // BG acid rain text
        for x in 0..5 as i32 {
            for y in 0..8 {
                let c = (x as u32 + y as u32 * self.frame / 100) as u16 % 3 + 0;
                self.colors_set(c);
                text("ACID", x * 32, y * 20);
                let c2 = (x as u32 + (y as u32 * 3) * self.frame / 128) as u16 % 3 + 0;
                self.colors_set(c2);
                text("RAIN", x * 32, 10 + y * 20);
            }
        }
        self.colors_set(1);
        for x in 0..5 as i32 {
            for y in 0..8 {
                text("ACID", 1 + x * 32, 1 + y * 20);
                text("RAIN", 1 + x * 32, 1 + 10 + y * 20);
            }
        }
        // Scanlines
        if self.frame % 512 > 20 {
            for y in 0..80 {
                hline(0, y * 2, 160);
            }
        }

        self.render_rain();

        // Sine
        for i in 0..160 {
            let sina = (self.frame as f32 / 320.).sin() * 2.0;
            let sin = ((self.frame as f32 / 10.) + (i as f32 / (4. + sina))).sin();
            let y = (sin * 8.0 + 140.0) as i32;
            self.colors_set(2);
            rect(i as i32, y, 1, (160 - y) as u32);
        }
        self.colors_set(2);
        text("MATHIEU/\nDOMBROCK\n2025////", 12, 50);
        self.colors_set(3);
        if (self.frame / 32) % 2 == 0 {
            self.colors_set(4);
        }
        let sx = ((self.frame as f32 / 8.).sin() * 2.0) as i32;
        text(b"PRESS \x80 TO START", 17 + sx, 105);
        self.colors_set(3);
        let x = 10;
        let y = 10;
        self.render_logo_acid(x, y);
        self.render_logo_rain(x, y + 18);
        let x = 12;
        let y = 12;
        self.colors_set(4);
        self.render_logo_acid(x, y);
        self.render_logo_rain(x, y + 18);

        // Options
        let diff_strs: [&str; MAX_DIFF as usize] = [
            "BABY", "EASY", "MEDIUM", "HARD", "WILD", "OHNO!", "HECK", "HELL",
        ];
        let diff_str = diff_strs[self.difficulty as usize];
        let drill_str = if self.auto_drill { "AUTO" } else { "MANUAL" };
        let mode_str = "ARCADE";
        self.colors_set(3);
        text(b"\x86LVL", 95, 12);
        self.colors_set(4);
        text(diff_str, 104, 22);
        self.colors_set(3);
        text(b"\x87DRILL", 95, 32);
        self.colors_set(4);
        text(drill_str, 104, 42);
        self.colors_set(3);
        text(b"\x85MODE", 95, 52);
        self.colors_set(4);
        text(mode_str, 104, 62);
        //
        self.colors_set(1);
        text("GPLv3        v1.0", 13, 150);
    }

    fn render_sc_shop(&mut self) {
        if self.screen == Screen::Shop {
            self.colors_set(1);
            rect(0, 0, 160, 160);
            self.colors_set(3);
            rect(0, 0, 160, 80);
            self.colors_set(1);
            // Sine
            for x in 0..160 {
                let sina = (self.frame as f32 / 320.).sin() * 2.0;
                let sin = ((self.frame as f32 / 10.) + (x as f32 / (4. + sina))).sin();
                let y = (sin * 8.0 + 32.0) as i32;
                rect(x as i32, y, 1, (160 - y) as u32);
            }
            self.render_rain();
            self.colors_set(2);
            let sy = (self.frame as f32 / 8.).sin() * 2.0;
            text("UPGRADES!", 48, 6 + sy as i32);
            self.colors_set(4);
            text("UPGRADES!", 49, 7 + sy as i32);
            self.colors_set(2);
            self.render_gold_text(49, 14 + sy as i32, self.gold);
            self.colors_set(4);
            self.render_gold_text(50, 15 + sy as i32, self.gold);
            self.colors_set(3);
            vline(115, 45, 80);
            // Up
            text(b"\x86HEART PIECE", 15, 50);
            self.render_gold_text(120, 50, self.cost_heart);
            text(format!("{}/8", self.hp), 24, 60);
            // Left
            text(b"\x84DRILL SPEED", 15, 80);
            self.render_gold_text(120, 80, self.cost_drill_speed);
            text(format!("{}/128", self.drill_speed), 24, 90);
            // Righ
            text(b"\x85DRILL COOLR", 15, 110);
            self.render_gold_text(120, 110, self.cost_drill_cool);
            text(format!("{}/1024", self.drill_heat_max), 24, 120);
            // Down
            self.colors_set(4);
            hline(0, 135, 160);
            text(b"\x87NEXT  LEVEL", 33, 145);

            // Purchased
            if self.purchased > 0 {
                self.colors_set(1);
                rect(0, 45, 160, 120);
                self.colors_set(4);
                text("PURCHASED!", 42, 60);
                let mut pur_string = String::new();
                let mut amt_string = String::new();
                match self.purchased {
                    1 => {
                        pur_string = "HEART PIECE".to_string();
                        amt_string = format!("{}/{}", self.hp, 8);
                    }
                    2 => {
                        pur_string = "DRILL SPEED".to_string();
                        amt_string = format!("{}/{}", self.drill_speed, 128);
                    }
                    3 => {
                        pur_string = "DRILL COOLR".to_string();
                        amt_string = format!("{}/{}", self.drill_heat_max, 1024);
                    }
                    _ => {}
                }
                self.colors_set(3);
                text(pur_string, 38, 80);
                self.colors_set(4);
                text(amt_string, 38, 90);
                self.colors_set(3);
                for x in 0..160 {
                    let sina = -(self.frame as f32 / 320.).sin() * 2.0;
                    let sin = ((self.frame as f32 / 10.) + (x as f32 / (4. + sina))).sin();
                    let y = (sin * 8.0 + 32.0) as i32;
                    rect(x as i32, y + 100, 1, (160 - y) as u32);
                }
            }
        }
    }

    fn render_sc_transition(&mut self) {
        if self.screen != Screen::Transition {
            return;
        }
        self.colors_set(1);
        rect(0, 0, 160, 160);

        self.colors_set(4);
        text(format!("DAY {}", self.lvl), 62, 20);
        hline(62, 30, 60);
        self.colors_set(2);
        text(self.cur_lvl_data.text, 62, 40);

        self.colors_set(3);
        text("WEATHER", 62, 90);
        hline(62, 100, 60);
        self.colors_set(2);
        text(format!("WIND: {}>>", self.wind_speed), 62, 110);
        text(
            format!("ACID: {}%", self.cur_lvl_data.rain_acidity),
            62,
            120,
        );
        let mut rain_v =
            (self.cur_lvl_data.rain_chance_rte + self.cur_lvl_data.rain_amount_rte) / 20;
        rain_v = 100 - rain_v;
        text(format!("RAIN: {}%", rain_v), 62, 130);

        // TODO: This is a bit CPU intensive
        // Sine
        self.colors_set(2);
        for i in 0..160 {
            let sina = (self.frame as f32 / 320.).sin() * 2.0;
            let sin = ((self.frame as f32 / 16.) + (i as f32 / (4. + sina))).sin();
            let y = (sin * 4.0 + 125.0) as i32;
            rect(0, i as i32, (160 - y) as u32, 1);
        }
        // Sine
        for i in 0..160 {
            let sina = (self.frame as f32 / 320.).sin() * 2.0;
            let sin = ((self.frame as f32 / 16.) + (i as f32 / (4. + sina))).sin();
            let z = (sin * 2.0 + 155.0) as i32;
            self.colors_set((i + (self.frame as u16 / 32)) % 2 + 1);
            rect(i as i32, 0, 1, (160 - z) as u32);
            rect(160 - i as i32, z, 1, (160 - z) as u32);
            rect(0, i as i32, (160 - z) as u32, 1);
            rect(z, 160 - i as i32, 10, 1);
        }
    }

    fn render_sc_gameover(&mut self) {
        if self.screen != Screen::GameOver {
            return;
        }
        let won = self.lvl == MAX_LVL - 1;
        self.palette_set(PAL_DMG);
        self.colors_set(1);
        rect(0, 0, 160, 160);
        self.colors_set(4);
        let over_text = if won { "YOU   WIN" } else { "GAME OVER" };
        self.colors_set(2);
        text(over_text, 45, 60);
        self.colors_set(4);
        text(over_text, 46, 61);
        self.colors_set(4);
        self.render_gold_text(50, 80, self.gold);
    }

    fn render_sc_main(&mut self) {
        if self.screen != Screen::Game {
            return;
        }
        // Render the world
        self.colors_set(3);
        for y in 0..WORLD_SIZE {
            for x in 0..WORLD_SIZE {
                if let Some(cell) = self.world_get(x, y) {
                    if cell {
                        self.colors_set(2);
                        rect(x as i32, y as i32, 1, 1);
                    }
                }
            }
        }

        // Highlight the top layer of blocks
        // TODO: This is probablly slow
        // Really only want to highlight visible to sky
        for x in 0..WORLD_SIZE {
            for y in 0..WORLD_SIZE {
                if let Some(cell) = self.world_get(x, y) {
                    if cell {
                        // Check if block above is empty
                        if y == 0 || self.world_get(x, y - 1) == Some(false) {
                            self.colors_set(3);
                            if self.rng.i32(0..4) == 0 {
                                self.colors_set(4);
                            }
                            rect(x as i32, y as i32, 1, 1);
                        }
                    }
                }
            }
        }

        // Health
        for i in 0..self.hp {
            self.colors_set(3);
            // rect(45 + i as i32 * 6, 4, 4, 4);
            blit(&HEART, 76 + i as i32 * 10, 2, 8, 8, BLIT_1BPP);
        }
        // Gold collected
        // text(self.gold.to_string(), 4, 2);
        self.render_gold_text(4, 2, self.gold);

        // Heat bar
        self.colors_set(2);
        let heat_bar_width = 80;
        let heat_width = (self.drill_heat as u32 * heat_bar_width) / (self.drill_heat_max as u32);
        rect(76, 12, heat_bar_width, 4);
        self.colors_set(3);
        if self.drill_overheat {
            self.color_flash(2, 3, 16);
        }
        rect(76, 12, heat_width, 4);

        // Powerups UI
        if self.powerup_frames > 1 {
            if self.powerup_frames % 20 < 10 {
                self.colors_set(2);
            } else {
                self.colors_set(3);
            }
            let pu_text = match self.powerup_cur {
                PowerUp::SuperDrill => "SDRILL",
                PowerUp::Invincible => "INVINC",
                PowerUp::Magnet => "MAGNET",
                PowerUp::None => "",
            };
            text(pu_text, 4, 12);
        }

        // Render invincibility overlay
        if self.powerup_cur == PowerUp::Invincible && self.powerup_frames > 0 {
            self.colors_set(4);
            oval(
                self.player_pos.x as i32 - 4,
                self.player_pos.y as i32 - 4,
                PLAYER_SIZE as u32 + 8,
                PLAYER_SIZE as u32 + 8,
            );
            self.colors_set(1);
            oval(
                self.player_pos.x as i32 - 4,
                self.player_pos.y as i32 - 4,
                PLAYER_SIZE as u32 + 8,
                PLAYER_SIZE as u32 + 8,
            );
        }

        // Render gold locations
        let gold_sprite = self.sprite_frame(6, vec![GOLD1, GOLD2]);
        self.colors_set(3);
        for i in 0..self.gold_locs.len() {
            self.colors_set(3);
            // if (self.frame / 16) % self.gold_locs.len() as u32 == i as u32 {
            //     self.colors_set(4);
            // }
            if self.player_pos.distance(&self.gold_locs[i]) < 48. {
                self.color_flash(3, 4, 20);
            }
            let gold = &self.gold_locs[i];
            blit(&gold_sprite, gold.x as i32, gold.y as i32, 8, 4, BLIT_1BPP);
        }

        // Render exit
        let door_sprite = self.sprite_frame(6, vec![DOOR1, DOOR2]);
        self.colors_set(1);
        rect(self.door_loc.x as i32, self.door_loc.y as i32, 8, 8);
        self.colors_set(4);
        if self.door_timer > 0 {
            self.color_flash(4, 2, 10);
        }
        blit(
            &door_sprite,
            self.door_loc.x as i32,
            self.door_loc.y as i32,
            8,
            8,
            BLIT_1BPP,
        );

        // Render powerups
        if !self.powerup_taken {
            let powerup_sprite =
                self.sprite_frame(6, vec![PU1, PU1, PU1, PU1, PU1, PU1, PU1, PU1, PU1, PU2]);
            let mut powerup_x = self.powerup_loc.x as i32;
            match (self.frame / 20) % 8 {
                6 => powerup_x += 1,
                7 => powerup_x -= 1,
                _ => {}
            }
            self.colors_set(4);
            blit(
                &powerup_sprite,
                powerup_x,
                self.powerup_loc.y as i32,
                8,
                8,
                BLIT_1BPP,
            );
        }

        // Render rain
        self.render_rain();
        // Render drones
        let drone_frame = (self.frame / 10) % 2;
        let drone_sprite = if drone_frame == 0 { &DRONE1 } else { &DRONE2 };
        self.colors_set(4);
        for drone in &self.drone_locs {
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
        let fly_sprite = self.sprite_frame(6, vec![FLY1, FLY2]);
        for i in 0..self.fly_locs.len() {
            let fly = &self.fly_locs[i].clone();
            self.colors_set(4);
            blit(&fly_sprite, fly.x as i32, fly.y as i32, 8, 8, BLIT_1BPP);
            if fly.y < 16 {
                self.color_flash(4, 2, 20);
                blit(&EXC, fly.x as i32, 150, 8, 8, BLIT_1BPP);
            }
        }

        // Render sliders
        let slider_sprite = self.sprite_frame(6, vec![SLIDER1, SLIDER2]);
        self.colors_set(4);
        for slider in &self.slider_locs {
            // rect(slider.x as i32, slider.y as i32, 6, 4);
            blit(
                &slider_sprite,
                slider.x as i32,
                slider.y as i32,
                8,
                8,
                BLIT_1BPP,
            );
        }

        // Render seekers
        let seeker_sprite = self.sprite_frame(6, vec![SEEKER1, SEEKER2]);
        self.colors_set(4);
        for seeker in &self.seeker_locs {
            // rect(seeker.x as i32, seeker.y as i32, 6, 4);
            blit(
                &seeker_sprite,
                seeker.x as i32,
                seeker.y as i32,
                8,
                8,
                BLIT_1BPP,
            );
        }

        // Render bombers
        let bomber_sprite = self.sprite_frame(6, vec![BOMBER1, BOMBER2]);
        let bomber_locs: Vec<_> = self.bomber_locs.iter().cloned().collect();
        for (i, bomber) in bomber_locs.iter().enumerate() {
            self.colors_set(4);
            if self.bomber_times[i] > 0 {
                self.color_flash(4, 3, 32 + self.bomber_times[i] as u32 * 8);
            }
            blit(
                &bomber_sprite,
                bomber.x as i32,
                bomber.y as i32,
                8,
                8,
                BLIT_1BPP,
            );
        }

        // Render player
        self.colors_set(4);
        let player_flags = match self.dir {
            0 => self.player_flags_last,
            1 => BLIT_1BPP | BLIT_FLIP_X,
            2 => BLIT_1BPP,
            _ => self.player_flags_last,
        };
        self.player_flags_last = player_flags;
        let mut player_sprite = self.sprite_frame(12, vec![SMILEY1, SMILEY2, SMILEY3]);
        if self.dir == 0 {
            player_sprite = SMILEY1;
        }
        if self.hp == 0 {
            player_sprite = SMILEYDEAD;
        }
        blit(
            &player_sprite,
            self.player_pos.x as i32,
            self.player_pos.y as i32,
            8,
            PLAYER_SIZE as u32,
            player_flags,
        );

        // Render drill
        let drill_off = match self.dir {
            0 => Pos::new(PLAYER_SIZE as i16, 0),
            1 => Pos::new(-(PLAYER_SIZE as i16), 0),
            2 => Pos::new(PLAYER_SIZE as i16, 0),
            3 => Pos::new(0, PLAYER_SIZE as i16),
            4 => Pos::new(-8, PLAYER_SIZE as i16),
            5 => Pos::new(PLAYER_SIZE as i16, PLAYER_SIZE as i16),
            6 => Pos::new(0, -(PLAYER_SIZE as i16)),
            7 => Pos::new(-(PLAYER_SIZE as i16), -(PLAYER_SIZE as i16)),
            8 => Pos::new(PLAYER_SIZE as i16, -(PLAYER_SIZE as i16)),
            _ => Pos::new(PLAYER_SIZE as i16, 0),
        };
        let drill_flags = match self.dir {
            0 => BLIT_1BPP,
            1 => BLIT_1BPP | BLIT_FLIP_X,
            2 => BLIT_1BPP,
            3 => BLIT_1BPP | BLIT_FLIP_Y | BLIT_FLIP_X | BLIT_ROTATE,
            4 => BLIT_1BPP | BLIT_FLIP_X,
            5 => BLIT_1BPP,
            6 => BLIT_1BPP | BLIT_ROTATE,
            7 => BLIT_1BPP | BLIT_ROTATE | BLIT_FLIP_Y,
            8 => BLIT_1BPP | BLIT_ROTATE,
            _ => BLIT_1BPP,
        };
        let drill_show = match self.dir {
            0 => false,
            _ => true,
        };
        if drill_show && self.is_drilling {
            // let mut drill_sprite = if drill_frame == 0 { &DRILL1 } else { &DRILL2 };
            let drill_sprite_n = self.sprite_frame(12, vec![DRILL1, DRILL2]);
            let drill_sprite_d = self.sprite_frame(12, vec![DRILLD1, DRILLD2]);
            let drill_sprite = match self.dir {
                0..4 => drill_sprite_n,
                4..6 => drill_sprite_d,
                6 => drill_sprite_n,
                7 => drill_sprite_d,
                8 => drill_sprite_d,
                _ => drill_sprite_n,
            };
            blit(
                &drill_sprite,
                (self.player_pos.x + drill_off.x) as i32,
                (self.player_pos.y + drill_off.y) as i32,
                8,
                PLAYER_SIZE as u32,
                drill_flags,
            );
        }
        // Help text
        if self.has_drilled == false && self.lvl == 1 {
            self.colors_set(1);
            rect(50, 50, 60, 14);
            self.colors_set(4);
            if self.frame % 32 < 16 {
                self.colors_set(2);
            }
            let help_text = b"\x84\x85\x87+\x80";
            text(help_text, 60, 53);
        }

        // Damage text
        if self.dmg_frames > 0 {
            self.colors_set(1);
            text(
                &format!(" {:^8} ", self.last_dmg_from),
                self.player_pos.x as i32 - 34,
                self.player_pos.y as i32 - 10,
            );
            self.colors_set(4);
            text(
                &format!(" {:^8} ", self.last_dmg_from),
                self.player_pos.x as i32 - 33,
                self.player_pos.y as i32 - 9,
            );
        }
    }

    fn start(&mut self) {
        self.palette_set(PALS[self.pal_index as usize]);
        self.world = MiniBitVec::new();
    }

    // TODO: Frame inc can happen everywhere?
    fn update(&mut self) {
        self.up_sc_intro();
        self.up_sc_start();
        self.up_sc_main();
        self.up_sc_gameover();
        self.up_sc_shop();
        self.up_sc_transition();
        self.up_music();
        self.frame += 1;
        // No input frames countdown
        self.no_input_frames = self.no_input_frames.saturating_sub(1);

        // Damage frames countdown
        // Must happen in main update since we use dmg_frames for palette change
        self.dmg_frames = self.dmg_frames.saturating_sub(1);
        // Always run palette change first
        if self.dmg_frames > 0 {
            self.palette_set(PAL_DMG);
        } else if self.gameover_acc > 0 {
            self.palette_set(PAL_DMG);
        } else {
            self.palette_set(PALS[self.pal_index as usize]);
        }

        // DRAW
        self.render_sc_intro();
        self.render_sc_main();
        // NOTE: Other screens only render if active
        self.render_sc_start();
        self.render_sc_gameover();
        self.render_sc_shop();
        self.render_sc_transition();
        // No input overlay
        self.render_no_input();
        // Debug
        if GRID {
            self.colors_set(4);
            if self.frame % 32 < 16 {
                self.colors_set(0);
            }
            let grid = 16;
            for x in 0..WORLD_SIZE / grid {
                vline((x * grid) as i32, 0, WORLD_SIZE as u32);
            }
            for y in 0..WORLD_SIZE / grid {
                hline(0, (y * grid) as i32, WORLD_SIZE as u32);
            }
        }
    }
}

static mut GM: LazyLock<GameMaster> = LazyLock::new(|| GameMaster::new());

#[no_mangle]
#[allow(static_mut_refs)]
fn start() {
    unsafe {
        GM.start();
    }
}

#[no_mangle]
#[allow(static_mut_refs)]
fn update() {
    unsafe { GM.update() };
}
