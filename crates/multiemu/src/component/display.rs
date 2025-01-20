use super::Component;
use crate::runtime::rendering_backend::{
    DisplayComponentFramebuffer, DisplayComponentInitializationData,
};

pub trait DisplayComponent: Component {
    fn set_display_data(&self, display_data: DisplayComponentInitializationData);
    fn get_framebuffer(&self) -> DisplayComponentFramebuffer;
}
