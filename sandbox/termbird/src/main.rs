use mewo_term::*;
use rust_burrito::*;

struct GamePlugin {}
impl Plugin for GamePlugin {
    fn name() -> &'static str {
        "mewo_sandbox_termbird"
    }
    fn plugin(pb: PluginBuilder) -> PluginBuilder {
        pb.comp::<Player>()
            .sys(|mut sb: SystemBus, _: Wish<TermInitEvent, (), ()>| {
                if let Some(term_ctx) = sb.resources.get::<TermContext>() {
                    let x = term_ctx.width() / 2;
                    let y = term_ctx.height() / 2;
                    sb.entities.spawn().insert(Player).insert(TermQuad::create(
                        (x, y),
                        TermQuadType::Dot,
                        None,
                        None,
                    ));
                }
            })
            .sys(game_exit)
            .sys(game_player_gravity)
            .sys(game_player_border)
    }
}

#[derive(Clone)]
struct Player;
impl Component for Player {
    fn component_is_copy() -> bool {
        true
    }
}

fn game_exit(_: SystemBus, w: Wish<TermKeyEvent, (), ()>) {
    let key = w.event();
    if key.unicode == 'q' as u32 {
        panic!()
    }
}

fn game_player_gravity(_: SystemBus, w: Wish<(), &mut TermQuad, With<Player>>) {
    for quad in w.iter() {
        quad.position.1 += 1;
    }
}

fn game_player_border(mut sb: SystemBus, w: Wish<(), &mut TermQuad, With<Player>>) {
    if let Some(term_ctx) = sb.resources.get::<TermContext>() {
        for (e, quad) in w.eiter() {
            let h = term_ctx.height();
            if quad.position.1 >= h || quad.position.1 <= 0 {
                sb.entities.despawn(e);
                panic!();
            }
        }
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
