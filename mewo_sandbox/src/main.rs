use mewo_ecs::*;

fn main() {
    let mut app = AppBuilder::create()
        .build::<DefaultExecutor>();
    app.run();
}
