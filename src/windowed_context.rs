use three_d::Context;
use three_d::SurfaceSettings;
use three_d::WindowError;
use std::sync::Arc;
use dioxus::desktop::tao::{dpi::PhysicalSize, window::Window};

use glutin::{prelude::PossiblyCurrentContextGlSurfaceAccessor, surface::*};
pub struct WindowedContext {
    pub context: Context,
    surface: Surface<WindowSurface>,
    glutin_context: glutin::context::PossiblyCurrentContext,
}

impl std::ops::Deref for WindowedContext {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl WindowedContext {
    /// Creates a new windowed context from a [winit](https://crates.io/crates/winit) window.
    #[allow(unsafe_code)]
    pub fn from_tao_window(
        window: &Window,
        settings: SurfaceSettings,
    ) -> Result<Self, WindowError> {
        if settings.multisamples > 0 && !settings.multisamples.is_power_of_two() {
            Err(WindowError::InvalidNumberOfMSAASamples)?;
        }
        use glutin::prelude::*;
        use raw_window_handle::*;
        let raw_display_handle = window.raw_display_handle();
        let raw_window_handle = window.raw_window_handle();

        // EGL is crossplatform and the official khronos way
        // but sometimes platforms/drivers may not have it, so we use back up options
        // where possible. TODO: check whether we can expose these options as
        // "features", so that users can select the relevant backend they want.

        // try egl and fallback to windows wgl. Windows is the only platform that
        // *requires* window handle to create display.
        #[cfg(target_os = "windows")]
        let preference =
            glutin::display::DisplayApiPreference::WglThenEgl(Some(raw_window_handle));
        // try egl and fallback to x11 glx
        #[cfg(target_os = "linux")]
        let preference = glutin::display::DisplayApiPreference::EglThenGlx(Box::new(
            winit::platform::x11::register_xlib_error_hook,
        ));
        #[cfg(target_os = "macos")]
        let preference = glutin::display::DisplayApiPreference::Cgl;
        #[cfg(target_os = "android")]
        let preference = glutin::display::DisplayApiPreference::Egl;

        let gl_display =
            unsafe { glutin::display::Display::new(raw_display_handle, preference)? };
        let swap_interval = if settings.vsync {
            glutin::surface::SwapInterval::Wait(std::num::NonZeroU32::new(1).unwrap())
        } else {
            glutin::surface::SwapInterval::DontWait
        };

        let hardware_acceleration = match settings.hardware_acceleration {
            three_d::HardwareAcceleration::Required => Some(true),
            three_d::HardwareAcceleration::Preferred => None,
            three_d::HardwareAcceleration::Off => Some(false),
        };
        let config_template = glutin::config::ConfigTemplateBuilder::new()
            .prefer_hardware_accelerated(hardware_acceleration)
            .with_depth_size(settings.depth_buffer);
        // we don't know if multi sampling option is set. so, check if its more than 0.
        let config_template = if settings.multisamples > 0 {
            config_template.with_multisampling(settings.multisamples)
        } else {
            config_template
        };
        let config_template = config_template
            .with_stencil_size(settings.stencil_buffer)
            .compatible_with_native_window(raw_window_handle)
            .build();
        // finds all valid configurations supported by this display that match the
        // config_template this is where we will try to get a "fallback" config if
        // we are okay with ignoring some native options required by user like multi
        // sampling, srgb, transparency etc..
        let config = unsafe {
            gl_display
                .find_configs(config_template)?
                .next()
                .ok_or(WindowError::SurfaceCreationError)?
        };

        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(Some(raw_window_handle));
        // for surface creation.
        let (width, height): (u32, u32) = window.inner_size().into();
        let width = std::num::NonZeroU32::new(width.max(1)).unwrap();
        let height = std::num::NonZeroU32::new(height.max(1)).unwrap();
        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(raw_window_handle, width, height);
        // start creating the gl objects
        let gl_context = unsafe { gl_display.create_context(&config, &context_attributes)? };

        let gl_surface =
            unsafe { gl_display.create_window_surface(&config, &surface_attributes)? };
        let gl_context = gl_context.make_current(&gl_surface)?;
        gl_surface.set_swap_interval(&gl_context, swap_interval)?;

        Ok(Self {
            context: Context::from_gl_context(Arc::new(unsafe {
                three_d::context::Context::from_loader_function(|s| {
                    let s = std::ffi::CString::new(s)
                        .expect("failed to construct C string from string for gl proc address");

                    gl_display.get_proc_address(&s)
                })
            }))?,
            glutin_context: gl_context,
            surface: gl_surface,
        })
    }

    /// Resizes the context
    pub fn resize(&self, physical_size: PhysicalSize<u32>) {
        let width = std::num::NonZeroU32::new(physical_size.width.max(1)).unwrap();
        let height = std::num::NonZeroU32::new(physical_size.height.max(1)).unwrap();
        self.surface.resize(&self.glutin_context, width, height);
    }

    /// Make this context current. Needed when using multiple windows (contexts) on native.
    pub fn _make_current(&self) -> Result<(), WindowError> {
        Ok(self.glutin_context.make_current(&self.surface)?)
    }

    /// Swap buffers - should always be called after rendering.
    pub fn swap_buffers(&self) -> Result<(), WindowError> {
        Ok(self.surface.swap_buffers(&self.glutin_context)?)
    }

    /// Enables or disabled vsync.
    pub fn _set_vsync(&self, enabled: bool) -> Result<(), WindowError> {
        let swap_interval = if enabled {
            glutin::surface::SwapInterval::Wait(std::num::NonZeroU32::new(1).unwrap())
        } else {
            glutin::surface::SwapInterval::DontWait
        };
        Ok(self
            .surface
            .set_swap_interval(&self.glutin_context, swap_interval)?)
    }
}
