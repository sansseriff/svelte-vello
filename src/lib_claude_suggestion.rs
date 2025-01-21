use std::cell::RefCell;
use std::rc::{Rc, Weak};

thread_local! {
    static ACTIVE_CONTEXT: RefCell<Option<Rc<RefCell<VelloContext>>>> = RefCell::new(None);
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

#[wasm_bindgen]
pub struct ShapeHandle {
    id: usize,
    #[wasm_bindgen(skip)]
    context: Weak<RefCell<VelloContext>>,
}

#[wasm_bindgen]
impl VelloContext {
    #[wasm_bindgen]
    pub async fn create(canvas_id: &str) -> Result<VelloContext, JsValue> {
        // ... existing setup code ...

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
            Color::rgba8(r, g, b, a),
        )));

        self.render();

        ShapeHandle {
            id,
            context: Rc::downgrade(&ACTIVE_CONTEXT.with(|ctx| ctx.borrow().clone().unwrap())),
        }
    }
}

#[wasm_bindgen]
impl ShapeHandle {
    pub fn follow(&self, other: &ShapeHandle) -> Result<(), JsValue> {
        if let Some(context) = self.context.upgrade() {
            let mut context = context.borrow_mut();
            if let (Some(shape1), Some(shape2)) = (
                context.shapes.get_mut(self.id),
                context.shapes.get(other.id),
            ) {
                shape1.node().start_following(shape2.node(), other.id);
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
