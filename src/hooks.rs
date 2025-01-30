use dioxus::{prelude::*, web::WebEventExt};
use wasm_bindgen::prelude::*;

pub fn use_element<T: JsCast>(
) -> (ReadOnlySignal<Option<T>>, Callback<Event<MountedData>>) {
    let mut element = use_signal(|| None::<T>);
    let on_mounted = use_callback(move |event: Event<MountedData>| {
        element.set(event.data().as_web_event().dyn_into().ok());
    });
    (element.into(), on_mounted)
}
