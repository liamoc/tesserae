use tesserae::{Graphic,TileSet,Tile};
use super::swatch::Swatch;
use super::colors;
use std::ops::IndexMut;
use sdl2::render::{Texture,TextureCreator};
use sdl2::video::WindowContext;

pub struct ColorChooser<'r> {
    graphic:Graphic<Texture<'r>>,
    swatch:Swatch,
    current_color:usize,
}
impl <'r> ColorChooser<'r> {
    pub fn create(swatch : Swatch, texture_creator: &'r TextureCreator<WindowContext>) -> ColorChooser<'r> {
        let swatch_graphic = Graphic::blank(16, 16).textured(texture_creator);
        ColorChooser {
            graphic:swatch_graphic,
            swatch:swatch,
            current_color:0
        }
    }
    pub fn selected(&self) -> usize {
        self.current_color
    }
    pub fn swatch(&self) -> &Swatch {
        &self.swatch
    }
    pub fn swatch_mut(&mut self) -> &mut Swatch {
        &mut self.swatch
    }
    pub fn graphic(&self) -> &Graphic<Texture<'r>> {
        &self.graphic
    }
    pub fn refresh(&mut self, tile_set : &TileSet) {
        let mut i = 0;
        for y in 0..self.graphic.height() {
            for x in 0..self.graphic.width() {
                let c = self.swatch[i];
                let t = Tile { index: if i == self.current_color { 254 } else { 0 }, fg: colors::inverse(c,255), bg: c };                
                self.graphic.set_tile(x, y, t);
                i += 1
            }
        }
        self.graphic.update_texture(tile_set);
    }
    pub fn move_selected(&mut self, delta: i32) {
        let new = ((self.current_color as i32 + delta).max(0).min((self.swatch.len()-1) as i32)) as usize;
        self.current_color = new;
    }
}

pub struct TileSetChooser<'r> {
    graphic: Graphic<Texture<'r>>,
    current_tile:usize,
    tile_set: TileSet
}
impl <'r> TileSetChooser<'r> {
    pub fn create(tile_set : TileSet, texture_creator: &'r TextureCreator<WindowContext>) -> TileSetChooser<'r> {
        let tileset_graphic = Graphic::blank(16, 32).textured(texture_creator);
        TileSetChooser {
            graphic:tileset_graphic,
            tile_set: tile_set,
            current_tile:0
        }
    }
    pub fn selected_data_mut(&mut self) -> &mut u64 {
        self.tile_set.index_mut(self.current_tile)
    }
    pub fn selected_data(&self) -> u64 {
        self.tile_set[self.current_tile]
    }
    pub fn selected(&self) -> usize {
        self.current_tile
    }
    pub fn tile_set(&self) -> &TileSet {
        &self.tile_set
    }
    pub fn tile_set_mut(&mut self) -> &mut TileSet {
        &mut self.tile_set
    }
    pub fn graphic(&self) -> &Graphic<Texture<'r>> {
        &self.graphic
    }
    pub fn refresh(&mut self) {
        let mut i = 0;
        for y in 0..self.graphic.height() {
            for x in 0..self.graphic.width() {
                let fg = if i == self.current_tile { colors::CYAN } else { colors::WHITE };
                let bg = if i == self.current_tile { colors::TEAL } else { colors::BLACK };
                
                let t = Tile { index: i, fg: fg, bg: bg};
                self.graphic.set_tile(x, y, t);
                i += 1
            }
        }
        self.graphic.update_texture(&self.tile_set);
    }
    pub fn set_selected(&mut self, new: usize) {
        let new_clamped = new.min(self.tile_set.len()-1);
        self.current_tile = new_clamped;
    }
    pub fn move_selected(&mut self, delta: i32) {
        let new = ((self.current_tile as i32 + delta).max(0).min((self.tile_set.len()-1) as i32)) as usize;
        self.current_tile = new;
    }

}
