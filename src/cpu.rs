// #![allow(dead_code)]

use raylib::{prelude::*, ffi::{rand, IsKeyDown, GetKeyPressed}};
use std::collections::LinkedList;
use std::rc::Rc;

use crate::render;


pub struct CPU {
    memoire: [u8; 0x1000],
    pc: u16,
    i: u16,
    stack: LinkedList<u16>,
    delay_timer: u8,
    sound_timer: u8,
    registres: [u8; 0x10],
    pause: bool,
    key_reg: usize,
    speed: usize,
    render: Rc<render::Render>,
}

impl CPU {
    pub fn new(speed: usize, rom: &str, render: Rc<render::Render>) -> Self {
        let mut tmp = Self { 
            memoire: [0; 4096],
            pc: 0x200,
            i: 0,
            stack: LinkedList::new(),
            delay_timer: 0,
            sound_timer: 0,
            registres: [0; 0x10],
            pause: false,
            key_reg: 0,
            speed: speed,
            render: render
        };
        tmp.load_fonts();
        tmp.load_rom(rom);
        tmp
    }

    fn load_fonts(&mut self) {
        let fonts = vec![
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
		    0x20, 0x60, 0x20, 0x20, 0x70, // 1
		    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
		    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
		    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
		    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
		    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
		    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
		    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
		    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
		    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
		    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
		    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
		    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
		    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
		    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        for i in 0..fonts.len() {
            self.memoire[i] = fonts[i];
        }
    }

    fn load_rom(&mut self, rom: &str) {
        let bytes = std::fs::read(rom).unwrap_or(vec![0x12, 0x00]);
        for i in 0..bytes.len() {
            self.memoire[i + 0x200] = bytes[i];
        }
    }

    pub fn cycle(&mut self, d: &mut RaylibDrawHandle) {

        for _ in 0..self.speed {
            if self.pause {
                break;
            }
            let opcode = (self.memoire[self.pc as usize] as u16) << 8 | self.memoire[self.pc as usize + 1] as u16;
            self.execute(opcode);
        }

        if !self.pause {
            self.update_timer();
        }

        if self.pause && unsafe { GetKeyPressed() } != 0 {
            self.registres[self.key_reg] = unsafe { translate_key_rev(GetKeyPressed())};
            self.pause = false;
        }

        self.render.affiche(d);
    }

    fn update_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn execute(&mut self, opcode: u16) {
        self.pc += 2;
        self.pc %= 0x1000;
        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let kk = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => Rc::get_mut(&mut self.render).unwrap().clear_render(),
            (0x0, 0x0, 0xE, 0xE) => {
                match self.stack.front() {
                    Some(expr) => {
                        self.pc = *expr;
                        self.stack.pop_front();
                    },
                    None => self.pc = 0x200,
                }
                
            },
            (0x1, _, _, _) => self.pc = nnn,
            (0x2, _, _, _) => {
                self.stack.push_front(self.pc);
                self.pc = nnn;
            }
            (0x3, _, _, _) => if self.registres[x] == kk {
                self.pc += 2;
            }
            (0x4, _, _, _) => if self.registres[x] != kk {
                self.pc += 2;
            }
            (0x5, _, _, _) => if self.registres[x] == self.registres[y] {
                self.pc += 2;
            }
            (0x6, _, _, _) => self.registres[x] = kk,
            (0x7, _, _, _) => self.registres[x] = self.registres[x].wrapping_add(kk),
            (0x8, _, _, 0x0) => self.registres[x] = self.registres[y],
            (0x8, _, _, 0x1) => self.registres[x] |= self.registres[y],
            (0x8, _, _, 0x2) => self.registres[x] &= self.registres[y],
            (0x8, _, _, 0x3) => self.registres[x] ^= self.registres[y],
            (0x8, _, _, 0x4) => {
                let sum = (self.registres[x] as u16).wrapping_add(self.registres[y] as u16);
                self.registres[x] = sum as u8;
                self.registres[0xF] = 0;
                if sum > 0xFF {
                    self.registres[0xF] = 1
                }
            }
            (0x8, _, _, 0x5) => {
                let under = self.registres[x] > self.registres[y];
                self.registres[x] = self.registres[x].wrapping_sub(self.registres[y]);
                self.registres[0xF] = 0;
                if under {
                    self.registres[0xF] = 1;
                }
            }
            (0x8, _, _, 0x6) => {
                self.registres[x] = self.registres[y];
                let tmp = self.registres[x] & 1;
                self.registres[x] >>= 1;
                self.registres[0xF] = tmp;
            }
            (0x8, _, _, 0x7) => {
                let under = self.registres[y] > self.registres[x];
                self.registres[x] = self.registres[y].wrapping_sub(self.registres[x]);
                self.registres[0xF] = 0;
                if under {
                    self.registres[0xF] = 1;
                }
            }
            (0x8, _, _, 0xE) => {
                self.registres[x] = self.registres[y];
                let tmp = self.registres[x] >> 7;
                self.registres[x] <<= 1;
                self.registres[0xF] = tmp;
            }
            (0x9, _, _, _) => if self.registres[x] != self.registres[y] {
                self.pc += 2;
            }
            (0xA, _, _, _) => self.i = nnn,
            (0xB, _, _, _) => self.pc = (self.registres[0] as u16) + nnn,
            (0xC, _, _, _) => self.registres[x] = unsafe { rand() as u8 } & kk,
            (0xD, _, _, _) => {
                let width = 8;
                let height = opcode & 0xF;
                self.registres[0xF] = 0;
                for row in 0..height {
                    let mut sprite = self.memoire[(self.i as usize) + (row as usize)];
                    for col in 0..width {
                        if (sprite & 0x80) > 0 {
                            let xpos = (self.registres[x] + col) as usize;
                            let ypos = (self.registres[y] + row as u8) as usize;
                            let pixel_res = Rc::get_mut(&mut self.render).unwrap().change_pixel(xpos, ypos);
                            if pixel_res {
                                self.registres[0xF] = 1;
                            }
                        }
                        sprite <<= 1;
                    }
                }
            }
            (0xE, _, 0x9, 0xE) => {
                let key = translate_key(self.registres[x]);
                if unsafe { IsKeyDown(key) } {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 0x1) => {
                let key = translate_key(self.registres[x]);
                if unsafe { !IsKeyDown(key) } {
                    self.pc += 2;
                }
            }
            (0xF, _, 0x0, 0x7) => self.registres[x] = self.delay_timer,
            (0xF, _, 0x0, 0xA) => {
                self.pause = true;
                self.key_reg = x;
            }
            (0xF, _, 0x1, 0x5) => self.delay_timer = self.registres[x],
            (0xF, _, 0x1, 0x8) => self.sound_timer = self.registres[x],
            (0xF, _, 0x1, 0xE) => self.i += self.registres[x] as u16,
            (0xF, _, 0x2, 0x9) => self.i = (self.registres[x] as u16) * 5,
            (0xF, _, 0x3, 0x3) => {
                self.memoire[self.i as usize] = self.registres[x] / 100;
                self.memoire[self.i as usize + 1] = (self.registres[x] % 100) / 10;
                self.memoire[self.i as usize + 2] = self.registres[x] % 10;
            }
            (0xF, _, 0x5, 0x5) => {
                for index in 0..=x {
                    self.memoire[self.i as usize + index] = self.registres[index];
                }
                self.i += 1;
            }
            (0xF, _, 0x6, 0x5) => {
                for index in 0..=x {
                    self.registres[index] = self.memoire[self.i as usize + index];
                }
                self.i += 1;
            }
            _ => {}
        }
    }
}

fn translate_key(key: u8) -> i32 {
    (match key {
        0x1 => KeyboardKey::KEY_ONE,
	    0x2 => KeyboardKey::KEY_TWO,
	    0x3 => KeyboardKey::KEY_THREE,
	    0xC => KeyboardKey::KEY_FOUR,
	    0x4 => KeyboardKey::KEY_Q,
	    0x5 => KeyboardKey::KEY_W,
	    0x6 => KeyboardKey::KEY_E,
	    0xD => KeyboardKey::KEY_R,
	    0x7 => KeyboardKey::KEY_A,
	    0x8 => KeyboardKey::KEY_S,
	    0x9 => KeyboardKey::KEY_D,
	    0xE => KeyboardKey::KEY_F,
	    0xA => KeyboardKey::KEY_Z,
	    0x0 => KeyboardKey::KEY_X,
	    0xB => KeyboardKey::KEY_C,
	    0xF => KeyboardKey::KEY_V,
        _ => panic!()
    }) as i32
}

fn key_to_i32(key: KeyboardKey) -> i32 {
    key as i32
}

fn translate_key_rev(key: i32) -> u8 {
    let _k1: i32 = key_to_i32(KeyboardKey::KEY_ONE);
    let _k2 = key_to_i32(KeyboardKey::KEY_TWO);
    let _k3 = key_to_i32(KeyboardKey::KEY_THREE);
    let _k4 = key_to_i32(KeyboardKey::KEY_FOUR);
    let _kq = key_to_i32(KeyboardKey::KEY_Q);
    let _kw = key_to_i32(KeyboardKey::KEY_W);
    let _ke = key_to_i32(KeyboardKey::KEY_E);
    let _kr= key_to_i32(KeyboardKey::KEY_R);
    let _ka = key_to_i32(KeyboardKey::KEY_A);
    let _ks = key_to_i32(KeyboardKey::KEY_S);
    let _kd = key_to_i32(KeyboardKey::KEY_D);
    let _kf= key_to_i32(KeyboardKey::KEY_F);
    let _kz = key_to_i32(KeyboardKey::KEY_Z);
    let _kx = key_to_i32(KeyboardKey::KEY_X);
    let _kc = key_to_i32(KeyboardKey::KEY_C);
    let _kv= key_to_i32(KeyboardKey::KEY_V);
    #[allow(unreachable_patterns)]
    match key {
        _k1 => 0x1,
        _k2 => 0x2,
        _k3 => 0x3,
        _k4 => 0xC,
        _kq => 0x4,
        _kw => 0x5,
        _ke => 0x6,
        _kr => 0xD,
        _ka => 0x7,
        _ks => 0x8,
        _kd => 0x9,
        _kf => 0xE,
        _kz => 0xA,
        _kx => 0x0,
        _kc => 0xB,
        _kv => 0xF,
        _ => panic!()
    }
}