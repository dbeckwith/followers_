use crate::{color::Color, image::Image, world::World};
use dioxus::{logger::tracing::debug, prelude::*};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{self, AtomicBool, AtomicUsize},
};
use wasm_bindgen::prelude::*;

pub struct WorldRenderer {
    world: Signal<World>,
    image: Rc<RefCell<Image>>,
    context: Rc<RefCell<web_sys::CanvasRenderingContext2d>>,
    paused: Rc<AtomicBool>,
    frame_idx: Rc<AtomicUsize>,
    window: web_sys::Window,
    #[allow(clippy::type_complexity)]
    closure_handle: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
}

impl WorldRenderer {
    pub fn new(
        canvas: &web_sys::HtmlCanvasElement,
        mut world: Signal<World>,
        background: Color,
        frame_limit: Signal<usize>,
    ) -> WorldRenderer {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let width = canvas.width() as usize;
        let height = canvas.height() as usize;
        let mut image = Image::new(width, height, background);
        world.peek().render(&mut image);

        let image_data = image.to_image_data();
        context.put_image_data(&image_data, 0.0, 0.0).unwrap();

        let image = Rc::new(RefCell::new(image));
        let context = Rc::new(RefCell::new(context));
        let paused = Rc::new(AtomicBool::new(false));
        let frame_idx = Rc::new(AtomicUsize::new(0));

        let window = canvas.owner_document().unwrap().default_view().unwrap();

        let closure_handle =
            Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
        let closure = Closure::new({
            let image = Rc::clone(&image);
            let context = Rc::clone(&context);
            let paused = Rc::clone(&paused);
            let frame_idx = Rc::clone(&frame_idx);
            let window = window.clone();
            let closure_handle = Rc::clone(&closure_handle);
            move || {
                if paused.load(atomic::Ordering::SeqCst) {
                    return;
                }
                {
                    let frame_idx_ = frame_idx.load(atomic::Ordering::SeqCst);
                    let frame_limit_ = *frame_limit.peek();
                    if frame_limit_ > 0 && frame_idx_ >= frame_limit_ {
                        paused.store(true, atomic::Ordering::SeqCst);
                        // force a dioxus re-render so paused state is observed
                        world.write();
                        return;
                    }
                }
                debug!("update");
                let mut world = world.write();
                world.update();
                let image = &mut *image.borrow_mut();
                let context = &mut *context.borrow_mut();
                world.render(image);
                let image_data = image.to_image_data();
                context.put_image_data(&image_data, 0.0, 0.0).unwrap();
                frame_idx.fetch_add(1, atomic::Ordering::SeqCst);
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
        debug!("start renderer {}x{}", width, height);
        window
            .request_animation_frame(closure.as_ref().unchecked_ref())
            .unwrap();
        *closure_handle.borrow_mut() = Some(closure);

        WorldRenderer {
            world,
            image,
            context,
            paused,
            frame_idx,
            window,
            closure_handle,
        }
    }

    pub fn paused(&self) -> bool {
        self.paused.load(atomic::Ordering::SeqCst)
    }

    pub fn pause_resume(&mut self) {
        let was_paused = self.paused.fetch_not(atomic::Ordering::SeqCst);
        let resumed = was_paused;
        if resumed {
            self.restart_render_loop();
        }
    }

    pub fn resume(&mut self) {
        let was_paused = self.paused.swap(false, atomic::Ordering::SeqCst);
        let resumed = was_paused;
        if resumed {
            self.restart_render_loop();
        }
    }

    fn restart_render_loop(&mut self) {
        // restart the render loop
        let closure = self.closure_handle.borrow();
        let closure = closure.as_ref().unwrap();
        self.window
            .request_animation_frame(closure.as_ref().unchecked_ref())
            .unwrap();
    }

    pub fn frame_idx(&self) -> usize {
        self.frame_idx.load(atomic::Ordering::SeqCst)
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
        debug!("update renderer {}x{}", width, height);

        let image = &mut *self.image.borrow_mut();
        image.resize(width, height);

        let image_data = image.to_image_data();
        context.put_image_data(&image_data, 0.0, 0.0).unwrap();

        *self.context.borrow_mut() = context;
    }

    pub fn clear(&mut self) {
        let image = &mut *self.image.borrow_mut();
        let context = &*self.context.borrow_mut();

        self.frame_idx.store(0, atomic::Ordering::SeqCst);

        image.clear();
        self.world.peek().render(image);

        let image_data = image.to_image_data();
        context.put_image_data(&image_data, 0.0, 0.0).unwrap();
    }
}
