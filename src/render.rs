use raylib::prelude::*;

pub struct Render {
    affichage: [[bool; 32]; 64],
}

impl Render {
    pub fn new() -> Self {
        Self { affichage: [[false; 32]; 64] }
    }
    pub fn change_pixel(&mut self, x: usize,y : usize) -> bool {
        let x = x % 64;
        let y = y % 32;
        self.affichage[x][y] = !self.affichage[x][y];
        !self.affichage[x][y]
    }

    pub fn clear_render(&mut self) {
        self.affichage = [[false; 32]; 64];
    }

    pub fn affiche(&self, d: &mut RaylibDrawHandle) {
        for x in 0..64 {
            for y in 0..32 {
                if self.affichage[x][y] {
                    d.draw_rectangle((x as i32)*16, (y as i32)*16, 16, 16, Color::WHITE);
                }
            }
        }
    }
}