#![allow(unused_imports)]

use log::debug;
use std::f64;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, DomError, HtmlCanvasElement};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen(start)]
pub fn start() {
    set_panic_hook();

    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug).message_on_new_line());

    log::debug!("hello, world, from logging");
}

fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// https://github.com/rustwasm/wasm-bindgen/tree/master/examples/canvas

macro_rules! unwrap_option_or_throw {
    ($e:expr, $msg:expr) => {
        if let Some(value) = $e {
            value
        } else {
            DomError::new_with_message("unwrap_option_or_throw", $msg)?;
            panic!();
        }
    };
}

#[wasm_bindgen]
pub fn hello_world() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();

    let canvas_node = unwrap_option_or_throw!(
        document.get_element_by_id("trellis_canvas"),
        "Missing canvas"
    );
    let canvas: HtmlCanvasElement = canvas_node.dyn_into::<HtmlCanvasElement>()?;

    let context = unwrap_option_or_throw!(
        canvas.get_context("2d")?,
        "get_context() should return a non-null value"
    )
    .dyn_into::<CanvasRenderingContext2d>()?;

    context.begin_path();

    // Draw the outer circle.
    context.arc(75.0, 75.0, 50.0, 0.0, f64::consts::PI * 2.0)?;

    // Draw the mouth.
    context.move_to(110.0, 75.0);
    context.arc(75.0, 75.0, 35.0, 0.0, f64::consts::PI)?;

    // Draw the left eye.
    context.move_to(65.0, 65.0);
    context.arc(60.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)?;

    // Draw the right eye.
    context.move_to(95.0, 65.0);
    context.arc(90.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)?;

    context.stroke();

    Ok(())
}

use trellis::testing::graph_from_paths;

#[derive(Clone, Debug)]
struct GraphDrawingParams {
    pub layer_y0: f64,
    pub layer_dy: f64,

    pub column_x0: f64,
    pub column_dx: f64,

    pub vert_width: f64,
    pub vert_height: f64,
}

impl GraphDrawingParams {
    pub fn new() -> Self {
        Self {
            layer_y0: 400.0,
            layer_dy: -50.0,
            column_x0: 20.0,
            column_dx: 50.0,
            vert_width: 25.0,
            vert_height: 25.0,
        }
    }

    pub fn layer_y(&self, layer: u32) -> f64 {
        self.layer_y0 + (layer as f64) * self.layer_dy
    }

    pub fn column_x(&self, column: u32) -> f64 {
        self.column_x0 + (column as f64) * self.column_dx
    }
}

use trellis::graph::Graph;
use trellis::layering::{create_proper_graph, ProperGraph};
use trellis::ramp_table::RampTable;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct CanvasApp {
    graph: Graph,
    proper_graph: ProperGraph,
    params: GraphDrawingParams,
}

#[wasm_bindgen]
pub fn make_app() -> CanvasApp {
    let graph = graph_from_paths(&[
        &[1, 10, 11, 12, 13, 14],
        &[1, 20, 21, 22, 23, 24],
        &[1, 14],
        &[1, 24],
        &[10, 23],
    ]);

    let proper_graph = create_proper_graph(&graph).unwrap();

    let app = CanvasApp {
        graph,
        proper_graph,
        params: GraphDrawingParams::new(),
    };

    app
}

#[wasm_bindgen]
pub fn paint_graph(app: &CanvasApp, context: JsValue) -> Result<(), JsValue> {
    // debug!("paint_graph: {:?}", app);

    let c = context.dyn_into::<CanvasRenderingContext2d>().unwrap();

    let params = &app.params;
    let proper_graph = &app.proper_graph;

    let vw = params.vert_width;
    let vh = params.vert_height;

    let v_pos = &proper_graph.v_pos;

    // paint edges
    c.set_stroke_style(&JsValue::from_str("black"));
    for (to_layer, edges) in proper_graph.edges.iter().enumerate() {
        let from_layer = to_layer + 1;
        for &(from_v, to_v) in edges.iter() {
            let from_vx = params.column_x(v_pos[from_v as usize]) + vw / 2.0;
            let to_vx = params.column_x(v_pos[to_v as usize]) + vh / 2.0;

            let from_vy = params.layer_y(from_layer as u32) + vw / 2.0;
            let to_vy = params.layer_y(to_layer as u32) + vh / 2.0;

            c.begin_path();
            c.move_to(from_vx, from_vy);
            c.line_to(to_vx, to_vy);
            c.stroke();
        }
    }

    // paint verts
    for (layer, verts) in proper_graph.verts.iter().enumerate() {
        // debug!("painting L{}", layer);

        for &v in verts.iter() {
            let v_x = params.column_x(proper_graph.v_pos[v as usize]);
            let v_y = params.layer_y(layer as u32);
            let lx = v_x; // left x
            let rx = v_x + vw; // right x
            let ty = v_y; // top y
            let by = v_y + vh; // bottom y
            c.begin_path();

            let notch_dx = 5.0;
            let notch_dy = 5.0;

            c.move_to(lx + notch_dx, ty); // top left point (right of notch)
            c.line_to(rx, ty); // top edge
            c.line_to(rx, by); // right edge
            c.line_to(lx, by); // bottom edge
            c.line_to(lx, ty + notch_dy);

            c.close_path();

            c.set_fill_style(&JsValue::from_str("LightBlue"));
            c.fill();

            c.set_stroke_style(&JsValue::from_str("black"));
            c.stroke();

            let label = format!("v{}", v);
            c.set_fill_style(&JsValue::from_str("black"));
            c.fill_text(&label, lx + 4.0, by - 4.0)?;
        }
    }

    Ok(())
}
