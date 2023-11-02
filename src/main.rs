#![windows_subsystem = "windows"]

use std::rc::Rc;

use raylib::prelude::*;

mod cpu;
use cpu::CPU;
mod render;
use render::Render;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(64*16, 32*16)
        .title("emulateur chip 8")
        .build();
    rl.set_target_fps(60);
    let rend = Rc::new(Render::new());

    let mut cpu = CPU::new(10, "program.ch8", rend);
     
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        cpu.cycle(&mut d);
    }
}