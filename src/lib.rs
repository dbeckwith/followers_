mod color;
mod hooks;
mod image;
mod math;
mod renderer;
mod world;

use crate::{
    color::Color,
    hooks::{use_element, use_element_size},
    image::Image,
    math::lerp,
    renderer::WorldRenderer,
    world::{DisplayParams, Seed, SimParams, World},
};
use anyhow::Result;
use base64::prelude::*;
use dioxus::{
    logger::tracing::{debug, info, warn},
    prelude::*,
};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    dioxus::logger::init(dioxus::logger::tracing::Level::DEBUG)
        .expect("logger failed to init");
    info!("wasm start");

    let window = web_sys::window()
        .ok_or_else(|| JsError::new("failed to get window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsError::new("failed to get document of window"))?;
    let body = document
        .body()
        .ok_or_else(|| JsError::new("failed to get body of document"))?;

    dioxus::LaunchBuilder::web()
        .with_cfg(dioxus::web::Config::new().rootelement(body.into()))
        .launch(App);

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
const MIN_ACC_LIMIT: i32 = -10;
const MAX_ACC_LIMIT: i32 = 10;

const PALETTE_WIDTH: usize = 100;
const PALETTE_HEIGHT: usize = 40;

const BACKGROUND_COLOR: Color = Color::hex(0x000000ff);

const CONFIG_COMMIT_DELAY_MS: u32 = 400;
const CONFIG_QUERY_PARAM: &str = "c";

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    version: u64,
    sim_params: SimParams,
    display_params: DisplayParams,
    frame_limit: usize,
}

#[component]
fn App() -> Element {
    let mut seed_rng = use_signal(thread_rng);
    let mut sim_params = use_signal(|| SimParams {
        seed: Seed::from_hash(0x27e3771584a46455),
        particle_count: 1000,
        acc_limit: -1,
    });
    let mut display_params = use_signal(|| DisplayParams {
        particle_color_hue_mid: 120.0,
        particle_color_hue_spread: 240.0,
        particle_color_saturation_mid: 70.0,
        particle_color_saturation_spread: 20.0,
        particle_color_value: 100.0,
        particle_color_alpha: 6.0,
    });
    let mut frame_limit = use_signal(|| 60 * 60);
    let mut world = use_signal(|| {
        World::new(&sim_params.peek(), &display_params.peek()).unwrap()
    });
    let mut world_renderer = use_signal(|| None::<WorldRenderer>);
    let mut palette_image = use_signal(|| {
        Image::new(PALETTE_WIDTH, PALETTE_HEIGHT, Color::transparent())
    });

    let (world_canvas_element, on_world_canvas_mounted) =
        use_element::<web_sys::HtmlCanvasElement>();
    let world_canvas_size =
        use_element_size(world_canvas_element.read().clone());

    let (palette_canvas_element, on_palette_canvas_mounted) =
        use_element::<web_sys::HtmlCanvasElement>();

    let on_input_seed = use_callback(move |event: Event<FormData>| {
        let seed = event.value();
        sim_params.write().seed = Seed::from_str(seed);
    });

    let on_click_rand_seed = use_callback(move |_: Event<MouseData>| {
        let seed = seed_rng.write().gen::<u64>();
        sim_params.write().seed = Seed::from_hash(seed);
    });

    let on_input_particle_count =
        use_callback(move |event: Event<FormData>| {
            let particle_count =
                if let Ok(particle_count) = event.parsed::<usize>() {
                    particle_count
                } else {
                    return;
                };
            sim_params.write().particle_count =
                particle_count.clamp(MIN_PARTICLE_COUNT, MAX_PARTICLE_COUNT);
        });

    let on_input_acc_limit = use_callback(move |event: Event<FormData>| {
        let acc_limit = if let Ok(acc_limit) = event.parsed::<i32>() {
            acc_limit
        } else {
            return;
        };
        sim_params.write().acc_limit =
            acc_limit.clamp(MIN_ACC_LIMIT, MAX_ACC_LIMIT);
    });

    let on_input_particle_color_hue_mid =
        use_callback(move |event: Event<FormData>| {
            let particle_hue_mid =
                if let Ok(particle_hue_mid) = event.parsed::<f32>() {
                    particle_hue_mid
                } else {
                    return;
                };
            display_params.write().particle_color_hue_mid = particle_hue_mid
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
            display_params.write().particle_color_hue_spread =
                particle_hue_spread.clamp(
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
            display_params.write().particle_color_saturation_mid =
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
            display_params.write().particle_color_saturation_spread =
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
            display_params.write().particle_color_value = particle_value
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
            display_params.write().particle_color_alpha = particle_alpha
                .clamp(MIN_PARTICLE_COLOR_ALPHA, MAX_PARTICLE_COLOR_ALPHA);
        });

    let on_input_frame_limit = use_callback(move |event: Event<FormData>| {
        let frame_limit_ = if let Ok(frame_limit) = event.parsed::<usize>() {
            frame_limit
        } else {
            return;
        };
        frame_limit.set(frame_limit_.max(1));
        if let Some(world_renderer) = &mut *world_renderer.write() {
            world_renderer.resume();
        }
    });

    let on_click_pause_resume = use_callback(move |_: Event<MouseData>| {
        if let Some(world_renderer) = &mut *world_renderer.write() {
            world_renderer.pause_resume();
        }
    });

    let on_click_reset = use_callback(move |_: Event<MouseData>| {
        sim_params.write();
    });

    let on_click_save = use_callback(move |_: Event<MouseData>| {
        let world_canvas_element = &*world_canvas_element.read();
        let world_canvas_element =
            if let Some(world_canvas_element) = world_canvas_element {
                world_canvas_element
            } else {
                return;
            };
        let file_name = sim_params.read().file_name("png");
        let document = world_canvas_element.owner_document().unwrap();
        let closure = Closure::<dyn FnMut(Option<web_sys::Blob>)>::new(
            move |blob: Option<web_sys::Blob>| {
                let blob = if let Some(blob) = blob {
                    blob
                } else {
                    return;
                };
                download_blob(&document, &blob, &file_name);
            },
        );
        world_canvas_element
            .to_blob(closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget(); // FIXME: don't leak
    });

    let on_click_save_svg = use_callback(move |_: Event<MouseData>| {
        let file_name = sim_params.read().file_name("svg");
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        defer(&window, move || {
            let svg = world.peek().generate_svg(BACKGROUND_COLOR);
            // TODO: handle errors?
            let blob = web_sys::Blob::new_with_str_sequence(&vec![svg].into())
                .unwrap();
            download_blob(&document, &blob, &file_name);
        });
    });

    use_effect(move || {
        let new_world =
            match World::new(&sim_params.read(), &display_params.read()) {
                Ok(world) => world,
                Err(error) => {
                    warn!("failed to create world: {:?}", error);
                    return;
                },
            };
        world.set(new_world);
        if let Some(world_renderer) = &mut *world_renderer.write() {
            world_renderer.clear();
            world_renderer.resume();
        }
    });

    use_effect(move || {
        let world_canvas_element = &*world_canvas_element.read();
        let world_canvas_element =
            if let Some(world_canvas_element) = world_canvas_element {
                world_canvas_element
            } else {
                return;
            };
        let world_canvas_size = *world_canvas_size.read();
        let world_canvas_size =
            if let Some(world_canvas_size) = world_canvas_size {
                world_canvas_size
            } else {
                return;
            };
        world_canvas_element.set_width(world_canvas_size.width as u32);
        world_canvas_element.set_height(world_canvas_size.height as u32);
        world_renderer.with_mut(|renderer| {
            if let Some(renderer) = renderer {
                renderer.update(world_canvas_element);
            } else {
                *renderer = Some(WorldRenderer::new(
                    world_canvas_element,
                    world,
                    BACKGROUND_COLOR,
                    frame_limit,
                ));
            }
        });
    });

    use_effect(move || {
        let DisplayParams {
            particle_color_hue_mid,
            particle_color_hue_spread,
            particle_color_saturation_mid,
            particle_color_saturation_spread,
            particle_color_value,
            particle_color_alpha: _,
        } = &*display_params.read();
        let palette_image = &mut *palette_image.write();
        for y in 0..PALETTE_HEIGHT {
            for x in 0..PALETTE_WIDTH {
                let color = Color::hsva(
                    lerp(
                        x as f32,
                        0.0,
                        (PALETTE_WIDTH - 1) as f32,
                        particle_color_hue_mid
                            - particle_color_hue_spread / 2.0,
                        particle_color_hue_mid
                            + particle_color_hue_spread / 2.0,
                    ),
                    lerp(
                        y as f32,
                        (PALETTE_HEIGHT - 1) as f32,
                        0.0,
                        particle_color_saturation_mid
                            - particle_color_saturation_spread / 2.0,
                        particle_color_saturation_mid
                            + particle_color_saturation_spread / 2.0,
                    ),
                    *particle_color_value,
                    100.0,
                );
                palette_image.put_pixel(x, y, color);
            }
        }
    });

    use_effect(move || {
        let palette_canvas_element = &*palette_canvas_element.read();
        if let Some(palette_canvas_element) = palette_canvas_element {
            let context = palette_canvas_element
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();
            let image_data = palette_image.read().to_image_data();
            context.put_image_data(&image_data, 0.0, 0.0).unwrap();
        }
    });

    // read config from URL
    use_effect(move || {
        let mut read_config = move || {
            let window = web_sys::window().unwrap();
            let url =
                web_sys::Url::new(&window.location().href().unwrap()).unwrap();
            let config = if let Some(config) = url
                .search_params()
                .get(CONFIG_QUERY_PARAM)
                .as_deref()
                .and_then(decode_config_str)
            {
                config
            } else {
                return;
            };
            debug!("config from URL: {:#?}", config);
            let Config {
                version,
                sim_params: sim_params_,
                display_params: display_params_,
                frame_limit: frame_limit_,
            } = config;
            if version != 1 {
                return;
            }
            sim_params.set(sim_params_);
            display_params.set(display_params_);
            frame_limit.set(frame_limit_);
        };
        read_config();
        let window = web_sys::window().unwrap();
        let on_pop_state =
            Closure::<dyn FnMut(web_sys::PopStateEvent)>::new(move |_event| {
                read_config();
            });
        window
            .add_event_listener_with_callback(
                "popstate",
                on_pop_state.as_ref().unchecked_ref(),
            )
            .unwrap();
        on_pop_state.forget(); // FIXME: don't leak
    });

    // write config to URL
    let mut history_push_state_timeout_handle = use_signal(|| None::<i32>);
    use_effect(move || {
        let window = web_sys::window().unwrap();
        let url =
            web_sys::Url::new(&window.location().href().unwrap()).unwrap();
        let config_str = encode_config_str(Config {
            version: 1,
            sim_params: sim_params.read().clone(),
            display_params: display_params.read().clone(),
            frame_limit: *frame_limit.read(),
        });
        let config_str = config_str.as_deref().unwrap_or("");
        if url.search_params().get(CONFIG_QUERY_PARAM).as_deref()
            == Some(config_str)
        {
            return;
        }
        url.search_params().set(CONFIG_QUERY_PARAM, config_str);
        let url = url.href();
        let history = window.history().unwrap();
        if let Some(history_push_state_timeout_handle) =
            *history_push_state_timeout_handle.peek()
        {
            window.clear_timeout_with_handle(history_push_state_timeout_handle);
        }
        let callback = Closure::<dyn FnOnce()>::once_into_js(move || {
            history
                .push_state_with_url(&js_sys::Object::new(), "", Some(&url))
                .unwrap();
        });
        history_push_state_timeout_handle.set(Some(
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.unchecked_ref(),
                    CONFIG_COMMIT_DELAY_MS as i32,
                )
                .unwrap(),
        ));
    });

    // re-render when world updates
    world.read();

    let SimParams {
        seed,
        particle_count,
        acc_limit,
    } = &*sim_params.read();
    let DisplayParams {
        particle_color_hue_mid,
        particle_color_hue_spread,
        particle_color_saturation_mid,
        particle_color_saturation_spread,
        particle_color_value,
        particle_color_alpha,
    } = &*display_params.read();

    let world_renderer = world_renderer.read();
    let world_renderer = world_renderer.as_ref();
    let paused =
        world_renderer.is_some_and(|world_renderer| world_renderer.paused());
    let frame_idx = world_renderer
        .map(|world_renderer| world_renderer.frame_idx())
        .unwrap_or(0);

    rsx! {
        canvas {
            class: "world",
            onmounted: on_world_canvas_mounted,
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
                    class: "param-control",
                    input {
                        r#type: "text",
                        value: seed.as_str(),
                        oninput: on_input_seed,
                    }
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
                        value: *particle_count,
                        oninput: on_input_particle_count,
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
                    class: "param-control",
                    "2^"
                    input {
                        r#type: "number",
                        min: MIN_ACC_LIMIT,
                        max: MAX_ACC_LIMIT,
                        value: *acc_limit,
                        oninput: on_input_acc_limit,
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
                        value: *particle_color_hue_mid,
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
                        value: *particle_color_hue_spread,
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
                        value: *particle_color_saturation_mid,
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
                        value: *particle_color_saturation_spread,
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
                        value: *particle_color_value,
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
                        value: *particle_color_alpha,
                        oninput: on_input_particle_color_alpha,
                    }
                }
            }
            div {
                class: "param particle-color-palette",
                div {
                    class: "param-label",
                    "colors: "
                }
                div {
                    class: "param-value",
                    canvas {
                        width: PALETTE_WIDTH,
                        height: PALETTE_HEIGHT,
                        onmounted: on_palette_canvas_mounted,
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
                    "save png"
                }
            }
            div {
                class: "control",
                button {
                    onclick: on_click_save_svg,
                    "save svg"
                }
            }
        }
    }
}

fn encode_config_str(config: Config) -> Option<String> {
    let message_pack = rmp_serde::to_vec(&config).ok()?;
    let deflated_message_pack = deflate::deflate_bytes_conf(
        message_pack.as_slice(),
        deflate::CompressionOptions::high(),
    );
    let base64 =
        BASE64_URL_SAFE_NO_PAD.encode(deflated_message_pack.as_slice());
    Some(base64)
}

fn decode_config_str(s: &str) -> Option<Config> {
    let base64 = s;
    let deflated_message_pack = BASE64_URL_SAFE_NO_PAD.decode(base64).ok()?;
    let message_pack =
        inflate::inflate_bytes(deflated_message_pack.as_slice()).ok()?;
    let config = rmp_serde::from_slice(message_pack.as_slice()).ok()?;
    Some(config)
}

fn download_blob(
    document: &web_sys::Document,
    blob: &web_sys::Blob,
    file_name: &str,
) {
    let url = web_sys::Url::create_object_url_with_blob(blob).unwrap();
    download_url(document, &url, file_name);
    web_sys::Url::revoke_object_url(&url).unwrap();
}

fn download_url(document: &web_sys::Document, url: &str, file_name: &str) {
    let anchor = document.create_element("a").unwrap();
    let anchor = anchor.dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
    anchor.set_download(file_name);
    anchor.set_href(url);
    let body = document.body().unwrap();
    body.append_child(&anchor).unwrap();
    anchor.click();
    body.remove_child(&anchor).unwrap();
}

fn defer(window: &web_sys::Window, body: impl FnMut() + 'static) {
    let closure = Closure::<dyn FnMut()>::new(body);
    window
        .set_timeout_with_callback(closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget(); // FIXME: don't leak
}
