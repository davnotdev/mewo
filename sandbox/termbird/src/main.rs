use mewo_common::time::*;
use mewo_term::*;
use rand::prelude::*;
use rust_burrito::*;

const ASPECT_RATIO_X: f32 = 90.0;
const ASPECT_RATIO_Y: f32 = 28.0;

struct GamePlugin;

impl Plugin for GamePlugin {
    fn name() -> &'static str {
        "termbird"
    }

    fn plugin(pb: PluginBuilder) -> PluginBuilder {
        pb.startup(|mut sb: SystemBus<(), ()>| {
            let term_ctx = sb.resources.get::<&TermContext>().get()?;
            assert!(
                term_ctx.width() as f32 >= ASPECT_RATIO_X
                    && term_ctx.height() as f32 >= ASPECT_RATIO_Y
            );
            let bounds = GameBounds(ASPECT_RATIO_X, ASPECT_RATIO_Y);
            sb.resources.insert(bounds);
            sb.resources.insert(ObsticleSpawnTimer(Timer::create(
                std::time::Duration::from_millis(1300),
            )));
            sb.entities.spawn().insert(Bird).insert(TermQuad::create(
                (bounds.0 / 2.0, bounds.1 / 2.0),
                TermQuadType::Dot,
                None,
                None,
            ));
            spawn_obsticle(&mut sb);
            Some(())
        })
        .update(game_quit)
        .update(game_bird_jump)
        .update(game_bird_gravity)
        .update(game_bird_border)
        .update(game_obsticle_move)
        .update(game_obsticle_border)
        .update(game_spawn_loop)
        .update(game_collider_check)
    }
}

#[derive(Clone, Copy)]
struct GameBounds(f32, f32);
impl Resource for GameBounds {}

#[derive(Clone)]
struct Bird;
impl Component for Bird {}

#[derive(Clone)]
struct Obsticle;
impl Component for Obsticle {}

impl Obsticle {
    fn gap() -> f32 {
        10.0
    }
    fn thickness() -> f32 {
        5.0
    }
}

struct ObsticleSpawnTimer(Timer);
impl Resource for ObsticleSpawnTimer {}

fn game_quit(sb: SystemBus<(), ()>) -> Option<()> {
    let evs = sb.events.get::<TermKeyEvent>()?;
    for ev in evs {
        if ev.unicode == 'q' as u32 {
            panic!()
        }
    }
    Some(())
}

fn game_bird_jump(sb: SystemBus<&mut TermQuad, With<Bird>>) -> Option<()> {
    let evs = sb.events.get::<TermKeyEvent>()?;
    for ev in evs {
        if ev.key == ' ' as u16 {
            for quad in sb.components.iter() {
                quad.position.1 -= 6.0;
            }
        }
    }
    Some(())
}

fn game_bird_gravity(sb: SystemBus<&mut TermQuad, With<Bird>>) {
    for quad in sb.components.iter() {
        quad.position.1 += 0.22;
    }
}

fn game_bird_border(mut sb: SystemBus<&TermQuad, With<Bird>>) -> Option<()> {
    let bounds = sb.resources.get::<&GameBounds>().get()?;
    for (e, quad) in sb.components.eiter() {
        if quad.position.1 >= bounds.1 || quad.position.1 <= 0f32 {
            sb.entities.despawn(e);
            panic!()
        }
    }
    Some(())
}

fn spawn_obsticle<CA, CF>(sb: &mut SystemBus<CA, CF>) -> Option<()> {
    let height = sb.resources.get::<&GameBounds>().get()?.1;
    let min = 2;
    let max = (height - (Obsticle::gap() - 1.0)) as i32;
    let mut rng = rand::thread_rng();
    let top_height = rng.gen_range(min..max) as f32;
    sb.entities
        .spawn()
        .insert(Obsticle)
        .insert(TermQuad::create(
            (0.0, 0.0),
            TermQuadType::Hollow((Obsticle::thickness(), top_height)),
            None,
            None,
        ));
    let bottom_y = top_height + Obsticle::gap();
    let bottom_height = height - bottom_y;
    sb.entities
        .spawn()
        .insert(Obsticle)
        .insert(TermQuad::create(
            (0.0, bottom_y),
            TermQuadType::Hollow((Obsticle::thickness(), bottom_height)),
            None,
            None,
        ));
    Some(())
}

fn game_obsticle_move(sb: SystemBus<&mut TermQuad, With<Obsticle>>) {
    for quad in sb.components.iter() {
        quad.position.0 += 0.5;
    }
}

fn game_obsticle_border(mut sb: SystemBus<&TermQuad, With<Obsticle>>) -> Option<()> {
    let bounds = sb.resources.get::<&GameBounds>().get()?;
    for (e, quad) in sb.components.eiter() {
        if quad.position.0 > bounds.0 {
            sb.entities.despawn(e);
        }
    }
    Some(())
}

fn game_spawn_loop(mut sb: SystemBus<(), ()>) {
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

fn game_collider_check(
    sb: SystemBus<(&TermQuad, Option<&Bird>, Option<&Obsticle>), ()>,
) -> Option<()> {
    let bird = sb
        .components
        .iter()
        .filter(|(_, bird, _)| bird.is_some())
        .next()?
        .0;
    for obsticle in sb.components.iter() {
        if let (obsticle, None, Some(_)) = obsticle {
            if bird.position.0 >= obsticle.position.0
                && bird.position.0 <= obsticle.position.0 + obsticle.qtype.width()
                && bird.position.1 >= obsticle.position.1
                && bird.position.1 <= obsticle.position.1 + obsticle.qtype.height()
            {
                panic!()
            }
        }
    }
    Some(())
}

fn main() {
    let galaxy = Galaxy::create();
    galaxy
        .debug_dump_hook(mewo_log::dump_hook_file(
            Some("/tmp/dump".to_string()),
            None,
        ))
        .debug_log_hook(mewo_log::log_hook_file(Some("/tmp/log".to_string())))
        .plugins(vec![
            TimePlugin::build_plugin(&galaxy),
            TermPlugin::build_plugin(&galaxy),
            GamePlugin::build_plugin(&galaxy),
        ]);
    galaxy.run::<StraightExecutor>()
}
