use mewo_term::*;
use rust_burrito::*;

struct GamePlugin {}
impl Plugin for GamePlugin {
    fn name() -> &'static str {
        "mewo_sandbox_termbird"
    }
    fn plugin(pb: PluginBuilder) -> PluginBuilder {
        pb.comp::<TermQuad>()
            .sys(|mut args, _: Wish<Startup, (), ()>| {})
            .sys(game_exit)
    }
}

fn game_exit(_args: SystemArgs, wish: Wish<TermKeyEvent, (), ()>) {
    let key = wish.event();
    if key.unicode == 'q' as u32 {
        panic!()
    }
}

fn main() {
    Galaxy::create()
        .plugins(
            RustRuntime::create()
                .plugin::<TermPlugin>()
                .plugin::<GamePlugin>()
                .done(),
        )
        .run::<StraightExecutor>();
}
