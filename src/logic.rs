use macroquad::prelude::mouse_position;

pub enum Stages {
    ChessGame,
}

// A trait I made for integers jut to make my life easer
pub trait MyNumTrait {
    fn is_even(&self) -> bool;
    fn is_odd(&self) -> bool;
    fn distance(self, y: u8) -> u8;

}

impl MyNumTrait for u8 {
    #[inline(always)]
    fn is_even(&self) -> bool {
        // Checks the least significant bit for a 0, since if it's a zero, the integer is guaranteed to be even
        *self & 1 == 0
    }

    #[inline(always)]
    fn is_odd(&self) -> bool {
        !self.is_even()
    }

    #[inline(always)]
    fn distance(self, y: u8) -> u8 {
        match self < y {
            true => y - self,
            false => self - y,
        }
    }
}

pub trait GameStage {
    fn draw(&self);
    fn logic(&mut self); 
    fn set_new_stage(&mut self) -> Option<Stages>;

}

#[inline]
pub fn mouse_in_rectangle(coords: (f32, f32), size: (f32, f32)) -> bool {
    let mouse_pos = mouse_position();
    
    mouse_pos.0 > coords.0 && 
    mouse_pos.1 > coords.1 &&
    mouse_pos.0 < coords.0 + size.0 &&
    mouse_pos.1 < coords.1 + size.1 

}
