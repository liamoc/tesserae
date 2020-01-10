//! `tesserae` is a library for manipulating and drawing graphics consisting of 
//! 8x8 pixel two-color tiles, in the spirit of text mode drawing in early 8-bit
//! computers, using SDL.
//! 
//! It includes file formats for saving and loading tile sets and graphics,
//! as well as a number of built-in tile sets.
//! 
//! It also includes a tile and graphics editor, itself made with tesserae,
//! called `tesseraed`.
//! 
//! `tesseraed` is stored as an example in the library, and can be run with
//! `cargo run --example tesseraed`.

extern crate sdl2;
extern crate byteorder;

use std::fs::File;
use std::path::Path;
use std::io::{Cursor,Read,Write};
use std::io;
use std::ops::{Index,IndexMut};
use byteorder::{LittleEndian,ReadBytesExt,WriteBytesExt};

use sdl2::pixels::{Color,PixelFormatEnum};
use sdl2::render::{Texture,TextureCreator,BlendMode,Canvas, RenderTarget};
use sdl2::rect::{Point,Rect};

const TILESET_SIZE : usize = 512;

const CHAR_MAP : [usize;256] = create_character_map();
const fn create_character_map () -> [usize;256] {
    let mut c = [ 0 as usize; 256];
    c['A' as usize] = 2;
    c['B' as usize] = 3;
    c['C' as usize] = 4;
    c['D' as usize] = 5;
    c['E' as usize] = 6;
    c['F' as usize] = 7;
    c['G' as usize] = 8;
    c['H' as usize] = 9;
    c['I' as usize] = 10;
    c['J' as usize] = 11;
    c['K' as usize] = 12;
    c['L' as usize] = 13;
    c['M' as usize] = 14;
    c['N' as usize] = 15;
    c['O' as usize] = 16;
    c['P' as usize] = 17;
    c['Q' as usize] = 18;
    c['R' as usize] = 19;
    c['S' as usize] = 20;
    c['T' as usize] = 21;
    c['U' as usize] = 22;
    c['V' as usize] = 23;
    c['W' as usize] = 24;
    c['X' as usize] = 25;
    c['Y' as usize] = 26;
    c['Z' as usize] = 27;
    c['a' as usize] = 78;
    c['b' as usize] = 79;
    c['c' as usize] = 80;
    c['d' as usize] = 81;
    c['e' as usize] = 82;
    c['f' as usize] = 83;
    c['g' as usize] = 84;
    c['h' as usize] = 85;
    c['i' as usize] = 86;
    c['j' as usize] = 87;
    c['k' as usize] = 88;
    c['l' as usize] = 89;
    c['m' as usize] = 90;
    c['n' as usize] = 91;
    c['o' as usize] = 92;
    c['p' as usize] = 93;
    c['q' as usize] = 94;
    c['r' as usize] = 95;
    c['s' as usize] = 96;
    c['t' as usize] = 97;
    c['u' as usize] = 98;
    c['v' as usize] = 99;
    c['w' as usize] = 100;
    c['x' as usize] = 101;
    c['y' as usize] = 102;
    c['z' as usize] = 103;
    c['.' as usize] = 28;
    c['!' as usize] = 29;
    c['?' as usize] = 30;
    c['-' as usize] = 31;
    c[',' as usize] = 32;
    c['\'' as usize] = 33;
    c[':' as usize] = 34;
    c[';' as usize] = 35;
    c['_' as usize] = 36;
    c[')' as usize] = 37;
    c['(' as usize] = 38;
    c['/' as usize] = 39;
    c['\\' as usize] = 40;
    c[']' as usize] = 41;
    c['[' as usize] = 42;
    c['>' as usize] = 43;
    c['<' as usize] = 44;
    c['}' as usize] = 45;
    c['{' as usize] = 46;
    c['"' as usize] = 47;
    c['|' as usize] = 48;
    c['+' as usize] = 49;
    c['=' as usize] = 50;
    c['~' as usize] = 61;
    c['`' as usize] = 62;
    c['$' as usize] = 63;
    c['#' as usize] = 72;
    c['@' as usize] = 73;
    c['%' as usize] = 74;
    c['^' as usize] = 75;
    c['&' as usize] = 76;
    c['*' as usize] = 77;
    c['0' as usize] = 51;
    c['1' as usize] = 52;
    c['2' as usize] = 53;
    c['3' as usize] = 54;
    c['4' as usize] = 55;
    c['5' as usize] = 56;
    c['6' as usize] = 57;
    c['7' as usize] = 58;
    c['8' as usize] = 59;
    c['9' as usize] = 60;
    c
}

/// A set of 512 8x8 pixel monochrome tiles, along with a map for the basic 256 ASCII characters 
/// to tile indices. Most tile sets only include character mappings for the typable
/// characters on a conventional keyboard. 
/// 
/// If indexed with `usize`, gives the 64 bit integer corresponding to the tile data for that tile index.
/// Can also be mutated by assigning to a particular `usize` index.
///
/// Can also be indexed by `char`, which gives the tile index (a `usize`) corresponding to that particular character.
/// The character map can be changed by assigning to a particular `char` index.
#[derive(Clone)]
pub struct TileSet {
    data: Vec<u64>,
    char_map: [usize;256]
}

impl TileSet { 
    
    fn new() -> TileSet {
        TileSet {
            data: Vec::new(),
            char_map: CHAR_MAP
        }
    }
    /// Create a blank tile set with 512 tiles (all pixels off) and the default character map.
    pub fn blank() -> TileSet {
        let mut ts = TileSet::new();
        for _ in 0..TILESET_SIZE {
            ts.data.push(0x0000000000000000);
        }
        ts
    }
    /// Load a tile set from the file with the given path. Sugar for `load_from` with `File::open`.
    pub fn load_file<P: AsRef<Path>>(path : P) -> io::Result<TileSet> {
        let f = File::open(path)?;
        Ok(TileSet::load_from(f))
    }
    /// Load a tile set from a `Read` instance such as a file. Commonly used with `include_bin!` like so: 
    /// ```
    /// let ts = include_bytes!("tile_set");
    /// TileSet::load_from(Cursor::new(&ts[..]))
    /// ```
    pub fn load_from<R: Read>(mut input : R) -> TileSet {
        let mut ts = TileSet::new();
        let mut c = 0;
        while c < TILESET_SIZE {
            match input.read_u64::<LittleEndian>() {
                Ok(i) => { ts.data.push(i); c += 1 },
                Err(_) => break
            }            
        }
        while c < TILESET_SIZE {
            c += 1;
            ts.data.push(0);
        }
        c = 0;
        while c < 256 {
            match input.read_u16::<LittleEndian>() {
                Ok(i) => ts.char_map[c] = i as usize,
                Err(_) => break
            }
            c += 1;
        }
        ts
    }
    /// A built-in tile set used in, among other things, the tesseraed editor.
    pub fn default() -> TileSet {
        let ts = include_bytes!("../tile_set");
        TileSet::load_from(Cursor::new(&ts[..]))
    }
    /// A built-in tile set containing the 256 CGA standard ASCII characters using 
    /// the font used in Hercules graphics cards.
    pub fn cga_ascii() -> TileSet {
        let ts = include_bytes!("../cga");
        TileSet::load_from(Cursor::new(&ts[..]))
    }
    /// The regular, all-uppercase shifted PETscii tile set used in 
    /// Commodore PET and Commodore 64 machines.
    pub fn petscii() -> TileSet {
        let ts = include_bytes!("../petscii");
        TileSet::load_from(Cursor::new(&ts[..]))
    }
    /// The unshifted PETscii tile set which includes lower case letters 
    /// used in Commodore PET and Commodore 64 machines.
    pub fn petscii_unshifted() -> TileSet {
        let ts = include_bytes!("../petscii_unshifted");
        TileSet::load_from(Cursor::new(&ts[..]))
    }

    /// Save the tileset to a file at the provided path.
    pub fn store<P: AsRef<Path>>(&self,path : P ) -> io::Result<()> {
        let mut f = File::create(path)?;
        for i in &self.data {
            f.write_u64::<LittleEndian>(*i)?;
        }
        for i in 0..256 {
            f.write_u16::<LittleEndian>(self.char_map[i] as u16)?;
        }
        Ok(())
    }

    /// Should always return 512, but using this gives you future-proofing 
    /// in case the tile set size changes in future.
    pub fn len(&self) -> usize {
        self.data.len()
    }   
    
    fn draw_tile_to<P : Into<Point>>(&self, index: usize , tex: &mut Texture, point: P, fg: Color, bg: Color) {        
        draw_tile_data(self.data[index],tex,point, fg, bg)
    }
}
impl Index<char> for TileSet {
    type Output = usize;
    fn index(&self,index:char) -> &usize {
        &self.char_map[index as usize]
    }
}
impl IndexMut<char> for TileSet {
    fn index_mut(&mut self,index:char) -> &mut usize {
        &mut self.char_map[index as usize]
    }
}
impl Index<usize> for TileSet {
    type Output = u64 ;
    fn index(&self, index: usize) -> &u64 {
        &self.data[index]
    }
}
impl IndexMut<usize> for TileSet {
    fn index_mut(&mut self, index: usize) -> &mut u64 {
        &mut self.data[index]
    }
}

/// An index into a `TileSet`, paired with a foreground and background
/// colour. One cell of a `Graphic`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Tile {
    /// 0 ≤ index < 512, the maximum size of a tile set.
    pub index : usize,
    pub fg: Color,
    pub bg: Color
}
impl Default for Tile {
    fn default() -> Tile { Tile {index:0, fg: Color::RGBA(0,255,0,0), bg: Color::RGBA(0,255,0,0) } }
}


/// A `Graphic` is an image composed of many `Tile`s in a rectangle.
/// 
/// The type parameter `T` determines whether the Graphic maintains 
/// an SDL `Texture` for drawing the graphic to screen. A `Graphic<()>`
/// is a dummy graphic with no texture, useful only for graphics that 
/// don't get drawn to the screen.
/// 
/// A `Graphic<Texture>` contains a cached 
/// SDL texture and features additional methods for refreshing
/// the texture cache and drawing the texture to screen. Any time the 
/// tiles in the graphic are changed, a `dirty` flag for those tiles is set to true.
/// This can be set back to false again by calling `update_texture`,
/// which redraws all dirty tiles in the graphic to the texture. 
#[derive(Clone)]
pub struct Graphic<T> {
    width: u32, 
    height: u32,
    tiles: Vec<Tile>,
    texture: T,
    dirty: Vec<bool>,
}

impl <T> Index<(u32, u32)> for Graphic<T> {
    type Output = Tile;
    fn index(&self, index : (u32,u32)) -> &Tile {
        &self.tiles[(index.0 + index.1 * self.width) as usize]
    }
}
impl <T> IndexMut<(u32, u32)> for Graphic<T> {
    fn index_mut(&mut self, index : (u32,u32)) -> &mut Tile {
        &mut self.tiles[(index.0 + index.1 * self.width) as usize]
    }
}

impl Graphic<()> {
    /// Load a graphic from a file given its path. Sugar for using 
    /// `load_from` with a freshly-opened `File`: 
    /// ```
    /// Graphic::load_file(path)
    /// // same as
    /// Graphic::load_from(File::open(path)?) 
    /// ```
    pub fn load_file<P: AsRef<Path>>(path : P) -> io::Result<Graphic<()>> {
        Graphic::load_from(File::open(path)?)
    }
    /// Load a graphic from any instance of the `Read` trait.
    /// Commonly used for loading files statically bundled into the binary with
    /// `include_bytes!`.
    /// 
    /// ```
    /// let g = Graphic::load_from(Cursor::new(&include_bytes!("file")[..]))
    /// ```
    pub fn load_from<R: Read>(input : R) -> io::Result<Graphic<()>> {
        let mut f = input;
        let w = f.read_u32::<LittleEndian>()?;
        let h = f.read_u32::<LittleEndian>()?;
        let mut me = Graphic::blank(w,h);
        let mut cur = 0;
        loop {
            match f.read_u32::<LittleEndian>() {
                Ok(index) => {
                    let fg = Color::RGBA(f.read_u8()?, f.read_u8()?, f.read_u8()?, f.read_u8()?);
                    let bg = Color::RGBA(f.read_u8()?, f.read_u8()?, f.read_u8()?, f.read_u8()?);
                    me.tiles[cur] = Tile{index:index as usize,fg:fg,bg:bg};
                    cur += 1;
                },
                _ => break
            }
        }
        Ok(me)
    }
    /// Create a graphic comprised of the given tile repeated `width` times `height` times.
    pub fn solid(width: u32, height: u32, tile: Tile) -> Graphic<()>{
        let mut tiles = Vec::new();
        let mut dirty = Vec::new();
        for _ in 0..width {
            for _ in 0..height {
                tiles.push(tile);
                dirty.push(true)
            }
        }
        Graphic {
            width: width,
            height: height,
            tiles: tiles,
            texture: (),
            dirty: dirty
        }
    }
    /// Create a blank graphic of the given size, with the tile index 0, fully transparent colors.
    /// ```
    /// Graphic::blank(8,12)
    /// // same as
    /// Graphic::solid(8,12,Default::default())
    /// ```
    pub fn blank(width: u32, height: u32) -> Graphic<()>{
        Graphic::solid(width,height,Default::default())
    }

    /// A method to attach an SDL texture, converting the graphic from an unrenderable one to a renderable one.
    /// Commonly used straight after creation, like so:
    /// ```
    /// let g = Graphics::load_file("path")?.textured(texture_creator);
    /// ```
    /// Note that the texture has not rendered yet (and is marked as dirty), so typically you would want to call `update_texture` 
    /// and provide a tileset before drawing to screen.
    pub fn textured<'r, T>(&self,texture_creator: &'r TextureCreator<T>) -> Graphic<Texture<'r>> {
        let mut tex = texture_creator.create_texture_streaming(PixelFormatEnum::ARGB8888, 8 * self.width, 8 * self.height).unwrap();
        tex.set_blend_mode(BlendMode::Blend);
        let g = Graphic {
            width: self.width,
            height: self.height,
            tiles: self.tiles.clone(),
            texture: tex,
            dirty: self.dirty.clone()
        };
        g
    }
}
impl <T>Graphic<T> {

    /// Save a graphic to some instance of `Write` (such as a file), using the same file format used in `load_from`.
    pub fn save<W:Write>(&self,file:&mut W) -> io::Result<()> {
        file.write_u32::<LittleEndian>(self.width)?;
        file.write_u32::<LittleEndian>(self.height)?;
        for t in &self.tiles {
            file.write_u32::<LittleEndian>(t.index as u32)?;
            file.write_u8(t.fg.r)?;
            file.write_u8(t.fg.g)?;
            file.write_u8(t.fg.b)?;
            file.write_u8(t.fg.a)?;
            file.write_u8(t.bg.r)?;
            file.write_u8(t.bg.g)?;
            file.write_u8(t.bg.b)?;
            file.write_u8(t.bg.a)?;
        }
        Ok(())
    }
    /// The width of the graphic in tiles.    
    pub fn width(&self) -> u32 {
        self.width
    }
    /// The height of the graphic in tiles.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Return the tile at the given location in the graphic.
    /// Equivalent to indexing, but returns a blank tile if out of bounds
    /// instead of a panic.
    pub fn get_tile(&self, x:u32, y:u32) -> Tile  {
        if x < self.width && y < self.height {
            self.tiles[(x + y * self.width) as usize]
        } else {
            Default::default()
        }
    }
    /// Change the tile at position `(x,y)` in the graphic.
    /// Equivalent to assigning to an index, but does nothing if the tile 
    /// is out of bounds rather than panic.
    pub fn set_tile(&mut self, x: u32, y: u32, tile : Tile) {        
        if x < self.width && y < self.height {
            let i = (x + y * self.width) as usize;
            if let Some(mut t) = self.tiles.get_mut(i) {
                if *t != tile {
                    t.index = tile.index;
                    t.fg = tile.fg;
                    t.bg = tile.bg;
                    self.dirty[i] = true;
                }
            }
        }
    }
    /// Set the colors of the given tile in the graphic but leave the tile index unchanged.
    pub fn color_tile(&mut self, x: u32, y:u32, fg: Color, bg: Color) {
        if x < self.width && y < self.height {
            let i = (x + y * self.width) as usize;
            if let Some(mut t) = self.tiles.get_mut(i) {
                if t.fg != fg || t.bg != bg { 
                    t.fg = fg;
                    t.bg = bg;
                    self.dirty[i] = true;
                }
            }
        }

    }

    /// Draw a filled rectangle starting at `(x,y)` in the top-left of dimensions 
    /// `width` times `height`, consisting of the tile `tile`.
    pub fn draw_rect(&mut self, x: u32, y:u32, width:u32, height:u32, tile: Tile) {
        for xi in 0..width {
            for yi in 0..height {
                self.set_tile(xi+x, yi+y, tile)
            }
        }
    }

    /// Change the foreground and background colors of all tiles  in the 
    /// rectangle starting at `(x,y)` in the top-left of dimensions 
    /// `width` times `height`, but leaving the tile index unchanged.
    pub fn color_rect(&mut self, x: u32, y:u32, width:u32, height:u32, fg: Color, bg: Color) {
        for xi in 0..width {
            for yi in 0..height {
                self.color_tile(xi+x, yi+y, fg,bg)
            }
        }
    }

    /// Draw text using one tile per character moving leftward, starting at `(x,y)`, according to the character map built-in to the 
    /// tile set. If it overflows the end of the graphic, the text is truncated.
    pub fn draw_text(&mut self, string: &str, tile_set : &TileSet, x : u32, y : u32, fg : Color, bg : Color) {
        let bytes = string.as_bytes();        
        let mut i = 0;
        for b in bytes {
            self.set_tile(x + i, y, Tile{index: tile_set[*b as char], fg: fg, bg:bg});
            i += 1
        }        
    }



    /// Copy tiles from another `Graphic` to this one. Every tile in the `other` graphic in the rectangle of size `src_w` times `src_height` 
    /// starting at `(src_x,src_y)` in the top-left is copied to this graphic starting at `(dest_x, dest_y)`.
    pub fn copy_tiles_from<U>(&mut self, other: &Graphic<U>, src_x: u32, src_y: u32, src_w: u32, src_h: u32, dest_x: u32, dest_y: u32) {
        for j in 0..src_h {
            if src_y + j < other.height {
                for i in 0..src_w {
                    if src_x + i < other.width {
                        self.set_tile(dest_x + i, dest_y + j, other.get_tile(src_x + i, src_y + j))
                    }
                }
            }
        }

    }
    
    /// Copy all tiles from another `Graphic` to this one. Every tile in the `other` graphic is copied to this graphic starting at `(dest_x, dest_y)`.
    pub fn copy_all_tiles_from<U>(&mut self, other: &Graphic<U>, dest_x:u32, dest_y: u32) {
        self.copy_tiles_from(other, 0, 0, other.width, other.height, dest_x, dest_y)
    }

}
impl <'r>Graphic<Texture<'r>> {

    /// A shortcut to load a file and associate a texture in one step. Equivalent to using `load_file` and then `textured`.
    pub fn load_file_textured<P: AsRef<Path>,T>(path : P,texture_creator: &'r TextureCreator<T>) -> io::Result<Graphic<Texture<'r>>> {
        let g = Graphic::load_from(File::open(path)?)?;
        Ok(g.textured(texture_creator))
    }
    /// Mark all tiles in the graphic as needing redrawing (via `update_texture`). This is useful if you need to change tile sets.
    pub fn mark_dirty(&mut self) {
        for i in 0..self.dirty.len() {
            self.dirty[i] = true
        }
    }
    /// Draw each tile marked dirty in the graphic to the internal texture using the provided tile set.
    pub fn update_texture(&mut self, tile_set : &TileSet) {
        let mut i = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.dirty[i] {
                    let t = self.tiles[i];
                    tile_set.draw_tile_to(t.index,&mut self.texture,Point::new((x * 8) as i32, (y * 8) as i32), t.fg,t.bg);
                    self.dirty[i] = false;
                }
                i += 1
            }
        }
    }
    /// Draw the graphic to the screen at the provided position.
    /// Note that you may wish to call `update_texture` and provide a tile set first, as this simply draws the 
    /// cached texture.
    pub fn draw<P : Into<Point>,T:RenderTarget>(&self, canvas: &mut Canvas<T>, position : P) {   
        let position = position.into();
        canvas.copy(&self.texture, None, Rect::new(position.x,position.y,self.width * 8, self.height * 8)).unwrap();
    }
    /// Get the SDL texture associated with this graphic.
    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}

/// Mostly used internally. 
/// Given a 64 bit integer, interprets it as an 8x8 tile and draws it to the 
/// given texture at the given point with the given foreground and background colors.
pub fn draw_tile_data<P : Into<Point>>(data: u64 , tex: &mut Texture, point: P, fg: Color, bg: Color) {
        let point = point.into();
        let mut m = data;
        let mut pixel_data = [0 as u8; (64 * 4)];        
        let mut curr = 0;
        for _ in 0..64 {
            let (r,g,b,a) = if (m & 0x01) != 0 { fg.rgba() } else { bg.rgba() };
            m >>= 1;
            pixel_data[curr] = b; curr += 1;
            pixel_data[curr] = g; curr += 1;
            pixel_data[curr] = r; curr += 1;
            pixel_data[curr] = a; curr += 1;
        }
        tex.update(Rect::new(point.x,point.y,8,8),&pixel_data, 8 * 4).unwrap();
}