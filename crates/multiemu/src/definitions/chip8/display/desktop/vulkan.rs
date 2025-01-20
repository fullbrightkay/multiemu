use crate::{
    definitions::chip8::display::Chip8DisplayImplementation,
    runtime::rendering_backend::DisplayComponentFramebuffer,
};
use bitvec::{prelude::Msb0, view::BitView};
use nalgebra::{DMatrix, DMatrixViewMut, Point2};
use palette::Srgba;
use std::{ops::DerefMut, sync::Arc};
use vulkano::{
    buffer::Subbuffer,
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferToImageInfo, PrimaryCommandBufferAbstract,
    },
    device::Queue,
    image::Image,
    sync::GpuFuture,
};

#[derive(Debug)]
pub struct VulkanState {
    pub staging_buffer: Subbuffer<[Srgba<u8>]>,
    pub render_image: Arc<Image>,
    pub queue: Arc<Queue>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
}

impl Chip8DisplayImplementation for VulkanState {
    fn draw_sprite(&self, position: Point2<u8>, sprite: &[u8]) -> bool {
        let mut staging_buffer = self.staging_buffer.write().unwrap();
        let mut staging_buffer = DMatrixViewMut::from_slice(staging_buffer.deref_mut(), 64, 32);

        let mut collided = false;

        for (y, sprite_row) in sprite.view_bits::<Msb0>().chunks(8).enumerate() {
            for (x, sprite_pixel) in sprite_row.iter().enumerate() {
                let x = position.x as usize + x;
                let y = position.y as usize + y;

                if x >= 64 || y >= 32 {
                    continue;
                }

                let old_sprite_pixel = staging_buffer[(x, y)] == Srgba::new(255, 255, 255, 255);

                if *sprite_pixel && old_sprite_pixel {
                    collided = true;
                }

                staging_buffer[(x, y)] = if *sprite_pixel ^ old_sprite_pixel {
                    Srgba::new(255, 255, 255, 255)
                } else {
                    Srgba::new(0, 0, 0, 255)
                };
            }
        }

        collided
    }

    fn clear_display(&self) {
        let mut staging_buffer = self.staging_buffer.write().unwrap();
        staging_buffer.fill(Srgba::new(0, 0, 0, 255));
    }

    fn save_screen_contents(&self) -> DMatrix<Srgba<u8>> {
        let staging_buffer = self.staging_buffer.read().unwrap();
        DMatrix::from_vec(64, 32, staging_buffer.to_vec())
    }

    fn load_screen_contents(&self, buffer: DMatrix<Srgba<u8>>) {
        let mut staging_buffer = self.staging_buffer.write().unwrap();
        staging_buffer.copy_from_slice(buffer.as_slice());
    }

    fn get_framebuffer(& self) -> DisplayComponentFramebuffer {
        DisplayComponentFramebuffer::Vulkan(self.render_image.clone())
    }

    fn commit_display(&self) {
        let mut command_buffer = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        command_buffer
            // Copy the staging buffer to the image
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                self.staging_buffer.clone(),
                self.render_image.clone(),
            ))
            .unwrap();
        command_buffer
            .build()
            .unwrap()
            .execute(self.queue.clone())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();
    }
}
