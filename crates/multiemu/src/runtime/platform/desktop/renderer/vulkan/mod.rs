use crate::{
    component::display::DisplayComponent,
    config::GLOBAL_CONFIG,
    machine::Machine,
    runtime::rendering_backend::{
        DisplayComponentFramebuffer, DisplayComponentInitializationData, RenderingBackendState,
    },
};
use nalgebra::Vector2;
use std::sync::Arc;
use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, BlitImageInfo,
        CommandBufferUsage,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo, QueueFlags,
    },
    image::{sampler::Filter, view::ImageView, Image, ImageLayout, ImageUsage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    single_pass_renderpass,
    swapchain::{
        acquire_next_image, PresentMode, Surface, Swapchain, SwapchainCreateInfo,
        SwapchainPresentInfo,
    },
    sync::GpuFuture,
    Validated, VulkanError, VulkanLibrary,
};
use winit::window::Window;

pub struct VulkanRenderingRuntime {
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    device: Arc<Device>,
    gui_queue: Arc<Queue>,
    queues_for_components: Vec<Arc<Queue>>,
    swapchain: Arc<Swapchain>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    render_pass: Arc<RenderPass>,
    previous_frame_future: Option<Box<dyn GpuFuture>>,
    framebuffers: Vec<Arc<Framebuffer>>,
    swapchain_images: Vec<Arc<Image>>,
    recreate_swapchain: bool,
    display_api_handle: Arc<Window>,
}

impl RenderingBackendState for VulkanRenderingRuntime {
    type DisplayApiHandle = Arc<Window>;

    fn new(display_api_handle: Self::DisplayApiHandle) -> Self {
        let window_dimensions = display_api_handle.inner_size();
        let window_dimensions = Vector2::new(window_dimensions.width, window_dimensions.height);

        let global_config_guard = GLOBAL_CONFIG.read().unwrap();

        let library = VulkanLibrary::new().unwrap();

        tracing::info!("Found vulkan {} implementation", library.api_version());

        let required_extensions = Surface::required_extensions(&display_api_handle);
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .unwrap();
        let surface = Surface::from_window(instance.clone(), display_api_handle.clone()).unwrap();
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .unwrap();

        tracing::info!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();
        let queues: Vec<_> = queues.collect();

        tracing::info!("Using {} queue(s)", queues.len());

        let (gui_queue, queues_for_components) = if queues.len() == 1 {
            (queues[0].clone(), vec![queues[0].clone()])
        } else {
            let (gui_queue, queues) = queues.split_first().unwrap();
            (gui_queue.clone(), queues.to_vec())
        };

        let (swapchain, swapchain_images) = {
            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();
            let image_format = device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0;

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: window_dimensions.into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),
                    present_mode: if global_config_guard.vsync {
                        PresentMode::Fifo
                    } else {
                        PresentMode::Immediate
                    },
                    ..Default::default()
                },
            )
            .unwrap()
        };
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let render_pass = single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap();

        let framebuffers: Vec<Arc<Framebuffer>> = swapchain_images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect();

        drop(global_config_guard);

        Self {
            previous_frame_future: Some(vulkano::sync::now(device.clone()).boxed()),
            instance,
            surface,
            device,
            gui_queue,
            queues_for_components,
            swapchain,
            memory_allocator,
            command_buffer_allocator,
            render_pass,
            framebuffers,
            swapchain_images,
            recreate_swapchain: false,
            display_api_handle,
        }
    }

    fn surface_resized(&mut self) {
        self.recreate_swapchain = true;
    }

    fn redraw(&mut self, machine: &Machine) {
        let window_dimensions = self.display_api_handle.inner_size();
        let window_dimensions = Vector2::new(window_dimensions.width, window_dimensions.height);

        let global_config_guard = GLOBAL_CONFIG.read().unwrap();
        // HACK: This only works with a single component
        let component_info = machine.display_components().next().unwrap();

        let DisplayComponentFramebuffer::Vulkan(component_framebuffer) =
            component_info.component.get_framebuffer()
        else {
            unreachable!()
        };

        self.previous_frame_future
            .as_mut()
            .unwrap()
            .cleanup_finished();

        // Skip rendering if impossible window size
        if window_dimensions.min() == 0 {
            return;
        }

        if self.recreate_swapchain {
            tracing::trace!("Recreating swapchain");

            let (new_swapchain, new_images) = self
                .swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent: window_dimensions.into(),
                    present_mode: if global_config_guard.vsync {
                        PresentMode::Fifo
                    } else {
                        PresentMode::Immediate
                    },
                    ..self.swapchain.create_info()
                })
                .expect("Failed to recreate swapchain");

            let new_framebuffers = new_images
                .iter()
                .map(|image| {
                    let view = ImageView::new_default(image.clone()).unwrap();
                    Framebuffer::new(
                        self.render_pass.clone(),
                        FramebufferCreateInfo {
                            attachments: vec![view],
                            ..Default::default()
                        },
                    )
                    .unwrap()
                })
                .collect::<Vec<_>>();

            self.swapchain = new_swapchain;
            self.swapchain_images = new_images;
            self.framebuffers = new_framebuffers;
            self.recreate_swapchain = false;
        }

        let (image_index, recreate_swapchain, acquire_future) = {
            acquire_next_image(self.swapchain.clone(), None).expect("Failed to acquire next image")
        };
        self.recreate_swapchain |= recreate_swapchain;

        let swapchain_image = self.swapchain_images[image_index as usize].clone();

        let mut command_buffer = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.gui_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        command_buffer
            .blit_image(BlitImageInfo {
                src_image_layout: ImageLayout::TransferSrcOptimal,
                dst_image_layout: ImageLayout::TransferDstOptimal,
                filter: Filter::Nearest,
                ..BlitImageInfo::images(component_framebuffer, swapchain_image.clone())
            })
            .unwrap();

        let command_buffer = command_buffer.build().unwrap();

        // Swap that swapchain very painfully
        match self
            .previous_frame_future
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.gui_queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.gui_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush()
            .map_err(Validated::unwrap)
        {
            Ok(previous_frame_future) => {
                self.previous_frame_future = Some(Box::new(previous_frame_future));
            }
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_future = Some(vulkano::sync::now(self.device.clone()).boxed());
            }
            Err(_) => panic!("Failed to present swapchain image"),
        }
    }

    fn redraw_menu(&mut self, _egui_context: &egui::Context, _full_output: egui::FullOutput) {}

    fn initialize_machine(&mut self, machine: &Machine) {
        for (component_info, queue) in machine
            .display_components()
            .zip(self.queues_for_components.iter().cycle().cloned())
        {
            component_info
                .component
                .set_display_data(DisplayComponentInitializationData::Vulkan(
                    VulkanDisplayComponentInitializationData {
                        device: self.device.clone(),
                        queue,
                        memory_allocator: self.memory_allocator.clone(),
                        command_buffer_allocator: self.command_buffer_allocator.clone(),
                    },
                ))
        }
    }
}

pub struct VulkanDisplayComponentInitializationData {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
}
