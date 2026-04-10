mod display;
mod errors;
mod image;
mod kitty;

use std::io::{
    BufWriter,
    Write,
};
use std::path::PathBuf;

use clap::Parser;
use cli_log::*;
use crossterm::cursor::MoveTo;
use crossterm::style::{
    ResetColor,
    SetForegroundColor,
};
use crossterm::{
    queue,
    style::{
        Color,
        Print,
    },
};
use display::Area;

use crate::{
    errors::ProgramError,
    image::SourceImage,
};

#[derive(Parser, Debug)]
struct Args {
    path: PathBuf,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    #[arg(long)]
    fg: Option<u8>,
    #[arg(long)]
    bg: Option<u8>,
}

pub fn main() -> Result<(), ProgramError> {
    init_cli_log!();

    let args = Args::parse();

    let area = Area {
        left: args.x,
        top: args.y,
        width: args.w,
        height: args.h,
    };

    // When using the file transmission medium, PNG files are described by paths, so use an absolute path
    let path_buf = std::path::absolute(args.path.as_path())?;
    let path = path_buf.as_path();

    let source_img = time!("decode image", path, SourceImage::new(path)?);

    let mut w = BufWriter::new(std::io::stderr());

    let mut kitty_manager = kitty::manager().lock().unwrap();
    let fg = args.fg.map(Color::AnsiValue).unwrap_or(Color::Reset);
    let bg = args.bg.map(Color::AnsiValue).unwrap_or(Color::Reset);
    let _kitty_image_id = kitty_manager.try_print_image(&mut w, &source_img, path, &area, bg)?;
    w.flush().unwrap();

    let dim = source_img.dimensions();
    let s = format!("{} x {}", dim.0, dim.1);
    if s.len() > area.width as usize {
        return Ok(());
    }
    queue!(w, MoveTo(area.left, area.top + area.height)).unwrap();
    queue!(w, SetForegroundColor(fg), Print(s), ResetColor).unwrap();
    w.flush().unwrap();

    Ok(())
}
