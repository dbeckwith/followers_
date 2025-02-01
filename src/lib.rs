mod color;
mod hooks;
mod image;
mod math;
mod renderer;
mod world;

use crate::{
    color::Color,
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

const MIN_PARTICLE_COUNT: usize = 3;
const MAX_PARTICLE_COUNT: usize = 1000000;
const MIN_PARTICLE_COLOR_HUE_MID: f32 = 0.0;
const MAX_PARTICLE_COLOR_HUE_MID: f32 = 360.0;
const MIN_PARTICLE_COLOR_HUE_SPREAD: f32 = 0.0;
const MAX_PARTICLE_COLOR_HUE_SPREAD: f32 = 360.0;
const MIN_PARTICLE_COLOR_SATURATION_MID: f32 = 0.0;
const MAX_PARTICLE_COLOR_SATURATION_MID: f32 = 100.0;
const MIN_PARTICLE_COLOR_SATURATION_SPREAD: f32 = 0.0;
const MAX_PARTICLE_COLOR_SATURATION_SPREAD: f32 = 100.0;
const MIN_PARTICLE_COLOR_VALUE: f32 = 1.0;
const MAX_PARTICLE_COLOR_VALUE: f32 = 100.0;
const MIN_PARTICLE_COLOR_ALPHA: f32 = 1.0;
const MAX_PARTICLE_COLOR_ALPHA: f32 = 100.0;
const MIN_ACC_LIMIT: f32 = -10.0;
const MAX_ACC_LIMIT: f32 = 10.0;

#[component]
fn App() -> Element {
    let mut seed_rng = use_signal(thread_rng);
    let mut params = use_signal(|| Params {
        seed: 0x27e3771584a46455,
        particle_count: 1000,
        particle_color_hue_mid: 120.0,
        particle_color_hue_spread: 240.0,
        particle_color_saturation_mid: 70.0,
        particle_color_saturation_spread: 20.0,
        particle_color_value: 100.0,
        particle_color_alpha: 6.0,
        acc_limit: -1.0,
    });
    let background_color = use_signal(|| Color::hex(0x000000ff));
    let mut frame_limit = use_signal(|| 60 * 60);
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
            let particle_count =
                if let Ok(particle_count) = event.parsed::<usize>() {
                    particle_count
                } else {
                    return;
                };
            params.write().particle_count =
                particle_count.clamp(MIN_PARTICLE_COUNT, MAX_PARTICLE_COUNT);
        });

    let on_input_particle_color_hue_mid =
        use_callback(move |event: Event<FormData>| {
            let particle_hue_mid =
                if let Ok(particle_hue_mid) = event.parsed::<f32>() {
                    particle_hue_mid
                } else {
                    return;
                };
            params.write().particle_color_hue_mid = particle_hue_mid
                .clamp(MIN_PARTICLE_COLOR_HUE_MID, MAX_PARTICLE_COLOR_HUE_MID);
        });

    let on_input_particle_color_hue_spread =
        use_callback(move |event: Event<FormData>| {
            let particle_hue_spread =
                if let Ok(particle_hue_spread) = event.parsed::<f32>() {
                    particle_hue_spread
                } else {
                    return;
                };
            params.write().particle_color_hue_spread = particle_hue_spread
                .clamp(
                    MIN_PARTICLE_COLOR_HUE_SPREAD,
                    MAX_PARTICLE_COLOR_HUE_SPREAD,
                );
        });

    let on_input_particle_color_saturation_mid =
        use_callback(move |event: Event<FormData>| {
            let particle_saturation_mid =
                if let Ok(particle_saturation_mid) = event.parsed::<f32>() {
                    particle_saturation_mid
                } else {
                    return;
                };
            params.write().particle_color_saturation_mid =
                particle_saturation_mid.clamp(
                    MIN_PARTICLE_COLOR_SATURATION_MID,
                    MAX_PARTICLE_COLOR_SATURATION_MID,
                );
        });

    let on_input_particle_color_saturation_spread =
        use_callback(move |event: Event<FormData>| {
            let particle_saturation_spread =
                if let Ok(particle_saturation_spread) = event.parsed::<f32>() {
                    particle_saturation_spread
                } else {
                    return;
                };
            params.write().particle_color_saturation_spread =
                particle_saturation_spread.clamp(
                    MIN_PARTICLE_COLOR_SATURATION_SPREAD,
                    MAX_PARTICLE_COLOR_SATURATION_SPREAD,
                );
        });

    let on_input_particle_color_value =
        use_callback(move |event: Event<FormData>| {
            let particle_value =
                if let Ok(particle_value) = event.parsed::<f32>() {
                    particle_value
                } else {
                    return;
                };
            params.write().particle_color_value = particle_value
                .clamp(MIN_PARTICLE_COLOR_VALUE, MAX_PARTICLE_COLOR_VALUE);
        });

    let on_input_particle_color_alpha =
        use_callback(move |event: Event<FormData>| {
            let particle_alpha =
                if let Ok(particle_alpha) = event.parsed::<f32>() {
                    particle_alpha
                } else {
                    return;
                };
            params.write().particle_color_alpha = particle_alpha
                .clamp(MIN_PARTICLE_COLOR_ALPHA, MAX_PARTICLE_COLOR_ALPHA);
        });

    let on_input_acc_limit = use_callback(move |event: Event<FormData>| {
        let acc_limit = if let Ok(acc_limit) = event.parsed::<f32>() {
            acc_limit
        } else {
            return;
        };
        params.write().acc_limit =
            acc_limit.clamp(MIN_ACC_LIMIT, MAX_ACC_LIMIT);
    });

    let on_input_frame_limit = use_callback(move |event: Event<FormData>| {
        let frame_limit_ = if let Ok(frame_limit) = event.parsed::<usize>() {
            frame_limit
        } else {
            return;
        };
        frame_limit.set(frame_limit_.max(1));
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
                    particle_color_alpha: _,
                    particle_color_hue_mid: _,
                    particle_color_hue_spread: _,
                    particle_color_saturation_mid: _,
                    particle_color_saturation_spread: _,
                    particle_color_value: _,
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
            world_renderer.resume();
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
                *renderer = Some(WorldRenderer::new(
                    canvas_element,
                    world,
                    background_color,
                    frame_limit,
                ));
            }
        });
    });

    // re-render when world updates
    world.read();

    let Params {
        seed,
        particle_count,
        particle_color_hue_mid,
        particle_color_hue_spread,
        particle_color_saturation_mid,
        particle_color_saturation_spread,
        particle_color_value,
        particle_color_alpha,
        acc_limit,
    } = *params.read();

    let acc_limit_display = (acc_limit.exp2() * 1000.0).round() / 1000.0;

    let world_renderer = world_renderer.read();
    let world_renderer = world_renderer.as_ref();
    let paused =
        world_renderer.is_some_and(|world_renderer| world_renderer.paused());
    let frame_idx = world_renderer
        .map(|world_renderer| world_renderer.frame_idx())
        .unwrap_or(0);

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
                class: "param particle-count",
                div {
                    class: "param-label",
                    "particles: "
                }
                div {
                    class: "param-control",
                    input {
                        r#type: "number",
                        min: MIN_PARTICLE_COUNT,
                        max: MAX_PARTICLE_COUNT,
                        value: particle_count,
                        oninput: on_input_particle_count,
                    }
                }
            }
            div {
                class: "param particle-color-hue-mid",
                div {
                    class: "param-label",
                    "hue mid: "
                }
                div {
                    class: "param-control",
                    input {
                        r#type: "number",
                        min: MIN_PARTICLE_COLOR_HUE_MID,
                        max: MAX_PARTICLE_COLOR_HUE_MID,
                        value: particle_color_hue_mid,
                        oninput: on_input_particle_color_hue_mid,
                    }
                }
            }
            div {
                class: "param particle-color-hue-spread",
                div {
                    class: "param-label",
                    "hue spread: "
                }
                div {
                    class: "param-control",
                    input {
                        r#type: "number",
                        min: MIN_PARTICLE_COLOR_HUE_SPREAD,
                        max: MAX_PARTICLE_COLOR_HUE_SPREAD,
                        value: particle_color_hue_spread,
                        oninput: on_input_particle_color_hue_spread,
                    }
                }
            }
            div {
                class: "param particle-color-saturation-mid",
                div {
                    class: "param-label",
                    "saturation mid: "
                }
                div {
                    class: "param-control",
                    input {
                        r#type: "number",
                        min: MIN_PARTICLE_COLOR_SATURATION_MID,
                        max: MAX_PARTICLE_COLOR_SATURATION_MID,
                        value: particle_color_saturation_mid,
                        oninput: on_input_particle_color_saturation_mid,
                    }
                }
            }
            div {
                class: "param particle-color-saturation-spread",
                div {
                    class: "param-label",
                    "saturation spread: "
                }
                div {
                    class: "param-control",
                    input {
                        r#type: "number",
                        min: MIN_PARTICLE_COLOR_SATURATION_SPREAD,
                        max: MAX_PARTICLE_COLOR_SATURATION_SPREAD,
                        value: particle_color_saturation_spread,
                        oninput: on_input_particle_color_saturation_spread,
                    }
                }
            }
            div {
                class: "param particle-color-value",
                div {
                    class: "param-label",
                    "brightness: "
                }
                div {
                    class: "param-control",
                    input {
                        r#type: "number",
                        min: MIN_PARTICLE_COLOR_VALUE,
                        max: MAX_PARTICLE_COLOR_VALUE,
                        value: particle_color_value,
                        oninput: on_input_particle_color_value,
                    }
                }
            }
            div {
                class: "param particle-color-alpha",
                div {
                    class: "param-label",
                    "opacity: "
                }
                div {
                    class: "param-control",
                    input {
                        r#type: "number",
                        min: MIN_PARTICLE_COLOR_ALPHA,
                        max: MAX_PARTICLE_COLOR_ALPHA,
                        value: particle_color_alpha,
                        oninput: on_input_particle_color_alpha,
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
                        min: MIN_ACC_LIMIT,
                        max: MAX_ACC_LIMIT,
                        value: acc_limit,
                        oninput: on_input_acc_limit,
                    }
                }
            }
            div {
                class: "param frame-limit",
                div {
                    class: "param-label",
                    "frame limit: "
                }
                div {
                    class: "param-control",
                    input {
                        r#type: "number",
                        min: 1,
                        value: frame_limit,
                        oninput: on_input_frame_limit,
                    }
                }
            }
            div {
                class: "param frame",
                div {
                    class: "param-label",
                    "frame: "
                }
                div {
                    class: "param-value",
                    "{frame_idx}"
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
