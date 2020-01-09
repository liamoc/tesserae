use std::fs::File;
use std::path::Path;
use std::ops::{Index, IndexMut};
use std::io;
use std::io::{Read,Cursor};
use sdl2::pixels::Color;
use byteorder::{ReadBytesExt,WriteBytesExt};

const SWATCH_SIZE: usize = 256;
pub struct Swatch {
    data: Vec<Color>,
}
impl Index<usize> for Swatch {
    type Output = Color;
    fn index(&self, index: usize) -> &Color {
        &self.data[index]
    }
}
impl IndexMut<usize> for Swatch {
    fn index_mut(&mut self, index: usize) -> &mut Color {
        &mut self.data[index]
    }
}
impl Swatch {
    fn new() -> Swatch {
        Swatch {
            data: Vec::new(),
        }
    }
    pub fn default() -> Swatch {
        let f = include_bytes!("../../../swatch");
        Swatch::load_from(Cursor::new(&f[..]))
    }
    fn read_color<R:Read>(mut f : R) -> io::Result<Color> {
        let r = f.read_u8()?;
        let g = f.read_u8()?;
        let b = f.read_u8()?; 
        let a = f.read_u8()?;        
        Ok(Color::RGBA(r,g,b,a))
    }
    
    pub fn load_from<R: Read>(mut input : R) -> Swatch {
        let mut ts = Swatch::new();
        let mut c = 0;
        loop {
            match Swatch::read_color(&mut input) {
                Ok(u) => { if c >= SWATCH_SIZE { break } else { ts.data.push(u); c += 1} },
                _ => break
            }           
        }
        while c < SWATCH_SIZE {
            c += 1;
            ts.data.push(Color::RGBA(0,255,0,0));
        }
        ts
    }
    pub fn load_file<P: AsRef<Path>>(path : P) -> io::Result<Swatch> {
        let f = File::open(path)?;
        Ok(Swatch::load_from(f))
    }
    pub fn store<P: AsRef<Path>>(&self, path : P) -> io::Result<()> {
        let mut f = File::create(path)?;
        for i in &self.data {
            f.write_u8(i.r)?;
            f.write_u8(i.g)?;
            f.write_u8(i.b)?;
            f.write_u8(i.a)?;
        }
        Ok(())
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }

}