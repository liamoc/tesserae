use tesserae::{Graphic,TileSet,Tile};
use super::widgets::TileSetChooser;
use super::locations;
use super::colors;

use std::io;
use std::io::Cursor;
use bit_reverse::ParallelReverse;
use sdl2::keyboard::Keycode;
use sdl2::render::{TextureCreator,Texture,Canvas};
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::video::{Window,WindowContext};

pub struct TileEditor<'r> {
    cursor_x: u32,
    cursor_y: u32,
    editor_view: Graphic<Texture<'r>>,
    tile_set_chooser: TileSetChooser<'r>,
    default_tiles: TileSet,
    preview: Graphic<Texture<'r>>,
    chrome_view: Graphic<Texture<'r>>,
    stats_view: Graphic<Texture<'r>>,
    clipboard: u64,
    help_graphic: Graphic<Texture<'r>>,
    prompt_graphic : Graphic<Texture<'r>>,
    file_name: &'r str,
}

impl <'r> TileEditor<'r> {
    pub fn create(file_name : &'r str, texture_creator: &'r TextureCreator<WindowContext>) -> io::Result<TileEditor<'r>> {
        let tile_set = TileSet::load_file(file_name)?;
        let default_tiles = TileSet::default();
        let editor_graphic = Graphic::blank(8, 8).textured(texture_creator);
        let stats_graphic = Graphic::blank(24,8).textured(texture_creator);
        let preview_graphic = Graphic::blank(6,2).textured(texture_creator);
        let mut chrome_graphic = Graphic::blank(160,100).textured(texture_creator);
        chrome_graphic.draw_rect(0,0,160,100, Tile{index:0, fg:colors::BLACK, bg:colors::BLACK});
        chrome_graphic.draw_rect(0,0,160,1, Tile{index:0, fg:colors::BLACK, bg:colors::GRAY});
        chrome_graphic.draw_text("tile editor :)", &default_tiles, 0, 0, colors::WHITE, colors::GRAY);
        let g = Graphic::load_from(Cursor::new(&include_bytes!("../../../tile_chrome")[..])).unwrap();
        let g2 = Graphic::load_from(Cursor::new(&include_bytes!("../../../tile_chrome_2")[..])).unwrap();
        chrome_graphic.copy_all_tiles_from(&g,0, 1);
        chrome_graphic.copy_all_tiles_from(&g2,19, 1);
        chrome_graphic.update_texture(&default_tiles);
        let mut help_graphic = Graphic::load_from(Cursor::new(&include_bytes!("../../../tile_help")[..])).unwrap().textured(texture_creator);
        help_graphic.update_texture(&default_tiles);
        let mut prompt_graphic= Graphic::load_from(Cursor::new(&include_bytes!("../../../tile_char_prompt")[..])).unwrap().textured(texture_creator);
        prompt_graphic.update_texture(&default_tiles);
        let tile_set_chooser = TileSetChooser::create(tile_set, texture_creator);
        Ok(TileEditor {
            cursor_x: 0,
            cursor_y: 0, 
            tile_set_chooser: tile_set_chooser,
            editor_view: editor_graphic,
            chrome_view: chrome_graphic,
            stats_view: stats_graphic,
            preview: preview_graphic,
            clipboard: 0,       
            help_graphic: help_graphic, 
            prompt_graphic: prompt_graphic,
            file_name: file_name,
            default_tiles,
        })
    }
    fn refresh_editor_view(&mut self) {
        let mut m = self.tile_set_chooser.tile_set()[self.tile_set_chooser.selected()];
        for y in 0..8 {
            for x in 0..8 {
                let fg = if x == self.cursor_x && y == self.cursor_y { colors::CYAN } else { colors::WHITE };
                let bg = if x == self.cursor_x && y == self.cursor_y { colors::TEAL } else { colors::BLACK };
                let t = Tile { index: (m & 1) as usize, fg: fg, bg: bg};
                self.editor_view.set_tile(x, y, t);
                m >>= 1                
            }
        }
        self.editor_view.update_texture(&self.default_tiles);
    }
    fn refresh_stats_view(&mut self) {
        let g = &mut self.stats_view;
        g.draw_rect(5,0,3,3,Tile{index:0, bg: colors::TRANSPARENT, fg: colors::TRANSPARENT});
        g.draw_text(&self.tile_set_chooser.selected().to_string(), &self.default_tiles, 5,0,colors::PALE_YELLOW, colors::TRANSPARENT);
        g.draw_text(&self.cursor_x.to_string(), &self.default_tiles, 5,1,colors::PALE_YELLOW, colors::TRANSPARENT);
        g.draw_text(&self.cursor_y.to_string(), &self.default_tiles, 5,2,colors::PALE_YELLOW, colors::TRANSPARENT);
        self.preview.draw_rect(0,0,3,2,Tile{index:self.tile_set_chooser.selected(), fg:colors::WHITE,bg:colors::BLACK});
        self.preview.draw_rect(3,0,3,2,Tile{index:self.tile_set_chooser.selected(), bg:colors::WHITE,fg:colors::BLACK});
        self.preview.mark_dirty();
        self.preview.update_texture(self.tile_set_chooser.tile_set());
        g.update_texture(&self.default_tiles);
    }
    fn refresh_views(&mut self) {
        self.refresh_editor_view();
        self.refresh_stats_view();
        self.tile_set_chooser.refresh(true);
    }
    fn change_tile(&mut self, delta: i32) {
        self.tile_set_chooser.move_selected(delta);
        self.refresh_views();
    }
    fn copy_clipboard(&mut self) {
        self.clipboard = self.tile_set_chooser.selected_data();
    }
    fn paste_clipboard(&mut self) {
        *self.tile_set_chooser.selected_data_mut() = self.clipboard;
        self.refresh_views();
    }
    fn move_cursor(&mut self, delta_x: i32, delta_y: i32) {
        let new_x = ((self.cursor_x as i32 + delta_x).max(0).min(7)) as u32;
        let new_y = ((self.cursor_y as i32 + delta_y).max(0).min(7)) as u32;
        self.cursor_x = new_x;
        self.cursor_y = new_y;
        self.refresh_views();
    }
    fn flip_pixel(&mut self) {
        let m : &mut u64 = self.tile_set_chooser.selected_data_mut();
        let bit = 0x1 << (self.cursor_x + (self.cursor_y *8));
        *m = *m ^ bit;
        self.refresh_views();
    }
    fn flip_horizontal(&mut self) {
        let x : &mut u64 = self.tile_set_chooser.selected_data_mut();
        let m : [u8;8] = x.to_le_bytes();
        let mut n = [0 as u8; 8];
        for i in 0..8 {
            n[i] = m[i].swap_bits();       
        }
        *x = u64::from_le_bytes(n);
        self.refresh_views();
    }
    fn flip_vertical(&mut self) {
        let x : &mut u64 = self.tile_set_chooser.selected_data_mut();
        let m : [u8;8] = x.to_le_bytes();        
        *x = u64::from_be_bytes(m);
        self.refresh_views();
    }

    fn rotate(&mut self) {
        let x : &mut u64 = self.tile_set_chooser.selected_data_mut();
        let mut m : [u8;8] = x.to_le_bytes();        
        let mut n = [0 as u8; 8];
        for i in 0..8 {            
            n[i] = (m[0] & 1) << 0
                 | (m[1] & 1) << 1 
                 | (m[2] & 1) << 2
                 | (m[3] & 1) << 3
                 | (m[4] & 1) << 4
                 | (m[5] & 1) << 5
                 | (m[6] & 1) << 6
                 | (m[7] & 1) << 7;
            for i in 0..8 {
                m[i] >>= 1;
            }
        }
        *x = u64::from_be_bytes(n);
        self.refresh_views();
    }
    pub fn set_selected(&mut self, index: usize) {
        self.tile_set_chooser.set_selected(index);
        self.refresh_views();
    }
    pub fn selected(&self) -> usize {
        self.tile_set_chooser.selected()
    }
    pub fn main_loop(&mut self, canvas : &mut Canvas<Window>, event_pump : &mut EventPump) {
        let mut showing_help = false;
        let mut typing_mode = false;
        canvas.set_draw_color(colors::BLACK);
        'running: loop {
            canvas.clear();
            self.chrome_view.draw(canvas,locations::CHROME);
            if showing_help || typing_mode { 
                if showing_help {
                    self.help_graphic.draw(canvas,(locations::CHROME.0,locations::CHROME.1+8)); 
                } else if typing_mode {
                    self.prompt_graphic.draw(canvas,(locations::CHROME.0,locations::CHROME.1+8)); 
                }
            } else { 
                self.tile_set_chooser.graphic().draw(canvas, locations::TILE_CHOOSER);
                self.stats_view.draw(canvas, locations::STATS);
                self.preview.draw(canvas, locations::PREVIEW);
            }
            self.editor_view.draw(canvas, locations::EDITOR);
            canvas.present();
            let event = event_pump.wait_event();
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },                
                Event::TextInput { text: ref st,..} if typing_mode => {
                    if let Some(c) = st.chars().next() {
                        self.tile_set_chooser.tile_set_mut()[c] = self.tile_set_chooser.selected();                    
                    }
                    typing_mode = false;
                },
                Event::KeyDown { keycode: Some(code),.. } if typing_mode => 
                    if code == Keycode::Tab { typing_mode = !typing_mode },
                Event::KeyDown { keycode: Some(Keycode::Tab), ..} => typing_mode = !typing_mode,
                Event::KeyDown { keycode: Some(Keycode::Backquote), ..} => showing_help = true,
                Event::KeyUp { keycode: Some(Keycode::Backquote), ..} => showing_help = false,
                Event::KeyDown { keycode: Some(Keycode::D ), ..} => self.change_tile(1),
                Event::KeyDown { keycode: Some(Keycode::A ), ..} => self.change_tile(-1),
                Event::KeyDown { keycode: Some(Keycode::S ), ..} => self.change_tile(16),
                Event::KeyDown { keycode: Some(Keycode::W ), ..} => self.change_tile(-16),
                Event::KeyDown { keycode: Some(Keycode::Up ), ..} => self.move_cursor(0, -1),
                Event::KeyDown { keycode: Some(Keycode::Down ), ..} => self.move_cursor(0, 1),
                Event::KeyDown { keycode: Some(Keycode::Left ), ..} => self.move_cursor(-1, 0),
                Event::KeyDown { keycode: Some(Keycode::Right ), ..} =>  self.move_cursor(1, 0),
                Event::KeyDown { keycode: Some(Keycode::Space ), ..} =>  self.flip_pixel(),
                Event::KeyDown { keycode: Some(Keycode::R), ..} =>  self.rotate(),
                Event::KeyDown { keycode: Some(Keycode::H), ..} =>  self.flip_horizontal(),
                Event::KeyDown { keycode: Some(Keycode::V), ..} =>  self.flip_vertical(),
                Event::KeyDown { keycode: Some(Keycode::Y ), ..} => self.copy_clipboard(),
                Event::KeyDown { keycode: Some(Keycode::P ), ..} => self.paste_clipboard(),
                _ => {}
            }
        }
        self.tile_set_chooser.tile_set().store(self.file_name).unwrap();

    }
}
