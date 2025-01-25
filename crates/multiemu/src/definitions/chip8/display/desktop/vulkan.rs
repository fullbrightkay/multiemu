use crate::{
    definitions::chip8::display::{draw_sprite_common, Chip8DisplayImplementation},
    runtime::rendering_backend::DisplayComponentFramebuffer,
};
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
        let staging_buffer = DMatrixViewMut::from_slice(staging_buffer.deref_mut(), 64, 32);

        draw_sprite_common(position, sprite, staging_buffer)
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

    fn get_framebuffer(&self) -> DisplayComponentFramebuffer {
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
