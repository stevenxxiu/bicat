use crossterm::{
    QueueableCommand,
    style::{
        Color,
        SetBackgroundColor,
    },
};
use once_cell::sync::Lazy;

use crate::display::Filling;

pub static SPACE_FILLING: Lazy<Filling> = Lazy::new(|| Filling::from_char(' '));

pub fn fill_bg<W>(
    w: &mut W,
    len: usize,
    bg: Color,
) -> Result<(), std::io::Error>
where
    W: std::io::Write,
{
    w.queue(SetBackgroundColor(bg))?;
    SPACE_FILLING.queue_unstyled(w, len)?;
    Ok(())
}
