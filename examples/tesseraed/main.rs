extern crate bit_reverse;
extern crate clap;
use clap::{Arg, App, SubCommand};
pub mod editor;
use editor::{Config,EditMode,run};
fn main() {
    let matches = App::new("tesseraed")
        .version("1.0")
        .author("Liam O'Connor")
        .about("Editor for tesserae tiles and graphics.")
        .arg(Arg::with_name("tileset")
                .short("t")
                .long("tileset")
                .value_name("FILE")
                .help("Tileset to use/edit")
                .takes_value(true) )
        .arg(Arg::with_name("swatch")
                .short("s")
                .long("swatch")
                .value_name("FILE")
                .help("Swatch colorset to use/edit")
                .takes_value(true) )
        .subcommand(SubCommand::with_name("open")
                .about("open an existing graphic file")
                .arg(Arg::with_name("filename").required(true)))
        .subcommand(SubCommand::with_name("new")
                .about("create a new graphic file")
                .arg(Arg::with_name("filename").required(true))
                .arg(Arg::with_name("width").required(true))
                .arg(Arg::with_name("height").required(true))).get_matches();

    let config = Config {
        tile_set_file_name: matches.value_of("tileset").unwrap_or("tile_set"),
        swatch_file_name: matches.value_of("swatch").unwrap_or("swatch"),
        edit_mode:  if let Some(matches) = matches.subcommand_matches("open") {
            EditMode::Open(matches.value_of("filename").unwrap())
        } else if let Some(matches) = matches.subcommand_matches("new") {
            EditMode::New(matches.value_of("filename").unwrap(),
                matches.value_of("width").unwrap().parse().unwrap(),
                matches.value_of("height").unwrap().parse().unwrap())
        } else {
            println!("{}",matches.usage());
            std::process::exit(0)
        }
    };    
    run(config);
}
