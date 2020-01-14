mod swatch;
mod widgets;
mod tile_editor;
mod locations;
mod colors;

use tesserae::{Graphic, TileSet,Tile};
use tile_editor::TileEditor;
use widgets::{TileSetChooser,ColorChooser};
use swatch::Swatch;
use std::io::Cursor;
use std::fs::File;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture,TextureCreator};
use sdl2::render::Canvas;
use sdl2::rect::Point;
use sdl2::video::WindowContext;
use sdl2::video::Window;
use sdl2::keyboard::Mod;
use sdl2::EventPump;

struct GraphicsEditor<'r> {
    cursor_x: u32,
    cursor_y: u32,
    fg: Color,
    bg: Color, 
    editor_view: Graphic<Texture<'r>>,
    cursor_sprite: Graphic<Texture<'r>>,
    tile_set_chooser: TileSetChooser<'r>,
    color_chooser: ColorChooser<'r>,
    chrome_view: Graphic<Texture<'r>>,
    stats_view: Graphic<Texture<'r>>,
    preview : Graphic<Texture<'r>>,
    rect_mark: Option<(u32,u32)>,
    clipboard: Option<Graphic<()>>,
    typing_mode: bool,
    help_graphic: Graphic<Texture<'r>>,
    config : &'r Config<'r>,
    default_tiles: TileSet,
}

impl <'r> GraphicsEditor<'r> {
    pub fn create(editor_graphic: Graphic<Texture<'r>>, config : &'r Config<'r>, texture_creator: &'r TextureCreator<WindowContext>) -> GraphicsEditor<'r> {
        let default_tiles = TileSet::default();
        let tile_set = TileSet::load_file(config.tile_set_file_name).unwrap();
        let swatch = Swatch::load_file(config.swatch_file_name).unwrap_or_else(|_| Swatch::default());
        let stats_graphic = Graphic::blank(16,16).textured(texture_creator);
        let cursor_graphic = Graphic::blank(1,1).textured(texture_creator); 
        let preview = Graphic::blank(9,9).textured(texture_creator);
        let mut help_graphic = Graphic::load_from(Cursor::new(&include_bytes!("../../../graphic_help")[..])).unwrap().textured(texture_creator); 
        help_graphic.update_texture(&default_tiles);
        let mut chrome_graphic = Graphic::blank(160,100).textured(texture_creator);
        chrome_graphic.draw_rect(0,0,160,100, Tile{index:0, fg:colors::BLACK, bg:colors::BLACK});
        chrome_graphic.draw_rect(0,0,160,1, Tile{index:0, fg:colors::BLACK, bg:colors::GRAY});
        chrome_graphic.draw_text("graphics editor :)",&default_tiles, 0, 0, colors::WHITE, colors::GRAY);
        let g = Graphic::load_from(Cursor::new(&include_bytes!("../../../graphic_chrome")[..])).unwrap();
        chrome_graphic.copy_all_tiles_from(&g,0, 1);
        chrome_graphic.draw_rect(20,3,editor_graphic.width(), editor_graphic.height(),Tile{index:255, fg: colors::WHITE,bg:colors::PALE_GRAY});
        chrome_graphic.update_texture(&default_tiles);
        let tile_set_chooser = TileSetChooser::create(tile_set, texture_creator);
        let color_chooser = ColorChooser::create(swatch, texture_creator);
        GraphicsEditor {
            cursor_x: 0,
            cursor_y: 0, 
            fg: colors::WHITE,
            bg: colors::BLACK,
            tile_set_chooser: tile_set_chooser,
            editor_view: editor_graphic,
            chrome_view: chrome_graphic,
            stats_view: stats_graphic,
            cursor_sprite: cursor_graphic,  
            preview: preview,
            color_chooser: color_chooser,
            rect_mark:None,
            clipboard:None,
            typing_mode:false,
            help_graphic: help_graphic,
            config: config,
            default_tiles: default_tiles
        }
    }
    fn refresh_views(&mut self) {
        self.refresh_editor_view();
        self.refresh_stats_view();
        self.refresh_cursor();
        self.tile_set_chooser.refresh(false);
        self.color_chooser.refresh(&self.default_tiles);
    }
    fn refresh_cursor(&mut self) {
        let t = self.editor_view.get_tile(self.cursor_x,self.cursor_y);
        let fg = colors::inverse(t.fg,128);
        let bg = colors::inverse(t.bg,128);
        self.cursor_sprite.set_tile(0,0,Tile{index:254,fg:fg, bg:bg});
        self.cursor_sprite.update_texture(&self.default_tiles);
    }
    fn change_tile(&mut self, delta: i32) {
        self.tile_set_chooser.move_selected(delta);
        self.refresh_views();
    }
    fn change_color(&mut self, delta: i32) {
        self.color_chooser.move_selected(delta);
        self.refresh_views();
    }
    fn flip_colors(&mut self) {
        let temp = self.fg;
        self.fg = self.bg;
        self.bg = temp;
        self.refresh_stats_view();
    }
    fn load_from_swatch(&mut self) {
        self.fg = self.color_chooser.swatch()[self.color_chooser.selected()];
        self.refresh_views()
    }
    fn store_into_swatch(&mut self) {
        let x = self.color_chooser.selected();
        self.color_chooser.swatch_mut()[x] = self.fg;
        self.refresh_views()
    }
    fn adjust_foreground_color(&mut self, dr : i16, dg:i16, db:i16,da:i16) {
        let new_r = (self.fg.r as i16 + dr).max(0).min(255) as u8;
        let new_g = (self.fg.g as i16 + dg).max(0).min(255) as u8;
        let new_b = (self.fg.b as i16 + db).max(0).min(255) as u8;
        let new_a = (self.fg.a as i16 + da).max(0).min(255) as u8;
        self.fg = Color::RGBA(new_r,new_g,new_b,new_a);
        self.refresh_stats_view();
    }
    fn dropper(&mut self) {
        {
            let t = self.editor_view.get_tile(self.cursor_x,self.cursor_y);
            self.tile_set_chooser.set_selected(t.index);
            self.fg = t.fg;
            self.bg = t.bg;
        }
        self.refresh_views();
    }
    fn mark_rectangle(&mut self) {
        if self.rect_mark.is_some() {
            self.rect_mark = None;
        } else { 
            self.rect_mark = Some((self.cursor_x,self.cursor_y));
        }
        self.refresh_views()
    }
    fn copy_clipboard(&mut self) {
        match self.rect_mark {
            Some((mx,my)) => { 
                let x = mx.min(self.cursor_x);
                let y = my.min(self.cursor_y);
                let w = mx.max(self.cursor_x) - x + 1;
                let h = my.max(self.cursor_y) - y + 1;
                let mut clip = Graphic::blank(w,h);
                clip.copy_tiles_from(&self.editor_view, x, y, w, h, 0, 0);
                self.clipboard=Some(clip);
            },
            _ => {
                let mut clip = Graphic::blank(1,1);
                clip.copy_tiles_from(&self.editor_view, self.cursor_x, self.cursor_y, 1, 1, 0, 0);
                self.clipboard=Some(clip);
            }
        }
    }
    fn paste_clipboard(&mut self) {
        match &self.clipboard {
            Some(g) => {
                self.editor_view.copy_all_tiles_from(g, self.cursor_x,  self.cursor_y)   
            }, 
            None => {}
        }
    }
    fn place_style(&mut self) {
        match self.rect_mark {
            Some((mx,my)) => { 
                let x = mx.min(self.cursor_x);
                let y = my.min(self.cursor_y);
                let w = mx.max(self.cursor_x) - x + 1;
                let h = my.max(self.cursor_y) - y + 1;
                self.editor_view.color_rect(x,y,w,h,self.fg,self.bg);
            },
            None => { 
                self.editor_view.color_tile(self.cursor_x,self.cursor_y,self.fg,self.bg);
            }
        }    
        self.rect_mark = None;    
        self.refresh_views()
    }
    fn move_cursor(&mut self, delta_x: i32, delta_y: i32) {
        let new_x = ((self.cursor_x as i32 + delta_x).max(0).min(self.editor_view.width()  as i32 - 1)) as u32;
        let new_y = ((self.cursor_y as i32 + delta_y).max(0).min(self.editor_view.height() as i32 - 1)) as u32;
        self.cursor_x = new_x;
        self.cursor_y = new_y;
        self.refresh_views();
    }
    fn place_tile(&mut self) {
        let t = Tile{index: self.tile_set_chooser.selected(), fg: self.fg, bg: self.bg };
        match self.rect_mark {
            Some((mx,my)) => { 
                let x = mx.min(self.cursor_x);
                let y = my.min(self.cursor_y);
                let w = mx.max(self.cursor_x) - x + 1;
                let h = my.max(self.cursor_y) - y + 1;
                self.editor_view.draw_rect(x,y,w,h,t);
            },
            None => { 
                self.editor_view.set_tile(self.cursor_x,self.cursor_y,t);
            }
        }    
        self.rect_mark = None;    
        self.refresh_views();
    }
    fn place_letter(&mut self, c : char) {
        self.tile_set_chooser.set_selected(self.tile_set_chooser.tile_set()[c]);
        self.place_tile();
        self.move_cursor(1,0);
        self.refresh_views()
    }
    fn refresh_stats_view(&mut self) {
        let g = &mut self.stats_view;
        g.draw_rect(5,0,3,7,Tile{index:0, bg: colors::TRANSPARENT, fg: colors::TRANSPARENT});
        g.draw_text(&self.tile_set_chooser.selected().to_string(), &self.default_tiles,5,0,colors::PALE_YELLOW, colors::TRANSPARENT);
        g.draw_text(&self.cursor_x.to_string(), &self.default_tiles, 5,1,colors::PALE_YELLOW, colors::TRANSPARENT);
        g.draw_text(&self.cursor_y.to_string(), &self.default_tiles, 5,2,colors::PALE_YELLOW, colors::TRANSPARENT);
        g.draw_text(&self.fg.r.to_string(),  &self.default_tiles, 5,3,colors::PALE_YELLOW, colors::TRANSPARENT);
        g.draw_text(&self.fg.g.to_string(),  &self.default_tiles, 5,4,colors::PALE_YELLOW, colors::TRANSPARENT);
        g.draw_text(&self.fg.b.to_string(),  &self.default_tiles, 5,5,colors::PALE_YELLOW, colors::TRANSPARENT);
        g.draw_text(&self.fg.a.to_string(),  &self.default_tiles, 5,6,colors::PALE_YELLOW, colors::TRANSPARENT);
        self.preview.draw_rect(3,0,3,3,Tile{index:self.tile_set_chooser.selected(), fg:self.fg, bg:self.bg});
        self.preview.draw_rect(0,0,3,3,Tile{index:0, fg:self.fg, bg:self.fg});
        self.preview.draw_rect(3,3,3,3,Tile{index:0, fg:self.bg, bg:self.bg});
        self.preview.draw_rect(0,3,3,3,self.editor_view.get_tile(self.cursor_x,self.cursor_y));
        g.set_tile(1,2,Tile{index: if self.rect_mark.is_none() { 0 } else { 19 }, fg:colors::YELLOW, bg:colors::BLACK});
        g.set_tile(1,3,Tile{index: if self.clipboard.is_none() { 0 } else { 4 }, fg:colors::PALE_RED, bg:colors::BLACK});
        g.set_tile(1,4,Tile{index: if self.typing_mode { 21 } else { 0 }, fg:colors::PALE_BLUE, bg:colors::BLACK});
        g.update_texture(&self.default_tiles);
        self.preview.update_texture(self.tile_set_chooser.tile_set());
    }
    fn refresh_editor_view(&mut self) {
        self.editor_view.update_texture(self.tile_set_chooser.tile_set());
    }
    fn main_loop(&mut self, canvas :&mut Canvas<Window>, event_pump : &mut EventPump) {
        let mut showing_help = false;
        'running: loop {
            
            canvas.set_draw_color(colors::BLACK);
            canvas.clear();
            self.chrome_view.draw(canvas,locations::CHROME);
            if showing_help { 
                self.help_graphic.draw(canvas,(locations::CHROME.0,locations::CHROME.1 + 8) ); 
            } else { 
                self.tile_set_chooser.graphic().draw(canvas, locations::TILE_CHOOSER);
                self.color_chooser.graphic().draw(canvas, locations::COLOR_CHOOSER);
                self.stats_view.draw(canvas, locations::STATS);
                self.preview.draw(canvas, locations::PREVIEW);
            }
            self.editor_view.draw(canvas, locations::EDITOR);
            match self.rect_mark {
                Some((mx,my)) => {
                    let x = mx.min(self.cursor_x);
                    let y = my.min(self.cursor_y);
                    let w = mx.max(self.cursor_x) - x + 1;
                    let h = my.max(self.cursor_y) - y + 1;
                    for j in 0..h {
                        for i in 0..w {
                            self.cursor_sprite.draw(canvas, Point::new((x + i) as i32 * 8 + locations::EDITOR.0, (y + j) as i32 * 8 + locations::EDITOR.1));
                        } 
                    }
                },
                _ => 
                    self.cursor_sprite.draw(canvas, Point::new(self.cursor_x as i32 * 8 + locations::EDITOR.0, self.cursor_y as i32 * 8 + locations::EDITOR.1)),
            }

            canvas.present();
            let event = event_pump.wait_event();
            match event {
                Event::KeyDown { keycode: Some(Keycode::Up ), ..} => self.move_cursor(0, -1),
                Event::KeyDown { keycode: Some(Keycode::Down ), ..} => self.move_cursor(0, 1),
                Event::KeyDown { keycode: Some(Keycode::Left ), ..} => self.move_cursor(-1, 0),
                Event::KeyDown { keycode: Some(Keycode::Right ), ..} =>  self.move_cursor(1, 0),
                Event::KeyDown { keycode: Some(Keycode::Return ), ..} => self.dropper(),
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(code),.. } if self.typing_mode => 
                    if code == Keycode::Tab { self.typing_mode = !self.typing_mode; self.refresh_views() },
                Event::TextInput { text: ref st,..} if self.typing_mode => {
                    self.place_letter(st.chars().next().unwrap_or_default() );                    
                },
                Event::KeyDown { keycode: Some(Keycode::Tab),.. } => { 
                    self.typing_mode = !self.typing_mode;
                    self.refresh_views() 
                },
                Event::KeyDown { keycode: Some(Keycode::Backquote), ..} => showing_help = true,
                Event::KeyUp { keycode: Some(Keycode::Backquote), ..} => showing_help = false,
                Event::KeyDown { keycode: Some(Keycode::D ), keymod:m,.. } =>
                    if m.intersects(Mod::LSHIFTMOD) || m.intersects(Mod::RSHIFTMOD) {
                        self.change_color(1) 
                    } else { 
                        self.change_tile(1) 
                    },
                Event::KeyDown { keycode: Some(Keycode::A ), keymod:m,.. } =>
                    if m.intersects(Mod::LSHIFTMOD) || m.intersects(Mod::RSHIFTMOD) { 
                        self.change_color(-1) 
                    } else { 
                        self.change_tile(-1) 
                    },
                Event::KeyDown { keycode: Some(Keycode::S ), keymod:m,.. } =>
                    if m.intersects(Mod::LSHIFTMOD) || m.intersects(Mod::RSHIFTMOD) {
                        self.change_color(16) 
                    } else { 
                        self.change_tile(16) 
                    },
                Event::KeyDown { keycode: Some(Keycode::W ), keymod:m,.. } =>
                    if m.intersects(Mod::LSHIFTMOD) || m.intersects(Mod::RSHIFTMOD) { 
                        self.change_color(-16) 
                    } else { 
                        self.change_tile(-16) 
                    },
                Event::KeyDown { keycode: Some(Keycode::Y ), keymod:m,.. } => {
                    self.copy_clipboard();
                    if !(m.intersects(Mod::LSHIFTMOD) || m.intersects(Mod::RSHIFTMOD)) { 
                        self.rect_mark = None;
                    }
                    self.refresh_views()
                },
                Event::KeyDown { keycode: Some(Keycode::P ), keymod:m,.. } => {
                    self.paste_clipboard();
                    if m.intersects(Mod::LSHIFTMOD) || m.intersects(Mod::RSHIFTMOD) { 
                        self.clipboard = None;
                    }
                    self.refresh_views()
                },
                Event::KeyDown { keycode: Some(Keycode::Space ), keymod:m,.. } =>
                    if m.intersects(Mod::LSHIFTMOD) || m.intersects(Mod::RSHIFTMOD) {
                        self.place_style();
                    } else {
                        self.place_tile();
                    } 
                Event::KeyDown { keycode: Some(Keycode::F ), ..} => self.flip_colors(),
                Event::KeyDown { keycode: Some(Keycode::Q ), keymod:m,.. } =>
                    if m.intersects(Mod::LSHIFTMOD) || m.intersects(Mod::RSHIFTMOD) { 
                        self.store_into_swatch()
                    } else { 
                        self.load_from_swatch()
                    },
                Event::KeyDown { keycode: Some(Keycode::R ), keymod:m,.. } =>
                    self.adjust_foreground_color(if m.intersects(Mod::LSHIFTMOD) || m.intersects(Mod::RSHIFTMOD) { -1 } else { 1 },0,0,0),
                Event::KeyDown { keycode: Some(Keycode::G ), keymod:m,.. } =>
                    self.adjust_foreground_color(0,if m.intersects(Mod::LSHIFTMOD) || m.intersects(Mod::RSHIFTMOD) { -1 } else { 1 },0,0),
                Event::KeyDown { keycode: Some(Keycode::B ), keymod:m,.. } =>
                    self.adjust_foreground_color(0,0,if m.intersects(Mod::LSHIFTMOD) || m.intersects(Mod::RSHIFTMOD) { -1 } else { 1 },0),
                Event::KeyDown { keycode: Some(Keycode::T ), keymod:m,.. } =>
                    self.adjust_foreground_color(0,0,0,if m.intersects(Mod::LSHIFTMOD) || m.intersects(Mod::RSHIFTMOD) { -1 } else { 1 }),
                Event::KeyDown { keycode: Some(Keycode::V), ..} => self.mark_rectangle(),
                Event::KeyDown { keycode: Some(Keycode::E),..} => {
                    let texture_creator = canvas.texture_creator();
                    let mut editor = TileEditor::create(self.config.tile_set_file_name, &texture_creator).unwrap();
                    editor.set_selected(self.tile_set_chooser.selected());
                    editor.main_loop(canvas, event_pump);
                    let tile_set = TileSet::load_file(self.config.tile_set_file_name).unwrap();
                    self.editor_view.mark_dirty();
                    self.tile_set_chooser.set_selected(editor.selected());
                    *self.tile_set_chooser.tile_set_mut() = tile_set;
                    self.tile_set_chooser.refresh(true);
                    self.refresh_views();
                }
                _ => {}
            }
        }
        self.tile_set_chooser.tile_set().store(self.config.tile_set_file_name).unwrap();
        self.color_chooser.swatch().store(self.config.swatch_file_name).unwrap();
    }
}

pub enum EditMode<'r> {
    Open(&'r str),
    New(&'r str, u32, u32)
}
impl <'r>EditMode<'r> {
    fn file_name(&self) -> &'r str {
        match *self {
            EditMode::Open(r) => r,
            EditMode::New(r,_,_) => r
        }
    }
}
pub struct Config<'r> {
    pub tile_set_file_name : &'r str,
    pub swatch_file_name : &'r str,
    pub edit_mode: EditMode<'r>
}
pub fn run<'r>(c: Config<'r>) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.text_input().start();
    //let (ddpi,hdpi,vdpi) = video_subsystem.display_dpi().unwrap();
    let window = video_subsystem.window("tesseraed", 1280, 800)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas().build().unwrap();    
    canvas.set_logical_size(1280,800).unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let graphic = match c.edit_mode {
        EditMode::New(_, w, h) => 
            Graphic::blank(w,h).textured(&texture_creator),
        EditMode::Open(f) =>
            Graphic::load_file_textured(f,&texture_creator).unwrap(),
    };
    let mut editor = GraphicsEditor::create(graphic,&c,&texture_creator);
    editor.refresh_views();
    editor.main_loop(&mut canvas, &mut event_pump);
    editor.editor_view.save(&mut File::create(c.edit_mode.file_name()).unwrap()).unwrap()
}
