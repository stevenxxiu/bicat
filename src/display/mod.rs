mod area;
mod cell_size;
mod fill_bg;
mod filling;

pub use area::Area;
pub use cell_size::cell_size_in_pixels;
pub use fill_bg::fill_bg;
pub use filling::Filling;

pub type W = std::io::BufWriter<std::io::Stderr>;
