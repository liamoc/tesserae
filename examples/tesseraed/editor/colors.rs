use sdl2::pixels::Color;

const fn rgba(r:u8,g:u8,b:u8,a:u8) -> Color {
    Color { r : r , g : g , b : b , a : a }
}

pub const BLACK       : Color = rgba(0  ,0  ,0  ,255);
pub const WHITE       : Color = rgba(255,255,255,255);
pub const GRAY        : Color = rgba(96 ,96 ,96 ,255);
pub const CYAN        : Color = rgba(0  ,255,255,255);
pub const TEAL        : Color = rgba(64 ,128,128,255);
pub const TRANSPARENT : Color = rgba(0  ,255,0  ,0  );
pub const PALE_YELLOW : Color = rgba(255,255,128,255);
pub const PALE_RED    : Color = rgba(255,128,128,255);
pub const YELLOW      : Color = rgba(255,255,0  ,255);
pub const PALE_GRAY   : Color = rgba(192,192,192,255);
pub const PALE_BLUE   : Color = rgba(32 ,196,255,255);
pub fn inverse(c : Color, a : u8) -> Color {
    rgba(255-c.r,255-c.g,255-c.b,a)
} 
