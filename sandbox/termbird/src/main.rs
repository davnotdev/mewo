//  This is the first ever usage of mewo, 
//  so I'm super stoked. :D (and nervous too)

use mewo_ecs::*;
use mewo_term::*;
use mewo_common::*;

fn main() {
    let mut app = App::builder()
        .plugin(TimePlugin)
        .plugin(TermPlugin)
        .plugin(GamePlugin)
        .build::<DefaultExecutor>();
    app.run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn name() -> &'static str {
        "termbird_game"
    }

    fn plugin(a: &mut App) {
        let cmds = a.commands();
        // cmds.spawn_entity(Some(|e: &mut EntityWrapper| {
        //     e.insert_component(Player { y: 5 });
        //     e.insert_component(TermQuad::create(()))
        // }))
        spawn_obsticle(cmds);
        a
            .dep(TermPlugin)
            .component::<Player>()
            .component::<Obsticle>()
            .sys(game_exit)
            .sys(game_obsticle_move)
            .sys(game_obsticle_despawn)
        ;
    }
}

struct Player {
    y: i32,
}
impl Component for Player {}

const OBSTICLE_GAP: i32 = 20;
const OBSTICLE_WIDTH: i32 = 5;

struct Obsticle {
    top: i32,
}
impl Component for Obsticle {}

fn game_exit(_args: &mut SystemArgs, wish: Wish<&TermKeyEvent, ()>) {
    for key in wish.iter() {
        if key.unicode == 'q' as u32 {
            panic!("std::process::exit doesn't call drop???")
        }
    } 
}

fn game_player_move(_args: &mut SystemArgs, wish: (Wish<&TermKeyEvent, ()>, Wish<&mut TermQuad, With<Player>>)) {
    let (keys, players) = wish;
    for key in keys.iter() {
        if key.unicode == ' ' as u32 {
            for player in players.iter() {
                player.position.1 += 12;
            }
        }
    }
}

fn game_player_gravity(_args: &mut SystemArgs, wish: Wish<&mut TermQuad, With<Player>>) {
    for player in wish.iter() {
        player.position.1 -= 1;
    }
}

fn game_obsticle_move(_args: &mut SystemArgs, wish: Wish<&mut TermQuad, With<Obsticle>>) {
    for quad in wish.iter() {
        quad.position.0 += 1;
    }
}

fn spawn_obsticle(cmds: &mut WorldCommands) {
    cmds.spawn_entity(Some(|e: &mut EntityWrapper| {
        e.insert_component(TermQuad::create((2, 2), TermQuadType::Hollow((10, 10)), None, None));
        e.insert_component(Obsticle {
            top: 0,
        });
    }))
}

fn game_obsticle_despawn(args: &mut SystemArgs, wish: Wish<&TermQuad, With<Obsticle>>) {
    let term = args.rmgr.get::<TermContext>().unwrap();
    for (e, quad) in wish.eiter() {
        if quad.position.0 > term.width() {
            args.cmds.remove_entity(e);
            spawn_obsticle(args.cmds);
        }
    }
}

