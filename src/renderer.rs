use crate::{color::Color, image::Image, world::World};
use dioxus::prelude::*;
use log::debug;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

pub struct WorldRenderer {
    image: Rc<RefCell<Image>>,
    context: Rc<RefCell<web_sys::CanvasRenderingContext2d>>,
    #[allow(clippy::type_complexity)]
    _closure_handle: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
}

impl WorldRenderer {
    pub fn new(
        canvas: &web_sys::HtmlCanvasElement,
        mut world: Signal<World>,
    ) -> WorldRenderer {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let width = canvas.width() as usize;
        let height = canvas.height() as usize;
        let image = Image::new(width, height, Color::transparent());

        let image = Rc::new(RefCell::new(image));
        let context = Rc::new(RefCell::new(context));

        let window = canvas.owner_document().unwrap().default_view().unwrap();

        let closure_handle =
            Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
        let closure = Closure::new({
            let image = Rc::clone(&image);
            let context = Rc::clone(&context);
            let window = window.clone();
            let closure_handle = Rc::clone(&closure_handle);
            move || {
                debug!("update");
                let image = &mut *image.borrow_mut();
                let context = &mut *context.borrow_mut();
                world.write().update(image);
                let image_data = image.to_image_data();
                context.put_image_data(&image_data, 0.0, 0.0).unwrap();
                window
                    .request_animation_frame(
                        closure_handle
                            .borrow()
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unchecked_ref(),
                    )
                    .unwrap();
            }
        });
        debug!("start render {}x{}", width, height);
        window
            .request_animation_frame(closure.as_ref().unchecked_ref())
            .unwrap();
        *closure_handle.borrow_mut() = Some(closure);

        WorldRenderer {
            image,
            context,
            _closure_handle: closure_handle,
        }
    }

    pub fn update(&mut self, canvas: &web_sys::HtmlCanvasElement) {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let width = canvas.width() as usize;
        let height = canvas.height() as usize;
        debug!("update render {}x{}", width, height);

        self.image.borrow_mut().resize(width, height);
        *self.context.borrow_mut() = context;
    }
}
