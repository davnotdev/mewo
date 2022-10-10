use mewo_galaxy::prelude::*;
use mewo_window::prelude::*;

fn main() {
    let mut galaxy = Galaxy::new();
    window_init(&galaxy);
    galaxy.update();
    galaxy
        .insert_entity()
        .insert(Window::new(&galaxy, 640, 480, "My Window", None));
    galaxy.update();
    loop {
        if galaxy.query::<&Window>().iter().count() == 0 {
            panic!("exit")
        }
        for e in galaxy.get_events::<WindowEvent>() {
            eprintln!("{:?}", e);
        }
        window_update(&galaxy);
        galaxy.update();
    }
}
