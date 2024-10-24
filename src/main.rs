mod windowed_context;

use std::time::Instant;
use dioxus::desktop::tao::event::Event as WryEvent;
use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::desktop::{use_window, use_wry_event_handler, window, DesktopContext};
use dioxus::prelude::*;
use ouroboros::self_referencing;
use std::borrow::Cow;
use wgpu;
use crate::manganis;
use three_d::SurfaceSettings;
use three_d::{Geometry, FrameInput, Camera, OrbitControl, Model, Viewport, vec3, degrees, ColorMaterial, Mesh, Gm, Srgba, Mat4, CpuMesh, radians, RenderTarget, ClearState};

// Urls are relative to your Cargo.toml file
const _TAILWIND_URL: &str = manganis::mg!(file("public/tailwind.css"));

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(WindowBuilder::new().with_transparent(true))
        .with_as_child_window();
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app);
}

struct GraphicsResources {
    context: windowed_context::WindowedContext,
    camera: Camera,
    control: OrbitControl,
    model: Gm<Mesh, ColorMaterial>,
    time_since_start: Instant,
}

fn app() -> Element {
    let mut graphics_resources = use_signal(|| {
        println!("recreating resources");
        let desktop_context = window();
        let window = &desktop_context.window;
        let context = windowed_context::WindowedContext::from_tao_window(window, SurfaceSettings::default()).unwrap();
        // Create camera
        let mut camera = Camera::new_perspective(
            Viewport::new_at_origo(1, 1),
            vec3(0.0, 2.0, 4.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            10.0,
        );
        let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

        // Create model
        let mut model = Gm::new(
            Mesh::new(&context, &CpuMesh::cube()),
            ColorMaterial {
                color: Srgba::GREEN,
                ..Default::default()
            },
        );
        model.set_animation(|time| Mat4::from_angle_y(radians(time * 0.0005)));
        GraphicsResources {
            context,
            camera,
            control,
            model,
            time_since_start: Instant::now(),
        }
    });

    let _: Coroutine<()> = use_coroutine(|_rx| async move {
        loop {
            window().window.request_redraw();
            tokio::time::sleep(tokio::time::Duration::from_secs_f64(1.0 / 90.0)).await;
        }
    });

    let desktop_context = use_window();

    use_wry_event_handler(move |event, _| {
        match event {
            WryEvent::RedrawRequested(_id) => {}
            WryEvent::WindowEvent {
                event: dioxus::desktop::tao::event::WindowEvent::Resized(_new_size),
                ..
            } => {}
            WryEvent::MainEventsCleared => {
                let window = &desktop_context.window;
                graphics_resources.with_mut(|graphics_resources| {
                    let mut events = Vec::new();
                    graphics_resources.control.handle_events(&mut graphics_resources.camera, &mut events);
                    graphics_resources.model.animate(Instant::now().duration_since(graphics_resources.time_since_start).as_millis() as f32);
                    let viewport = Viewport { x: 0, y: 0, width: window.inner_size().width / 2, height: window.inner_size().height / 2};
                    graphics_resources.camera.set_viewport(viewport);
                    RenderTarget::screen(&graphics_resources.context, viewport.width, viewport.height)
                        .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
                        .render(&graphics_resources.camera, &graphics_resources.model, &[]);
                    graphics_resources.context.swap_buffers().unwrap();
                })
            }
            _ => {}
        }
    });
    
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./public/tailwind.css") }
        header {
            class: "text-gray-400 body-font",
            div { class: "container mx-auto flex flex-wrap p-5 flex-col md:flex-row items-center",
                a { class: "flex title-font font-medium items-center text-white mb-4 md:mb-0",
                    StacksIcon {}
                    span { class: "ml-3 text-xl", "Hello Dioxus!" }
                }
                nav { class: "md:ml-auto flex flex-wrap items-center text-base justify-center",
                    a { class: "mr-5 hover:text-white", "First Link" }
                    a { class: "mr-5 hover:text-white", "Second Link" }
                    a { class: "mr-5 hover:text-white", "Third Link" }
                    a { class: "mr-5 hover:text-white", "Fourth Link" }
                }
                button { class: "inline-flex items-center bg-gray-800 border-0 py-1 px-3 focus:outline-none hover:bg-gray-700 rounded text-base mt-4 md:mt-0",
                    "Button"
                    RightArrowIcon {}
                }
            }
        }
    }
}
pub fn StacksIcon() -> Element {
    rsx!(
        svg {
            fill: "none",
            stroke: "currentColor",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            stroke_width: "2",
            class: "w-10 h-10 text-white p-2 bg-indigo-500 rounded-full",
            view_box: "0 0 24 24",
            path { d: "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" }
        }
    )
}
pub fn RightArrowIcon() -> Element {
    rsx!(
        svg {
            fill: "none",
            stroke: "currentColor",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            stroke_width: "2",
            class: "w-4 h-4 ml-1",
            view_box: "0 0 24 24",
            path { d: "M5 12h14M12 5l7 7-7 7" }
        }
    )
}