use super::*;

pub fn game_player_gravity(g: &Galaxy) {
    for player in g.query::<&mut Player>().iter() {
        player.1 += 0.22;
    }
}

pub fn game_player_jump(g: &Galaxy) {
    for ev in g.get_events::<TermKeyEvent>() {
        if ev.key == ' ' as u16 {
            for player in g.query::<&mut Player>().iter() {
                player.1 -= 6.0;
            }
        }
    }
}

pub fn game_player_border(g: &Galaxy) {
    let bounds = g.get_resource::<GameBounds>().unwrap();
    for (e, player) in g.query::<&Player>().eiter() {
        if player.1 >= bounds.1 || player.1 <= 0f32 {
            g.remove_entity(e);
            panic!()
        }
    }
}
