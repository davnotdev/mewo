use mewo_common::prelude::*;
use mewo_galaxy::prelude::*;
use mewo_galaxy_derive::*;
use std::time::Duration;

mod pipe;
mod player;
mod term;

use pipe::{game_pipe_border, game_pipe_despawn, game_pipe_move, game_pipe_spawn_loop, spawn_pipe};
use player::{game_player_border, game_player_gravity, game_player_jump};
use term::{term_init, term_input, term_render};

//  TODO FIX: Game Scale Factor

//  There is no good reason why Player is not a resource besides more testing.

//  Stores (x, y)
#[derive(Clone, Copy, CheapComponent)]
struct Player(f32, f32);

#[derive(Resource)]
struct PlayerEntity(Entity);

//  Stores (top, bottom)
#[derive(Clone, Copy, CheapComponent)]
struct Pipe(PipeSegment, PipeSegment);

//  Stores (x, y, size_x, size_y)
#[derive(Clone, Copy)]
struct PipeSegment(f32, f32, i32, i32);

#[derive(Resource)]
struct TermContext;

impl TermContext {
    pub fn new() -> Self {
        unsafe { termbox_sys::tb_init() };
        TermContext
    }

    pub fn width(&self) -> i32 {
        unsafe { termbox_sys::tb_width() }
    }

    pub fn height(&self) -> i32 {
        unsafe { termbox_sys::tb_height() }
    }
}

impl Drop for TermContext {
    fn drop(&mut self) {
        unsafe { termbox_sys::tb_shutdown() };
    }
}

#[derive(Clone, Resource)]
struct GameBounds(f32, f32);

#[derive(Resource)]
struct PipeSpawnTimer(Timer);

#[derive(Event)]
struct TermKeyEvent {
    pub key: u16,
    pub unicode: u32,
}

const MIN_X: f32 = 90.0;
const MIN_Y: f32 = 28.0;
const PIPE_THICKNESS: i32 = 5;
const PIPE_GAP: i32 = 10;

fn game_init(g: &Galaxy) {
    if let Some(tc) = g.get_resource::<TermContext>() {
        let width = tc.width() as f32;
        let height = tc.height() as f32;

        assert!(width >= MIN_X && height >= MIN_Y);

        let bounds = GameBounds(width, height);
        g.insert_resource(bounds.clone());

        let player = g
            .insert_entity()
            .insert(Player(bounds.0 / 2.0, bounds.1 / 2.0))
            .get_entity();
        g.insert_resource(PlayerEntity(player));

        g.insert_resource(PipeSpawnTimer(Timer::new(Duration::from_millis(1300))));
        spawn_pipe(g).unwrap();
    }
}

fn game_quit(g: &Galaxy) {
    for ev in g.get_events::<TermKeyEvent>() {
        if ev.unicode == 'q' as u32 {
            panic!()
        }
    }
}

fn main() {
    let mut galaxy = Galaxy::new();

    time_init(&galaxy);
    term_init(&galaxy);
    game_init(&galaxy);

    galaxy.update();

    let systems: Vec<fn(&Galaxy)> = vec![
        game_quit,
        term_input,
        term_render,
        game_player_jump,
        game_player_border,
        game_player_gravity,
        game_pipe_move,
        game_pipe_despawn,
        game_pipe_spawn_loop,
        game_pipe_border,
    ];
    loop {
        systems.iter().for_each(|sys| sys(&galaxy));

        galaxy.update();
    }
}
