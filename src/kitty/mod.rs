mod detect_support;
mod image_renderer;
mod terminal_esc;

pub use image_renderer::*;

use crate::display::cell_size_in_pixels;

use {
    crate::{
        display::{
            Area,
            W,
        },
        errors::ProgramError,
        image::SourceImage,
        kitty::detect_support::is_tmux,
    },
    crokey::crossterm::style::Color,
    once_cell::sync::Lazy,
    std::{
        path::Path,
        sync::Mutex,
    },
};

pub type KittyImageId = usize;

static MANAGER: Lazy<Mutex<KittyManager>> = Lazy::new(|| {
    let manager = KittyManager {
        renderer: MaybeRenderer::Untested,
    };
    Mutex::new(manager)
});

pub fn manager() -> &'static Mutex<KittyManager> {
    &MANAGER
}

#[derive(Debug)]
pub struct KittyManager {
    renderer: MaybeRenderer,
}

#[derive(Debug)]
enum MaybeRenderer {
    Untested,
    Disabled,
    Enabled { renderer: KittyImageRenderer },
}

impl KittyManager {
    /// return the renderer if it's already checked and enabled, none if
    /// it's disabled or if it hasn't been tested yet
    pub fn renderer_if_tested(&mut self) -> Option<&mut KittyImageRenderer> {
        match &mut self.renderer {
            MaybeRenderer::Enabled { renderer } => Some(renderer),
            _ => None,
        }
    }
    pub fn renderer(&mut self) -> Option<&mut KittyImageRenderer> {
        if matches!(self.renderer, MaybeRenderer::Disabled) {
            return None;
        }
        if matches!(self.renderer, MaybeRenderer::Enabled { .. }) {
            return self.renderer_if_tested();
        }
        let options = KittyImageRendererOptions {
            display: KittyGraphicsDisplay::Unicode,
            transmission_medium: TransmissionMedium::TempFile,
            kept_temp_files: std::num::NonZero::new(1)?,
            is_tmux: is_tmux(),
        };
        match KittyImageRenderer::new(options) {
            Some(renderer) => {
                self.renderer = MaybeRenderer::Enabled { renderer };
                self.renderer_if_tested()
            }
            None => {
                self.renderer = MaybeRenderer::Disabled;
                None
            }
        }
    }
    #[allow(clippy::too_many_arguments)] // yes, I know
    pub fn try_print_image(
        &mut self,
        w: &mut W,
        src: &SourceImage,
        src_path: &Path, // used to build a cache key
        area: &Area,
        bg: Color,
    ) -> Result<Option<KittyImageId>, ProgramError> {
        if let Some(renderer) = self.renderer() {
            let (cell_width, cell_height) = cell_size_in_pixels()?;
            let area_width = area.width as u32 * cell_width;
            let area_height = area.height as u32 * cell_height;
            let img = src.fitting(area_width, area_height)?;
            let new_id = renderer.print(w, &img, src_path, area, bg)?;
            Ok(Some(new_id))
        } else {
            Ok(None)
        }
    }
}
