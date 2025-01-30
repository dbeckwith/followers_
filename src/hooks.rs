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

pub fn use_resize_observer<
    TElement: AsRef<web_sys::Element> + PartialEq + Clone + 'static,
>(
    target: Option<TElement>,
    on_resize: Callback<Option<web_sys::DomRect>>,
) {
    struct ResizeObserver {
        observer: web_sys::ResizeObserver,
        _handler: Closure<dyn Fn(Vec<web_sys::ResizeObserverEntry>)>,
    }

    impl Drop for ResizeObserver {
        fn drop(&mut self) {
            self.observer.disconnect();
        }
    }

    let mut observer_handle = use_signal(|| None::<ResizeObserver>);
    use_effect(use_reactive(
        (&target, &on_resize),
        move |(target, on_resize)| {
            observer_handle.set(None);
            if let Some(target) = target {
                let handler = Closure::new(
                    move |entries: Vec<web_sys::ResizeObserverEntry>| {
                        for entry in entries {
                            on_resize(Some(
                                entry.target().get_bounding_client_rect(),
                            ));
                        }
                    },
                );
                let observer = web_sys::ResizeObserver::new(
                    handler.as_ref().unchecked_ref(),
                )
                .unwrap();
                observer.observe(target.as_ref());
                let observer = ResizeObserver {
                    observer,
                    _handler: handler,
                };
                observer_handle.set(Some(observer));
            }
        },
    ));
}

#[derive(Debug, Clone, Copy)]
pub struct ElementSize {
    pub width: f64,
    pub height: f64,
}

pub fn use_element_size<
    TElement: AsRef<web_sys::Element> + PartialEq + Clone + 'static,
>(
    target: Option<TElement>,
) -> ReadOnlySignal<Option<ElementSize>> {
    let mut size = use_signal(|| None::<ElementSize>);
    use_resize_observer(
        target,
        use_callback(move |rect: Option<web_sys::DomRect>| {
            size.set(rect.map(|rect| ElementSize {
                width: rect.width(),
                height: rect.height(),
            }));
        }),
    );
    size.into()
}
