#![warn(rust_2018_idioms, clippy::all)]
#![deny(clippy::correctness)]

use nannou::prelude::*;
use nannou_egui::Egui;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _window_id: WindowId,
    egui: Egui,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .title("FOLLOWERS")
        .view(view)
        .raw_event(raw_event)
        .event(event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);
    Model {
        _window_id: window_id,
        egui,
    }
}

fn raw_event(
    _app: &App,
    Model { _window_id, egui }: &mut Model,
    event: &nannou::winit::event::WindowEvent<'_>,
) {
    egui.handle_raw_event(event);
}

fn event(
    app: &App,
    Model { _window_id, egui }: &mut Model,
    event: WindowEvent,
) {
    let gui = egui.ctx();
    if gui.wants_pointer_input() {
        match &event {
            WindowEvent::MouseMoved(_)
            | WindowEvent::MousePressed(_)
            | WindowEvent::MouseReleased(_)
            | WindowEvent::MouseEntered
            | WindowEvent::MouseExited
            | WindowEvent::MouseWheel(..)
            | WindowEvent::Touch(_)
            | WindowEvent::TouchPressure(_) => return,
            _ => {},
        }
    }
    if gui.wants_keyboard_input() {
        match &event {
            WindowEvent::KeyPressed(_)
            | WindowEvent::KeyReleased(_)
            | WindowEvent::ReceivedCharacter(_) => return,
            _ => {},
        }
    }
    match event {
        WindowEvent::KeyPressed(Key::Space) => {},
        event => {},
    }
}

fn update(app: &App, Model { _window_id, egui }: &mut Model, update: Update) {
    egui.set_elapsed_time(update.since_start);
    let gui = egui.begin_frame();
}

fn view(app: &App, Model { _window_id, egui }: &Model, frame: Frame<'_>) {
    let draw = app.draw();
    draw.background().color(hsv(0.0 / 360.0, 0.00, 1.00));
    draw.to_frame(app, &frame).unwrap();
    egui.draw_to_frame(&frame).unwrap();
}
