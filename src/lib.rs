mod utils;

use leptos::{context, prelude::Read};
// use skrifa::raw::tables::variations::Tuple;
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

// use lazy_static::lazy_static;

// use std::collections::VecDeque;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use std::cell::RefCell;
use std::rc::Rc;

use wgpu;

use web_sys::Window;

use std::rc::Weak;

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
    pub scale_x: IrSignal,
    pub scale_y: IrSignal,
    pub rotation: IrSignal,
    pub following: Option<(usize, (f64, f64))>, // (target_index, initial_offset)
}

impl Node {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: IrSignal::new(x),
            y: IrSignal::new(y),
            scale_x: IrSignal::new(1.0),
            scale_y: IrSignal::new(1.0),
            rotation: IrSignal::new(0.0),
            following: None,
        }
    }

    pub fn start_following(&mut self, other: &Node, target_index: usize) {
        // Calculate initial offset in target's local space
        let dx = self.x.get() - other.x.get();
        let dy = self.y.get() - other.y.get();

        // Store target and offset
        self.following = Some((target_index, (dx, dy)));

        // Initial update
        self.update_transform(other);
    }

    pub fn update_transform(&mut self, target: &Node) {
        if let Some((_, (dx, dy))) = self.following {
            // Apply target's rotation to offset
            let rot = target.rotation.get();
            let cos_rot = rot.cos();
            let sin_rot = rot.sin();

            // Rotate and scale offset
            let scaled_dx = dx * target.scale_x.get();
            let scaled_dy = dy * target.scale_y.get();

            let rotated_dx = scaled_dx * cos_rot - scaled_dy * sin_rot;
            let rotated_dy = scaled_dx * sin_rot + scaled_dy * cos_rot;

            // Set new position
            self.x.set(target.x.get() + rotated_dx);
            self.y.set(target.y.get() + rotated_dy);

            // Match rotation and scale
            self.rotation.set(target.rotation.get());
            self.scale_x.set(target.scale_x.get());
            self.scale_y.set(target.scale_y.get());
        }
    }

    pub fn unfollow(&mut self) {
        self.following = None;
    }
}

// Define base Shape trait
pub trait Shape {
    // fn new(x: f64, y: f64, color: Color) -> Self;
    fn contains(&self, x: f64, y: f64) -> bool;
    fn draw(&self, scene: &mut Scene);
    fn node(&mut self) -> &mut Node;
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
    fn node(&mut self) -> &mut Node {
        &mut self.node
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

    fn node(&mut self) -> &mut Node {
        &mut self.node
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

thread_local! {
    // static CONTEXT_REGISTRY: RefCell<HashMap<u32, Rc<RefCell<VelloContext>>>> = RefCell::new(HashMap::new());
    // static NEXT_CONTEXT_ID: RefCell<u32> = RefCell::new(0);
    static ACTIVE_CONTEXT: RefCell<Option<Rc<RefCell<VelloContext>>>> = RefCell::new(None);
}

#[wasm_bindgen]
pub struct ShapeHandle {
    id: usize,
    #[wasm_bindgen(skip)]
    context: Weak<RefCell<VelloContext>>,
}

// #[wasm_bindgen]
// pub struct JsShape {
//     index: usize,
//     context_id: u32,
// }

// #[wasm_bindgen]
// impl JsShape {
//     pub fn follow(&self, other: &JsShape) -> Result<(), JsValue> {
//         CONTEXT_REGISTRY.with(|registry| {
//             let registry = registry.borrow();
//             let context = registry.get(&self.context_id).ok_or("Context not found")?;
//             let mut context = context.borrow_mut();
//             if let (Some(shape1), Some(shape2)) = (
//                 context.shapes.get_mut(self.index),
//                 context.shapes.get(other.index),
//             ) {
//                 shape1.node().start_following(shape2.node(), other.index);
//                 context.render();
//                 Ok(())
//             } else {
//                 Err(JsValue::from_str("Shape not found"))
//             }
//         })
//     }

//     pub fn unfollow(&self) -> Result<(), JsValue> {
//         CONTEXT_REGISTRY.with(|registry| {
//             let registry = registry.borrow();
//             let context = registry.get(&self.context_id).ok_or("Context not found")?;

//             let mut context = context.borrow_mut();

//             if let Some(shape) = context.shapes.get_mut(self.index) {
//                 shape.node().unfollow();
//                 context.render();
//                 Ok(())
//             } else {
//                 Err(JsValue::from_str("Shape not found"))
//             }
//         })
//     }
// }

#[wasm_bindgen]
impl ShapeHandle {
    pub fn follow(&self, other: &ShapeHandle) -> Result<(), JsValue> {
        if let Some(context) = self.context.upgrade() {
            let mut context = context.borrow_mut();
            if let (Some(shape1_follower), Some(shape2_followed)) = (
                context.shapes.get_mut(self.id),
                context.shapes.get(other.id),
            ) {
                shape1_follower
                    .node()
                    .start_following(shape2_followed.node(), other.id);
                context.render();
                Ok(())
            } else {
                Err(JsValue::from_str("Shape not found"))
            }
        } else {
            Err(JsValue::from_str("Context no longer exists"))
        }
    }

    pub fn unfollow(&self) -> Result<(), JsValue> {
        if let Some(context) = self.context.upgrade() {
            let mut context = context.borrow_mut();
            if let Some(shape) = context.shapes.get_mut(self.id) {
                shape.node().unfollow();
                context.render();
                Ok(())
            } else {
                Err(JsValue::from_str("Shape not found"))
            }
        } else {
            Err(JsValue::from_str("Context no longer exists"))
        }
    }
}

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

        let context = Rc::new(RefCell::new(VelloContext {
            shapes: Vec::new(),
            selected_shape: None,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            canvas,
            render_cx,
            state: render_state,
            renderer,
        }));

        ACTIVE_CONTEXT.with(|active| {
            *active.borrow_mut() = Some(context.clone());
        });

        Ok(VelloContext::from(context))
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
    ) -> ShapeHandle {
        let id = self.shapes.len();
        self.shapes.push(Box::new(IrRectangle::new(
            x,
            y,
            width,
            height,
            Color::from_rgba8(r, g, b, a),
        )));

        self.render();

        ShapeHandle {
            id,
            context: Rc::downgrade(&ACTIVE_CONTEXT.with(|ctx| ctx.borrow().clone().unwrap())),
        }
    }

    pub fn add_circle(
        &mut self,
        x: f64,
        y: f64,
        radius: f64,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) -> ShapeHandle {
        let id = self.shapes.len();
        self.shapes.push(Box::new(IrCircle::new(
            x,
            y,
            radius,
            Color::from_rgba8(r, g, b, a),
        )));

        self.render();

        ShapeHandle {
            id,
            context: Rc::downgrade(&ACTIVE_CONTEXT.with(|ctx| ctx.borrow().clone().unwrap())),
        }
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
                    base_color: Color::from_rgb8(240, 240, 240),
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
