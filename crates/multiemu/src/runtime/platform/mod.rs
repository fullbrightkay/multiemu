#[cfg(desktop)]
pub mod desktop;
#[cfg(desktop)]
pub use desktop::renderer::software::SoftwareRenderingRuntime;
#[cfg(desktop)]
pub use desktop::PlatformRuntime;

#[cfg(nintendo_3ds)]
pub mod nintendo_3ds;
#[cfg(nintendo_3ds)]
pub use nintendo_3ds::renderer::software::SoftwareRenderingRuntime;
#[cfg(nintendo_3ds)]
pub use nintendo_3ds::PlatformRuntime;
