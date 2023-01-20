use mewo_galaxy::prelude::*;
use mewo_window::prelude::*;

fn main() {
    let mut galaxy = Galaxy::new();
    window_init(&galaxy);
    galaxy.update();
    loop {
        for e in galaxy.get_events::<WindowEvent>() {
            eprintln!("{:?}", e);
        }
        window_update(&galaxy);
        galaxy.update();
    }
}
