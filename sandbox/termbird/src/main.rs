use mewo_common::time::*;
use mewo_term::*;
use rand::prelude::*;
use rust_burrito::*;

struct GamePlugin {}
impl Plugin for GamePlugin {
    fn name() -> &'static str {
        "mewo_sandbox_termbird"
    }
    fn plugin(pb: PluginBuilder) -> PluginBuilder {
        pb.comp::<Player>()
            .comp::<Obsticle>()
            .comp::<Collider>()
            .event::<ObsticleColliderCheckEvent>()
            .resource::<ObsticleSpawnTimer>()
            .sys(|mut sb, _: Events<TermInitEvent>, _: Components<(), ()>| {
                if let Some(term_ctx) = sb.resources.get::<&TermContext>().get() {
                    let x = term_ctx.width() / 2;
                    let y = term_ctx.height() / 2;
                    sb.entities
                        .spawn()
                        .insert(Player)
                        .insert(Collider {
                            x: x as f32,
                            y: y as f32,
                            w: 1.0,
                            h: 1.0,
                        })
                        .insert(TermQuad::create((x, y), TermQuadType::Dot, None, None));
                }
                sb.resources.insert(ObsticleSpawnTimer(Timer::create(
                    std::time::Duration::from_secs(3),
                )));
                spawn_obsticle(&mut sb);
            })
            .sys(game_exit)
            .sys(game_sync_pos)
            .sys(game_player_jump)
            .sys(game_player_gravity)
            .sys(game_player_border)
            .sys(game_obsticle_move)
            .sys(game_obsticle_border)
            .sys(game_spawn_loop)
            .sys(game_collider_send)
            .sys(game_collider_check)
    }
}

struct ObsticleSpawnTimer(Timer);
impl Resource for ObsticleSpawnTimer {}

#[derive(Clone)]
struct Player;
impl Component for Player {
    fn component_is_copy() -> bool {
        true
    }
}

#[derive(Clone)]
struct Obsticle;
impl Component for Obsticle {
    fn component_is_copy() -> bool {
        true
    }
}

impl Obsticle {
    fn gap() -> i32 {
        10
    }
    fn thickness() -> i32 {
        5
    }
}

#[derive(Clone, Copy)]
struct Collider {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}
impl Component for Collider {
    fn component_is_copy() -> bool {
        true
    }
}

fn game_exit(_: SystemBus, e: Events<TermKeyEvent>, _: Components<(), ()>) {
    if e.unicode == 'q' as u32 {
        panic!()
    }
}

fn game_sync_pos(_: SystemBus, _: Events<()>, c: Components<(&mut TermQuad, &Collider), ()>) {
    for (quad, collider) in c.iter() {
        quad.position.0 = collider.x as i32;
        quad.position.1 = collider.y as i32;
    }
}

fn game_player_jump(
    _: SystemBus,
    e: Events<TermKeyEvent>,
    c: Components<(&Player, &mut Collider), ()>,
) {
    if e.key == ' ' as u16 {
        for (_, collider) in c.iter() {
            collider.y -= 8f32;
        }
    }
}

fn game_player_gravity(_: SystemBus, _: Events<()>, c: Components<(&Player, &mut Collider), ()>) {
    for (_, collider) in c.iter() {
        //  Acceleration would be nice!
        collider.y += 0.3;
    }
}

fn game_player_border(mut sb: SystemBus, _: Events<()>, c: Components<(&Player, &Collider), ()>) {
    if let Some(term_ctx) = sb.resources.get::<&TermContext>().get() {
        for (e, (_, collider)) in c.eiter() {
            let h = term_ctx.height();
            if collider.y >= h as f32 || collider.y <= 0f32 {
                sb.entities.despawn(e);
                panic!()
            }
        }
    }
}

fn spawn_obsticle(sb: &mut SystemBus) {
    let height = if let Some(term_ctx) = sb.resources.get::<&TermContext>().get() {
        term_ctx.height()
    } else {
        panic!()
    };
    let min = 1;
    let max = height - Obsticle::gap() - 1;
    let mut rng = rand::thread_rng();
    let top_height = rng.gen_range(min..max);
    sb.entities
        .spawn()
        .insert(Obsticle)
        .insert(Collider {
            x: 0f32,
            y: 0f32,
            w: Obsticle::thickness() as f32,
            h: top_height as f32,
        })
        .insert(TermQuad::create(
            (0, 0),
            TermQuadType::Hollow((Obsticle::thickness(), top_height)),
            None,
            None,
        ));
    let bottom_y = top_height + Obsticle::gap();
    let bottom_height = height - bottom_y;
    sb.entities
        .spawn()
        .insert(Obsticle)
        .insert(Collider {
            x: 0f32,
            y: bottom_y as f32,
            w: Obsticle::thickness() as f32,
            h: bottom_height as f32,
        })
        .insert(TermQuad::create(
            (0, bottom_y),
            TermQuadType::Hollow((Obsticle::thickness(), bottom_height)),
            None,
            None,
        ));
}

fn game_obsticle_move(_: SystemBus, _: Events<()>, c: Components<(&Obsticle, &mut Collider), ()>) {
    for (_, collider) in c.iter() {
        collider.x += 0.75;
    }
}

fn game_obsticle_border(
    mut sb: SystemBus,
    _: Events<()>,
    c: Components<(&Obsticle, &Collider), ()>,
) {
    if let Some(term_ctx) = sb.resources.get::<&TermContext>().get() {
        for (e, (_, obsticle)) in c.eiter() {
            if obsticle.x as i32 > term_ctx.width() {
                sb.entities.despawn(e);
            }
        }
    }
}

fn game_spawn_loop(mut sb: SystemBus, _: Events<()>, _: Components<(), ()>) {
    let mut should_spawn = false;
    if let Some((time, timer)) = sb
        .resources
        .get::<(&mut Time, &mut ObsticleSpawnTimer)>()
        .get()
    {
        if timer.0.tick(time.delta_time()).passed() {
            should_spawn = true;
        }
    }

    if should_spawn {
        spawn_obsticle(&mut sb);
    }
}

//  TODO Solve me!
//  This is a very yucky solution for collision detection. It's not possible to iterate though
//  every single collider due to how storages are implemented. Therefore, an event is sent for each
//  obsticle and then processed. This solution works here but probably wouldn't work for a more
//  advanced game.

struct ObsticleColliderCheckEvent(Collider);
impl Event for ObsticleColliderCheckEvent {}

fn game_collider_send(mut sb: SystemBus, _: Events<()>, c: Components<&Collider, With<Obsticle>>) {
    for obsticle in c.iter() {
        sb.events.event(ObsticleColliderCheckEvent(*obsticle));
    }
}

fn game_collider_check(
    _: SystemBus,
    e: Events<ObsticleColliderCheckEvent>,
    c: Components<&Collider, With<Player>>,
) {
    let obsticle = e.0;
    for player in c.iter() {
        if player.x >= obsticle.x
            && player.x <= obsticle.x + obsticle.w
            && player.y >= obsticle.y
            && player.y <= obsticle.y + obsticle.h
        {
            panic!()
        }
    }
}

fn main() {
    Galaxy::create()
        .plugins(
            RustRuntime::create()
                .plugin::<TimePlugin>()
                .plugin::<TermPlugin>()
                .plugin::<GamePlugin>()
                .done(),
        )
        .run::<StraightExecutor>();
}
