use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        // Means a desktop platform, indicates we will use winit/cpal/gilrs/vulkan. Android is considered a desktop platform here cuz yeah
        platform_desktop: {
            all(
                any(
                    target_family = "unix",
                    target_os = "windows"
                ),
                // HACK: The 3ds is marked as a unix like despite not being one
                not(target_os = "horizon")
            )
        },
        platform_3ds: {
            target_os = "horizon"
        },
        // Mere speculative at this moment considering the rust port to the psp has not hit std support yet
        platform_psp: {
            target_os = "psp"
        },
        graphics_vulkan: {
            all(
                any(
                    target_family = "unix",
                    target_os = "windows"
                ),
                // HACK: The 3ds is marked as a unix like despite not being one
                not(target_os = "horizon"),
                feature = "vulkan"
            )
        },
    }
}
