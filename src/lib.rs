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
use log::{info, warn};
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
    let mut params = use_signal(|| Params {
        seed: 0x27e3771584a46455,
        particle_count: 1000,
        particle_alpha: 6.0,
        acc_limit: -1.0,
    });
    let mut world = use_signal(|| World::new(*params.peek()).unwrap());
    let mut world_renderer = use_signal(|| None::<WorldRenderer>);

    let (canvas_element, on_canvas_mounted) =
        use_element::<web_sys::HtmlCanvasElement>();
    let canvas_size = use_element_size(canvas_element.read().clone());

    let on_click_rand_seed = use_callback(move |_: Event<MouseData>| {
        let seed = seed_rng.write().gen();
        params.write().seed = seed;
    });

    let on_input_particle_count =
        use_callback(move |event: Event<FormData>| {
            let particle_count = if let Ok(particle_count) = event.parsed() {
                particle_count
            } else {
                return;
            };
            params.write().particle_count = particle_count;
        });

    let on_input_particle_alpha =
        use_callback(move |event: Event<FormData>| {
            let particle_alpha = if let Ok(particle_alpha) = event.parsed() {
                particle_alpha
            } else {
                return;
            };
            params.write().particle_alpha = particle_alpha;
        });

    let on_input_acc_limit = use_callback(move |event: Event<FormData>| {
        let acc_limit = if let Ok(acc_limit) = event.parsed() {
            acc_limit
        } else {
            return;
        };
        params.write().acc_limit = acc_limit;
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
                    seed,
                    particle_count,
                    particle_alpha: _,
                    acc_limit: _,
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
        let params = *params.read();
        let new_world = if let Ok(world) = World::new(params) {
            world
        } else {
            warn!("bad params: {:?}", params);
            return;
        };
        world.set(new_world);
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
        seed,
        particle_count,
        particle_alpha,
        acc_limit,
    } = *world.read().params();

    let acc_limit_display = (2.0f32.powf(acc_limit) * 1000.0).round() / 1000.0;

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
                class: "param seed",
                div {
                    class: "param-label",
                    "seed: "
                }
                div {
                    class: "param-value",
                    "0x{seed:016x}"
                }
                div {
                    class: "param-control",
                    button {
                        onclick: on_click_rand_seed,
                        "rand"
                    }
                }
            }
            div {
                class: "param count",
                div {
                    class: "param-label",
                    "particles: "
                }
                div {
                    class: "param-control",
                    input {
                        r#type: "number",
                        min: 2,
                        max: 1000000,
                        value: particle_count,
                        oninput: on_input_particle_count,
                    }
                }
            }
            div {
                class: "param alpha",
                div {
                    class: "param-label",
                    "alpha: "
                }
                div {
                    class: "param-control",
                    input {
                        r#type: "number",
                        min: 1,
                        max: 100,
                        value: particle_alpha,
                        oninput: on_input_particle_alpha,
                    }
                }
            }
            div {
                class: "param acc-limit",
                div {
                    class: "param-label",
                    "acc limit: "
                }
                div {
                    class: "param-value",
                    "{acc_limit_display}"
                }
                div {
                    class: "param-control",
                    input {
                        r#type: "range",
                        min: -10,
                        max: 10,
                        value: acc_limit,
                        oninput: on_input_acc_limit,
                    }
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
