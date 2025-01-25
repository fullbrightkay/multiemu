use crate::machine::Machine;
use egui::FullOutput;
use nalgebra::DMatrix;
use palette::Srgba;
use std::sync::{Arc, Mutex};

pub enum DisplayComponentInitializationData {
    Software,
    #[cfg(graphics_vulkan)]
    Vulkan(super::platform::desktop::renderer::vulkan::VulkanDisplayComponentInitializationData),
}

#[derive(Clone)]
pub enum DisplayComponentFramebuffer {
    Software(Arc<Mutex<DMatrix<Srgba<u8>>>>),
    #[cfg(graphics_vulkan)]
    Vulkan(Arc<vulkano::image::Image>),
}

pub trait RenderingBackendState: Sized {
    type DisplayApiHandle: Clone + 'static;

    fn new(display_api_handle: Self::DisplayApiHandle) -> Self;
    fn redraw(&mut self, machine: &Machine);
    fn redraw_menu(&mut self, egui_context: &egui::Context, full_output: FullOutput);
    fn surface_resized(&mut self) {}
    fn initialize_machine(&mut self, machine: &Machine);
}
