use super::*;
use rand::prelude::*;

//  height = bottom height
fn new_pipe(pos: f32, height: i32, bounds: &GameBounds) -> Pipe {
    let top = PipeSegment(
        pos - 1.0,
        -1.0,
        PIPE_THICKNESS,
        bounds.1 as i32 - (height + PIPE_GAP),
    );
    let bottom = PipeSegment(pos, bounds.1 - height as f32, PIPE_THICKNESS, height);
    Pipe(top, bottom)
}

pub fn spawn_pipe(g: &Galaxy) -> Option<()> {
    let bounds = g.get_resource::<GameBounds, _>(GameBounds::single_resource())?;
    let mut rng = rand::thread_rng();
    let pipe_height = rng.gen_range(2..(bounds.1 as i32 - PIPE_GAP));
    g.insert_entity()
        .insert(new_pipe(0.0, pipe_height, &bounds));
    Some(())
}

pub fn game_pipe_spawn_loop(g: &Galaxy) {
    let mut time = g
        .get_mut_resource::<GlobalTime, _>(GlobalTime::single_resource())
        .unwrap();
    let mut timer = g
        .get_mut_resource::<PipeSpawnTimer, _>(PipeSpawnTimer::single_resource())
        .unwrap();

    if timer.0.tick(time.delta_time()).passed() {
        spawn_pipe(g).unwrap();
    }
}

pub fn game_pipe_move(g: &Galaxy) {
    for pipe in g.query::<&mut Pipe>().iter() {
        pipe.0 .0 += 0.5;
        pipe.1 .0 += 0.5;
    }
}

pub fn game_pipe_despawn(g: &Galaxy) {
    let bounds = g
        .get_resource::<GameBounds, _>(GameBounds::single_resource())
        .unwrap();
    for (e, pipe) in g.query::<&Pipe>().eiter() {
        if pipe.0 .0 >= bounds.0 {
            g.remove_entity(e);
        }
    }
}

pub fn game_pipe_border(g: &Galaxy) {
    let player = g
        .get_resource::<PlayerEntity, _>(PlayerEntity::single_resource())
        .unwrap();
    let mut player = g.get_entity(player.0).unwrap();
    let player_pos = player.get::<&Player>().unwrap().get();

    // for player_pos in g.query::<&Player>().iter() {
    for pipe in g.query::<&Pipe>().iter() {
        if (player_pos.0 >= pipe.0 .0
            && player_pos.0 <= pipe.0 .0 + pipe.0 .2 as f32
            && player_pos.1 >= pipe.0 .1
            && player_pos.1 <= pipe.0 .1 + pipe.0 .3 as f32)
            || (player_pos.0 >= pipe.1 .0
                && player_pos.0 <= pipe.1 .0 + pipe.1 .2 as f32
                && player_pos.1 >= pipe.1 .1
                && player_pos.1 <= pipe.1 .1 + pipe.1 .3 as f32)
        {
            g.remove_entity(player.get_entity());
            panic!();
        }
    }
    // }
}
