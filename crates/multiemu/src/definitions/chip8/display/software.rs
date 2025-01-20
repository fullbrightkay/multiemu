use std::sync::{Arc, Mutex};
use super::{draw_sprite_common, Chip8DisplayImplementation};
use crate::runtime::rendering_backend::DisplayComponentFramebuffer;
use nalgebra::{DMatrix, Point2};
use palette::Srgba;

#[derive(Debug)]
pub struct SoftwareState {
    pub framebuffer: Arc<Mutex<DMatrix<Srgba<u8>>>>,
}

impl Chip8DisplayImplementation for SoftwareState {
    fn draw_sprite(&self, position: Point2<u8>, sprite: &[u8]) -> bool {
        let mut framebuffer = self.framebuffer.lock().unwrap();

        draw_sprite_common(position, sprite, framebuffer.as_view_mut())
    }

    fn clear_display(&self) {
        self.framebuffer
            .lock()
            .unwrap()
            .fill(Srgba::new(0, 0, 0, 255));
    }

    fn save_screen_contents(&self) -> DMatrix<Srgba<u8>> {
        self.framebuffer.lock().unwrap().clone()
    }

    fn load_screen_contents(&self, buffer: DMatrix<Srgba<u8>>) {
        self.framebuffer.lock().unwrap().clone_from(&buffer);
    }

    fn get_framebuffer(&self) -> DisplayComponentFramebuffer {
        DisplayComponentFramebuffer::Software(self.framebuffer.clone())
    }

    fn commit_display(&self) {
        // We don't use an extra staging buffer
    }
}
