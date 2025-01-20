use std::sync::{Arc, Mutex};

use super::Chip8DisplayImplementation;
use crate::runtime::rendering_backend::DisplayComponentFramebuffer;
use bitvec::{prelude::Msb0, view::BitView};
use nalgebra::{DMatrix, Point2};
use palette::Srgba;

#[derive(Debug)]
pub struct SoftwareState {
    pub framebuffer: Arc<Mutex<DMatrix<Srgba<u8>>>>,
}

impl Chip8DisplayImplementation for SoftwareState {
    fn draw_sprite(&self, position: Point2<u8>, sprite: &[u8]) -> bool {
        let mut collided = false;
        let mut framebuffer = self.framebuffer.lock().unwrap();

        for (y, sprite_row) in sprite.view_bits::<Msb0>().chunks(8).enumerate() {
            for (x, sprite_pixel) in sprite_row.iter().enumerate() {
                let x = position.x as usize + x;
                let y = position.y as usize + y;

                if x >= 64 || y >= 32 {
                    continue;
                }

                let old_sprite_pixel = framebuffer[(x, y)] == Srgba::new(255, 255, 255, 255);

                if *sprite_pixel && old_sprite_pixel {
                    collided = true;
                }

                framebuffer[(x, y)] = if *sprite_pixel ^ old_sprite_pixel {
                    Srgba::new(255, 255, 255, 255)
                } else {
                    Srgba::new(0, 0, 0, 255)
                };
            }
        }

        collided
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
