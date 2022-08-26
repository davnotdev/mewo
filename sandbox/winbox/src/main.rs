use mewo_log::*;
use mewo_window::*;
use rust_burrito::*;

struct GamePlugin;
impl Plugin for GamePlugin {
    fn name() -> &'static str {
        "mewo_sandbox_winbox"
    }

    fn plugin(pb: PluginBuilder) -> PluginBuilder {
        pb.update(|sb: SystemBus<(), ()>| {
            let evs = sb.events.get::<WindowEvent>()?;
            for ev in evs {
                println!("{:?}", ev.0)
            }
            Some(())
        })
    }
}

fn main() {
    let galaxy = Galaxy::create();
    galaxy
        .debug_log_hook(log_hook_stderr())
        .plugin(WindowPlugin::build_plugin(&galaxy))
        .plugin(GamePlugin::build_plugin(&galaxy));
    galaxy.run::<StraightExecutor>()
}
