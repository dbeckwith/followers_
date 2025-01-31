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
use rand::prelude::*;
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
    let mut seed_rng = use_signal(thread_rng);
    let mut params =
        use_signal(|| Params::new(1000, 0x27e3771584a46455).unwrap());
    let mut world = use_signal(|| World::new(*params.peek()));
    let mut world_renderer = use_signal(|| None::<WorldRenderer>);

    let (canvas_element, on_canvas_mounted) =
        use_element::<web_sys::HtmlCanvasElement>();
    let canvas_size = use_element_size(canvas_element.read().clone());

    let on_click_new_seed = use_callback(move |_: Event<MouseData>| {
        let seed = seed_rng.write().gen();
        params.write().seed = seed;
    });

    let on_click_pause_resume = use_callback(move |_: Event<MouseData>| {
        if let Some(world_renderer) = &mut *world_renderer.write() {
            world_renderer.pause_resume();
        }
    });

    let on_click_reset = use_callback(move |_: Event<MouseData>| {
        params.write();
    });

    let on_click_save = use_callback(move |_: Event<MouseData>| {
        let canvas_element = &*canvas_element.read();
        let canvas_element = if let Some(canvas_element) = canvas_element {
            canvas_element
        } else {
            return;
        };
        let params = *params.read();
        let document = canvas_element.owner_document().unwrap();
        let closure = Closure::<dyn FnMut(Option<web_sys::Blob>)>::new(
            move |blob: Option<web_sys::Blob>| {
                let blob = if let Some(blob) = blob {
                    blob
                } else {
                    return;
                };
                let anchor = document.create_element("a").unwrap();
                let anchor =
                    anchor.dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
                let Params {
                    particle_count,
                    seed,
                } = params;
                anchor.set_download(&format!(
                    "{particle_count}-0x{seed:016x}.png"
                ));
                let url =
                    web_sys::Url::create_object_url_with_blob(&blob).unwrap();
                anchor.set_href(&url);
                let body = document.body().unwrap();
                body.append_child(&anchor).unwrap();
                anchor.click();
                body.remove_child(&anchor).unwrap();
                web_sys::Url::revoke_object_url(&url).unwrap();
            },
        );
        canvas_element
            .to_blob(closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget(); // FIXME: don't leak
    });

    use_effect(move || {
        world.set(World::new(*params.read()));
        if let Some(world_renderer) = &mut *world_renderer.write() {
            world_renderer.clear();
        }
    });

    use_effect(move || {
        let canvas_element = &*canvas_element.read();
        let canvas_element = if let Some(canvas_element) = canvas_element {
            canvas_element
        } else {
            return;
        };
        let canvas_size = *canvas_size.read();
        let canvas_size = if let Some(canvas_size) = canvas_size {
            canvas_size
        } else {
            return;
        };
        canvas_element.set_width(canvas_size.width as u32);
        canvas_element.set_height(canvas_size.height as u32);
        world_renderer.with_mut(|renderer| {
            if let Some(renderer) = renderer {
                renderer.update(canvas_element);
            } else {
                *renderer = Some(WorldRenderer::new(canvas_element, world));
            }
        });
    });

    let Params {
        particle_count,
        seed,
    } = *world.read().params();

    let paused = world_renderer
        .read()
        .as_ref()
        .is_some_and(|world_renderer| world_renderer.paused());

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
            div {
                class: "control",
                button {
                    onclick: on_click_new_seed,
                    "new seed"
                }
            }
            div {
                class: "control",
                button {
                    onclick: on_click_pause_resume,
                    if paused { "resume" } else { "pause" }
                }
            }
            div {
                class: "control",
                button {
                    onclick: on_click_reset,
                    "reset"
                }
            }
            div {
                class: "control",
                button {
                    onclick: on_click_save,
                    "save"
                }
            }
        }
    }
}
