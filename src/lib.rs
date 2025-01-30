mod color;
mod hooks;
mod image;
mod math;
mod renderer;
mod world;

use crate::{
    hooks::{use_element, use_element_size},
    renderer::WorldRenderer,
    world::{Params, World},
};
use anyhow::Result;
use dioxus::prelude::*;
use log::info;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    info!("wasm start");

    let window = web_sys::window()
        .ok_or_else(|| JsError::new("failed to get window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsError::new("failed to get document of window"))?;
    let body = document
        .body()
        .ok_or_else(|| JsError::new("failed to get body of document"))?;

    dioxus::web::launch::launch_cfg(
        App,
        dioxus::web::Config::new().rootelement(body.into()),
    );

    Ok(())
}

#[component]
fn App() -> Element {
    let world = use_signal(World::new);
    let mut world_renderer = use_signal(|| None::<WorldRenderer>);

    let (canvas_element, on_canvas_mounted) =
        use_element::<web_sys::HtmlCanvasElement>();

    let canvas_size = use_element_size(canvas_element.read().clone());

    use_effect(move || {
        let canvas_element = canvas_element.read();
        let canvas_element = &*canvas_element;
        let canvas_size = canvas_size.read();
        let canvas_size = *canvas_size;
        if let Some(canvas_element) = canvas_element {
            if let Some(canvas_size) = canvas_size {
                canvas_element.set_width(canvas_size.width as u32);
                canvas_element.set_height(canvas_size.height as u32);
            }
            world_renderer.with_mut(|renderer| {
                if let Some(renderer) = renderer {
                    renderer.update(canvas_element);
                } else {
                    *renderer = Some(WorldRenderer::new(canvas_element, world));
                }
            });
        }
    });

    let Params {
        particle_count,
        seed,
    } = *world.read().params();

    rsx! {
        canvas {
            onmounted: on_canvas_mounted,
        }
        div {
            class: "ui",
            div {
                class: "param",
                "seed: "
                span {
                    class: "param-value seed",
                    "0x{seed:016x}"
                }
            }
            div {
                class: "param",
                "particles: "
                span {
                    class: "param-value",
                    "{particle_count}"
                }
            }
        }
    }
}
