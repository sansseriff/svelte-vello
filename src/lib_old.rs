mod utils;

use vello::util::RenderContext;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use std::num::NonZeroUsize;
use vello::{
    kurbo::{Affine, Circle},
    peniko::{Color, Fill},
    *,
};

// use vello::peniko::color:

use wgpu;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello from rust!, {}!", name)
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p")?;
    val.set_inner_html("My message yay!!");

    body.append_child(&val)?;

    Ok(())
}

#[wasm_bindgen]
pub fn setup_vello(canvas_id: &str) -> Result<(), JsValue> {
    // Get the existing canvas element
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id(canvas_id)
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    wasm_bindgen_futures::spawn_local(async move {
        let mut render_cx = RenderContext::new();
        // let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        //     backends: wgpu::Backends::BROWSER_WEBGPU,
        //     ..Default::default()
        // });

        // render_cx.instance.request_adapter(options);

        // Get canvas dimensions
        let width = canvas.width() as u32;
        let height = canvas.height() as u32;

        // https://github.com/gfx-rs/wgpu/discussions/2893
        // if I didn't have "rust-analyzer.cargo.target": "wasm32-unknown-unknown" in my .vscode/settings.json,
        // then I would have to use the cfg macro below inside a {} block to avoid the error
        // #[cfg(target_arch = "wasm32")]
        let surface_target = wgpu::SurfaceTarget::Canvas(canvas);

        let surface = render_cx
            .create_surface(surface_target, width, height, wgpu::PresentMode::AutoVsync)
            .await;

        if let Ok(surface) = surface {
            console_log!("Surface created successfully");

            let device_handle = &render_cx.devices[surface.dev_id];

            // let device = render_cx.devices[0].device;
            // let queue = render_cx.devices[0].queue;

            // // Request adapter and setup device/queue
            // let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, None)
            //     .await
            //     .ok_or("Failed to find adapter")?;

            // let surface_caps = surface.get_capabilities(surface.format);

            // let id = surface.dev_id;

            // Create renderer
            let mut renderer = Renderer::new(
                &device_handle.device,
                RendererOptions {
                    surface_format: Some(surface.format),
                    use_cpu: false,
                    antialiasing_support: AaSupport::all(),
                    num_init_threads: NonZeroUsize::new(1),
                },
            )
            .expect("Failed to create renderer");
            console_log!("Renderer created successfully");

            // Create and draw scene
            let mut scene = Scene::new();
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                Color::rgba8(242, 140, 168, 255),
                None,
                &Circle::new(((width as f32) / 2.0, (height as f32) / 2.0), 420.0),
            );

            console_log!("Scene created successfully");

            let surface_texture = surface
                .surface
                .get_current_texture()
                .expect("Failed to get current texture");

            renderer
                .render_to_surface(
                    &device_handle.device,
                    &device_handle.queue,
                    &scene,
                    &surface_texture,
                    &RenderParams {
                        base_color: Color::BLACK, // Background color
                        width,
                        height,
                        antialiasing_method: AaConfig::Msaa8,
                    },
                )
                .expect("Failed to render to surface");
        } else {
            console_log!("Failed to create surface");
        }
    });

    Ok(())
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
