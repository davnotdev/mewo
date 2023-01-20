use mewo_galaxy::prelude::*;
use mewo_galaxy_derive::*;
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use winit::{
    event::{Event as WinitEvent, WindowEvent as WinitWindowEvent},
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::{Window as WinitWindow, WindowBuilder},
};

//  TODO CHK: glfw-rs broke, so here's a hastily thrown together winit replacement.
//  TODO EXT: Add options for window size, title, fullscreen, etc.

pub mod prelude;

#[derive(SingleResource)]
pub struct Window {
    event_loop: EventLoop<()>,
    window: WinitWindow,
}

impl Window {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        window.set_maximized(true);
        Window { event_loop, window }
    }

    pub fn get_raw_display(&self) -> RawDisplayHandle {
        self.event_loop.raw_display_handle()
    }

    pub fn get_raw_window(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }

    pub fn get_width(&self) -> u32 {
        self.window.inner_size().width
    }

    pub fn get_height(&self) -> u32 {
        self.window.inner_size().height
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Event)]
pub struct WindowEvent<'a>(pub WinitWindowEvent<'a>);

pub fn window_init(g: &Galaxy) {
    g.insert_resource(Window::single_resource(), Window::new());
}

pub fn window_update(g: &Galaxy) {
    let Some(mut window) = g.get_mut_resource::<Window, _>(Window::single_resource()) else {
        return merr!("window_init not called");
    };
    window.event_loop.run_return(move |event, _, control_flow| {
        control_flow.set_wait();
        match event {
            WinitEvent::WindowEvent {
                event: WinitWindowEvent::CloseRequested { .. },
                ..
            } => {
                panic!("window closed");
            }
            WinitEvent::WindowEvent { event, .. } => {
                g.insert_event(WindowEvent(event.to_static().unwrap()));
            }
            WinitEvent::MainEventsCleared => {
                control_flow.set_exit();
            }
            _ => {}
        };
    });
}
