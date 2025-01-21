#[cfg(platform_desktop)]
pub mod desktop;
#[cfg(platform_desktop)]
pub use desktop::renderer::software::SoftwareRenderingRuntime;
#[cfg(platform_desktop)]
pub use desktop::PlatformRuntime;

#[cfg(platform_3ds)]
pub mod nintendo_3ds;
#[cfg(platform_3ds)]
pub use nintendo_3ds::renderer::software::SoftwareRenderingRuntime;
#[cfg(platform_3ds)]
pub use nintendo_3ds::PlatformRuntime;
