use glfw::{Context, WindowEvent as WindowEventData, WindowMode};
use mewo_galaxy::prelude::*;
use mewo_galaxy_derive::*;

pub mod prelude;

#[derive(Resource)]
pub struct WindowContext {
    glfw: glfw::Glfw,
}

impl WindowContext {
    pub fn new() -> Self {
        WindowContext {
            glfw: glfw::init(Some(glfw::Callback {
                f: |_, s, _| {
                    merr!("init GLFW {}", s);
                },
                data: (),
            }))
            .unwrap(),
        }
    }
}

#[derive(UniqueComponent)]
pub struct Window(
    glfw::Window,
    std::sync::mpsc::Receiver<(f64, WindowEventData)>,
);

impl Window {
    pub fn new(
        g: &Galaxy,
        width: usize,
        height: usize,
        name: &str,
        mode: Option<WindowMode>,
    ) -> Self {
        let window_ctx = g.get_resource::<WindowContext>().unwrap();
        let (mut window, events) = window_ctx
            .glfw
            .create_window(
                width as u32,
                height as u32,
                name,
                mode.unwrap_or(WindowMode::Windowed),
            )
            .unwrap();
        window.set_all_polling(true);
        Window(window, events)
    }
}

#[derive(Debug, Event)]
pub struct WindowEvent(pub Entity, pub WindowEventData);

pub fn window_init(g: &Galaxy) {
    g.insert_resource(WindowContext::new());
}

pub fn window_update(g: &Galaxy) {
    if let Some(mut window_ctx) = g.get_mut_resource::<WindowContext>() {
        for (e, window) in g.query::<&mut Window>().eiter() {
            if window.0.should_close() {
                g.remove_entity(e);
            }

            //  TODO CHK: Not sure if this is safe.
            //  window.0.glfw.poll_events();

            window_ctx.glfw.poll_events();

            for (_, event) in glfw::flush_messages(&window.1) {
                g.insert_event(WindowEvent(e, event));
            }

            //  Might cause problems later?
            window.0.swap_buffers();
        }
    } else {
        merr!("window_init not called");
    }
}
