#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
use wasm4::*;

macro_rules! u {
    ($($body:tt)*) => {
        unsafe {
            $($body)*
        }
    };
}

#[rustfmt::skip]
const SMILEY: [u8; 8] = [
    0b11100111,
    0b11000011,
    0b01000010,
    0b00011000,
    0b10011001,
    0b00111100,
    0b00000000,
    0b01000010,
];

#[derive(Copy, Clone)]
struct Pos {
    x: i32,
    y: i32,
}
impl Pos {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

struct State {
    frame: u32,
    pos: Pos,
    pbul: Vec<Pos>,
}
impl State {
    fn pshoot(&mut self) {
        let mut pos = self.pos;
        pos.x += 3;
        self.pbul.push(pos);
    }
    fn update_pbul(&mut self) {
        // For each bullet
        for bullet in &mut self.pbul {
            bullet.y -= 2; // Move the bullet down
            rect(bullet.x, bullet.y, 2, 4);
        }
        self.pbul.retain(|b| b.y > 0);
    }
}
static mut ST: State = State {
    frame: 0,
    pos: Pos { x: 80, y: 130 },
    pbul: Vec::new(),
};

fn set_colors(c: u16) {
    u! { *DRAW_COLORS = c }
}
fn get_gamepad() -> u8 {
    u! { *GAMEPAD1 }
}
fn triangle(a: Pos, b: Pos, c: Pos) {
    line(a.x, a.y, b.x, b.y);
    line(b.x, b.y, c.x, c.y);
    line(c.x, c.y, a.x, a.y);
}

#[no_mangle]
fn start() {
    u! {
        *PALETTE = [0x000000, 0x555555, 0xAAAAAA, 0xFFFFFF];
    }
}

#[no_mangle]
#[allow(static_mut_refs)]
unsafe fn update() {
    // UPDATE
    if ST.frame % 2 == 0 {
        ST.pshoot();
    }
    ST.update_pbul();

    ST.frame += 1;
    // DRAW
    set_colors(2);

    let gamepad = get_gamepad();
    if gamepad & BUTTON_1 != 0 {
        set_colors(4);
    }
    if gamepad & BUTTON_LEFT != 0 {
        ST.pos.x -= 2
    }
    if gamepad & BUTTON_RIGHT != 0 {
        ST.pos.x += 2
    }

    blit(&SMILEY, ST.pos.x, ST.pos.y, 8, 8, BLIT_1BPP);
}
