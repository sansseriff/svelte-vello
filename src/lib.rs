mod utils;

use leptos::prelude::Read;
use skrifa::raw::tables::variations::Tuple;
use vello::util::{RenderContext, RenderSurface};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
// use web_sys::VideoEncoder;

use std::num::NonZeroUsize;
use vello::{
    kurbo::{Affine, Circle, Rect},
    peniko::{Color, Fill},
    *,
};

// use reactive_graph::create_signal;

// use leptos::*;
use leptos::prelude::Get;
use leptos::prelude::Set;

use reactive_graph::signal::{signal, ReadSignal, WriteSignal};

// use std::collections::VecDeque;
// use std::sync::{Arc, Mutex};

use wgpu;

use web_sys::Window;

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

pub struct Node {
    pub x: IrSignal,
    pub y: IrSignal,
}

impl Node {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: IrSignal::new(x),
            y: IrSignal::new(y),
        }
    }

    // fn x(&self) -> &IrSignal {
    //     &self.x
    // }
    // fn y(&self) -> &IrSignal {
    //     &self.y
    // }
}

// Define base Shape trait
pub trait Shape {
    // fn new(x: f64, y: f64, color: Color) -> Self;
    fn contains(&self, x: f64, y: f64) -> bool;
    fn draw(&self, scene: &mut Scene);
    fn node(&self) -> &Node;
}

pub struct IrSignal {
    pub get: ReadSignal<f64>,
    pub set: WriteSignal<f64>,
}

impl IrSignal {
    fn new(value: f64) -> Self {
        let (get, set) = signal(value);
        Self { get, set }
    }

    fn get(&self) -> f64 {
        self.get.get()
    }

    fn set(&self, value: f64) {
        self.set.set(value);
    }
}

pub struct IrRectangle {
    pub node: Node,
    pub width: IrSignal,
    pub height: IrSignal,
    pub color: Color,
}

impl IrRectangle {
    pub fn new(x: f64, y: f64, width: f64, height: f64, color: Color) -> Self {
        Self {
            node: Node::new(x, y),
            width: IrSignal::new(width),
            height: IrSignal::new(height),
            color,
        }
    }
}

impl Shape for IrRectangle {
    fn node(&self) -> &Node {
        &self.node
    }

    fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.node.x.get()
            && px <= self.node.x.get() + self.width.get()
            && py >= self.node.y.get()
            && py <= self.node.y.get() + self.height.get()
    }

    fn draw(&self, scene: &mut Scene) {
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.color,
            None,
            &Rect::new(
                self.node.x.get(),
                self.node.y.get(),
                self.node.x.get() + self.width.get(),
                self.node.y.get() + self.height.get(),
            ),
        );
    }
}

// Circle implementation
pub struct IrCircle {
    pub node: Node,
    pub radius: IrSignal,
    pub color: Color,
}

impl IrCircle {
    pub fn new(x: f64, y: f64, radius: f64, color: Color) -> Self {
        Self {
            node: Node::new(x, y),
            radius: IrSignal::new(radius),
            color,
        }
    }
}

impl Shape for IrCircle {
    fn contains(&self, px: f64, py: f64) -> bool {
        let dx = self.node.x.get() - px;
        let dy = self.node.y.get() - py;
        (dx * dx + dy * dy).sqrt() <= self.radius.get()
    }

    fn draw(&self, scene: &mut Scene) {
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.color,
            None,
            &Circle::new((self.node.x.get(), self.node.y.get()), self.radius.get()),
        );
    }

    fn node(&self) -> &Node {
        &self.node
    }
}

struct RenderState<'s> {
    // SAFETY: We MUST drop the surface before the `window`, so the fields
    // must be in this order
    surface: RenderSurface<'s>,
    window: Window,
}

#[wasm_bindgen]
pub struct VelloContext {
    shapes: Vec<Box<dyn Shape>>,
    selected_shape: Option<usize>,
    drag_start_x: f64,
    drag_start_y: f64,
    canvas: HtmlCanvasElement,
    render_cx: RenderContext,
    state: RenderState<'static>,
    renderer: Renderer,
}

//added recently. Need to use this to keep track of RenderSurface... Maybe window too?

#[wasm_bindgen]
impl VelloContext {
    #[wasm_bindgen]
    pub async fn create(canvas_id: &str) -> Result<VelloContext, JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        let mut render_cx = RenderContext::new();
        let width = canvas.width();
        let height = canvas.height();

        let surface = render_cx
            .create_surface(
                wgpu::SurfaceTarget::Canvas(canvas.clone()),
                width,
                height,
                wgpu::PresentMode::AutoVsync,
            )
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let render_state = RenderState {
            surface,
            window: window,
        };
        // hmm interesting. I move stuff into the struct, and then I
        // can't access the stuff outside of the struct anymore.

        let id = render_state.surface.dev_id;

        let renderer = Renderer::new(
            &render_cx.devices[id].device,
            RendererOptions {
                surface_format: Some(render_state.surface.format),
                use_cpu: false,
                antialiasing_support: AaSupport::all(),
                num_init_threads: NonZeroUsize::new(1),
            },
        )
        .expect("Failed to create renderer");

        console_log!("renderer created");

        let context = VelloContext {
            shapes: Vec::new(),
            selected_shape: None,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            canvas,
            render_cx,
            state: render_state,
            renderer,
        };

        Ok(context)
    }

    pub fn add_rectangle(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) {
        self.shapes.push(Box::new(IrRectangle::new(
            x,
            y,
            width,
            height,
            Color::rgba8(r, g, b, a),
        )));
        self.render();
    }

    pub fn add_circle(&mut self, x: f64, y: f64, radius: f64, r: u8, g: u8, b: u8, a: u8) {
        self.shapes.push(Box::new(IrCircle::new(
            x,
            y,
            radius,
            Color::rgba8(r, g, b, a),
        )));
        self.render();
    }

    pub fn handle_mouse_down(&mut self, x: f64, y: f64) {
        self.selected_shape = self.shapes.iter().position(|shape| shape.contains(x, y));
        if self.selected_shape.is_some() {
            self.drag_start_x = x;
            self.drag_start_y = y;
        }
    }

    pub fn handle_mouse_move(&mut self, x: f64, y: f64) {
        if let Some(idx) = self.selected_shape {
            let dx = x - self.drag_start_x;
            let dy = y - self.drag_start_y;

            self.shapes[idx]
                .node()
                .x
                .set(self.shapes[idx].node().x.get() + dx);

            self.shapes[idx]
                .node()
                .y
                .set(self.shapes[idx].node().y.get() + dy);

            // self.shapes[idx].set_p

            self.drag_start_x = x;
            self.drag_start_y = y;

            self.render();
        }
    }

    pub fn handle_mouse_up(&mut self) {
        self.selected_shape = None;
    }

    fn render(&mut self) {
        let width = self.canvas.width();
        let height = self.canvas.height();
        // let shapes = self.shapes.clone();

        // Build scene
        let mut scene = Scene::new();
        for shape in &self.shapes {
            shape.draw(&mut scene);
        }

        // Render to surface
        let surface_texture = self
            .state
            .surface
            .surface
            .get_current_texture()
            .expect("Failed to get current texture");

        let id = self.state.surface.dev_id;

        self.renderer
            .render_to_surface(
                &self.render_cx.devices[id].device,
                &self.render_cx.devices[id].queue,
                &scene,
                &surface_texture,
                &RenderParams {
                    base_color: Color::rgba8(240, 240, 240, 255),
                    width,
                    height,
                    antialiasing_method: AaConfig::Msaa8,
                },
            )
            .expect("Failed to render to surface");

        surface_texture.present();
        // });
    }
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
