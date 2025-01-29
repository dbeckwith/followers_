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
    rsx! {
        span {
            "Hello, world!"
        }
    }
}
