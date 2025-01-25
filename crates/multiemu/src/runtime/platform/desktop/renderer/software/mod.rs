use crate::{
    component::display::DisplayComponent,
    gui::software_rasterizer::SoftwareEguiRenderer,
    machine::Machine,
    runtime::rendering_backend::{
        DisplayComponentFramebuffer, DisplayComponentInitializationData, RenderingBackendState,
    },
};
use nalgebra::{DMatrixViewMut, Vector2};
use palette::Srgba;
use softbuffer::{Context, Surface};
use std::{num::NonZero, sync::Arc};
use winit::window::Window;

pub struct SoftwareRenderingRuntime {
    surface: Surface<Arc<Window>, Arc<Window>>,
    display_api_handle: Arc<Window>,
    egui_renderer: SoftwareEguiRenderer,
}

impl RenderingBackendState for SoftwareRenderingRuntime {
    type DisplayApiHandle = Arc<Window>;

    fn new(display_api_handle: Self::DisplayApiHandle) -> Self {
        let window_dimensions = display_api_handle.inner_size();
        let window_dimensions = Vector2::new(
            NonZero::new(window_dimensions.width).unwrap(),
            NonZero::new(window_dimensions.height).unwrap(),
        );

        let context = Context::new(display_api_handle.clone()).unwrap();
        let mut surface = Surface::new(&context, display_api_handle.clone()).unwrap();

        surface
            .resize(window_dimensions.x, window_dimensions.y)
            .unwrap();

        Self {
            surface,
            display_api_handle,
            egui_renderer: SoftwareEguiRenderer::default(),
        }
    }

    fn surface_resized(&mut self) {
        let window_dimensions = self.display_api_handle.inner_size();
        let window_dimensions = Vector2::new(window_dimensions.width, window_dimensions.height);

        self.surface
            .resize(
                window_dimensions.x.try_into().unwrap(),
                window_dimensions.y.try_into().unwrap(),
            )
            .unwrap();
    }

    fn redraw(&mut self, machine: &Machine) {
        let window_dimensions = self.display_api_handle.inner_size();
        let window_dimensions = Vector2::new(window_dimensions.width, window_dimensions.height);

        // HACK: This only works with a single component
        let component_info = machine.display_components().next().unwrap();
        let DisplayComponentFramebuffer::Software(display_component_framebuffer) =
            component_info.component.get_framebuffer()
        else {
            unreachable!()
        };
        let display_component_framebuffer = display_component_framebuffer.lock().unwrap();

        // Skip rendering if impossible window size
        if window_dimensions.min() == 0 {
            return;
        }

        let mut surface_buffer = self.surface.buffer_mut().unwrap();
        let mut surface_buffer_view = DMatrixViewMut::from_slice(
            bytemuck::cast_slice_mut(surface_buffer.as_mut()),
            window_dimensions.x as usize,
            window_dimensions.y as usize,
        );

        // Clear the surface buffer
        surface_buffer_view.fill(Srgba::<u8>::new(0, 0, 0, 0xff));

        let component_display_buffer_size = Vector2::new(
            display_component_framebuffer.nrows(),
            display_component_framebuffer.ncols(),
        );

        let scaling = window_dimensions
            .cast::<f32>()
            .component_div(&component_display_buffer_size.cast::<f32>());

        // Iterate over each pixel in the display component buffer
        for x in 0..display_component_framebuffer.nrows() {
            for y in 0..display_component_framebuffer.ncols() {
                let source_pixel = display_component_framebuffer[(x, y)];

                let dest_start = Vector2::new(x, y)
                    .cast::<f32>()
                    .component_mul(&scaling)
                    .map(f32::round)
                    .try_cast::<usize>()
                    .unwrap()
                    .zip_map(&window_dimensions, |dest_dim, window_dim| {
                        dest_dim.min(window_dim as usize)
                    });

                let dest_end = Vector2::new(x, y)
                    .cast::<f32>()
                    .add_scalar(1.0)
                    .component_mul(&scaling)
                    .map(f32::round)
                    .try_cast::<usize>()
                    .unwrap()
                    .zip_map(&window_dimensions, |dest_dim, window_dim| {
                        dest_dim.min(window_dim as usize)
                    });

                // Fill the destination pixels with the source pixel
                let mut destination_pixels = surface_buffer_view.view_mut(
                    (dest_start.x, dest_start.y),
                    (dest_end.x - dest_start.x, dest_end.y - dest_start.y),
                );

                destination_pixels.fill(source_pixel);
            }
        }

        surface_buffer.present().unwrap();
    }

    fn redraw_menu(&mut self, egui_context: &egui::Context, full_output: egui::FullOutput) {
        let window_dimensions = self.display_api_handle.inner_size();
        let window_dimensions = Vector2::new(window_dimensions.width, window_dimensions.height);

        let mut surface_buffer = self.surface.buffer_mut().unwrap();
        let surface_buffer_view = DMatrixViewMut::from_slice(
            bytemuck::cast_slice_mut(surface_buffer.as_mut()),
            window_dimensions.x as usize,
            window_dimensions.y as usize,
        );

        self.egui_renderer
            .render(egui_context, surface_buffer_view, full_output);

        surface_buffer.present().unwrap();
    }

    fn initialize_machine(&mut self, machine: &Machine) {
        for component_info in machine.display_components() {
            component_info
                .component
                .set_display_data(DisplayComponentInitializationData::Software);
        }
    }
}
