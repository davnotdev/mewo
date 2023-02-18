use crate::prelude::*;
use mewo_galaxy::prelude::*;

#[derive(Hash)]
enum GameSet {
    A,
    B,
    C,
    D,
}
impl SystemSet for GameSet {}

#[derive(Debug)]
struct Value(String);
impl Resource for Value {}

//  https://xkcd.com/221/
const RANDOM_NUMBER: usize = 4;

fn f<const C: char>(galaxy: &Galaxy) {
    galaxy
        .get_mut_resource::<Value, _>(RANDOM_NUMBER)
        .unwrap()
        .0
        .push(C);
}

#[test]
fn test_tasker_sets() {
    let mut galaxy = Galaxy::new();
    let mut tasker = Tasker::new();

    tasker
        .systems([
            system(f::<'A'>, GameSet::A),
            system(f::<'B'>, GameSet::B),
            system(f::<'B'>, GameSet::B),
            system(f::<'C'>, GameSet::C),
        ])
        .configure_sets([
            GameSet::B.config(),
            GameSet::C.config().before(GameSet::B),
            GameSet::A.config().after(GameSet::B).after(GameSet::C),
        ]);

    galaxy.insert_resource(RANDOM_NUMBER, Value(String::new()));

    let mut runner = tasker.runner();
    runner.tick_systems(&mut galaxy);

    let value = galaxy.get_resource::<Value, _>(RANDOM_NUMBER).unwrap();
    assert_eq!(value.0, String::from("CBBA"));
}

#[derive(Hash)]
struct EmptyGameState;
impl SystemState for EmptyGameState {}

#[test]
fn test_tasker_states() {
    let mut galaxy = Galaxy::new();
    let mut tasker = Tasker::new();

    tasker
        .systems([
            system(f::<'A'>, GameSet::A),
            system(f::<'B'>, GameSet::B),
            system(f::<'C'>, GameSet::C),
            system(f::<'D'>, GameSet::D),
        ])
        .configure_sets([
            GameSet::A
                .config()
                .on_state(OnSystemState::On(vec![state(Init)])),
            GameSet::B
                .config()
                .on_state(OnSystemState::On(vec![state(EmptyGameState)])),
            GameSet::C
                .config()
                .on_state(OnSystemState::OnExit(vec![state(Init)])),
            GameSet::D
                .config()
                .on_state(OnSystemState::OnEnter(vec![state(EmptyGameState)])),
        ]);

    galaxy.insert_resource(RANDOM_NUMBER, Value(String::new()));

    let mut runner = tasker.runner();

    galaxy.update();
    runner.tick_systems(&mut galaxy);
    galaxy.set_state(state(EmptyGameState));

    galaxy.update();
    runner.tick_systems_transistions(&mut galaxy);
    runner.tick_systems(&mut galaxy);

    let value = galaxy.get_resource::<Value, _>(RANDOM_NUMBER).unwrap();
    assert_eq!(value.0, String::from("ACDB"));
}
