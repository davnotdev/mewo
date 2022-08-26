pub use glfw::{Context, WindowEvent as WindowEventData};
use rust_burrito::*;

pub struct WindowPlugin;
impl Plugin for WindowPlugin {
    fn name() -> &'static str {
        "mewo_window"
    }

    fn plugin(pb: PluginBuilder) -> PluginBuilder {
        pb.bootstrap(|mut sb: SystemBus<(), ()>| {
            if let Some(mut window) = Window::create() {
                window.window.set_all_polling(true);
                sb.resources.insert(window);
                sb.events.event(WindowCreate);
            } else {
                debug_error("Failed to create GLFW Window.");
            }
            Some(())
        })
        .update(|mut sb: SystemBus<(), ()>| {
            let window = sb.resources.get::<&mut Window>().get()?;
            if window.window.should_close() {
                panic!()
            }
            window.glfw.poll_events();
            for (_, event) in glfw::flush_messages(&window.events) {
                sb.events.event(WindowEvent(event));
            }
            //  Might cause problems later?
            window.window.swap_buffers();
            Some(())
        })
    }
}

pub struct WindowCreate;
impl Event for WindowCreate {}

#[derive(Debug)]
pub struct WindowEvent(pub WindowEventData);
impl Event for WindowEvent {}

//  Multiple Windows are not supported.
//  I mean.. What game uses multiple windows anyway?

pub struct Window {
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: std::sync::mpsc::Receiver<(f64, WindowEventData)>,
}
impl Resource for Window {}

impl Window {
    pub fn create() -> Option<Self> {
        let glfw = glfw::init(Some(glfw::Callback {
            f: |_, s, _| {
                debug_error(format!("GLFW Window ~ {}", s));
            },
            data: (),
        }))
        .ok()?;
        let (window, events) = glfw.create_window(640, 480, "TODO", glfw::WindowMode::Windowed)?;
        Some(Window {
            glfw,
            window,
            events,
        })
    }
}
