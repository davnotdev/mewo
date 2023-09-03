//! # Mewo
//!
//! ## Driving the Car
//!
//! ```rust,ignore
//! //  For single threaded use cases.
//! //  let galaxy = Galaxy::new();
//! let galaxy = Arc::new(RwLock::new(Galaxy::new()));
//!
//! {
//!     let mut galaxy = galaxy.write();
//!     
//!     //  Initialize Game.
//!     some_init_system(&galaxy);
//!
//!     galaxy.update();
//! }
//!
//! //  Although you can do this, you should prefer the built-in runner functions.
//! //
//! //  //  Game Loop
//! //  loop {
//! //      let galaxy = galaxy.write();
//! //      system_one(&galaxy);
//! //      system_two(&galaxy);
//! //      if galaxy.update().is_none() {
//! //          //  Game Over
//! //          return;
//! //      }
//! //  }
//!
//! run_spawn(Arc::clone(&galaxy)).join().unwrap();
//!
//! ```
//!
//! ## Defining Components
//! 
//! Components are pieces of data that are attached to entities.
//! There are two types of components: `CheapComponent` and `UniqueComponent`.
//!
//! ```rust,ignore
//! #[derive(Clone, Copy, CheapComponent)]
//! struct A;
//!
//! #[derive(Clone, UniqueComponent)]
//! struct B;
//! ```
//! 
//! `CheapComponent` is for components that are cheap to copy.
//! `UniqueComponent` is for components that either don't implement copy or shouldn't be copied too
//! often.
//!
//! ## Defining Resources 
//!
//! Resources are pieces of data that just exist and can be accessed with some value.
//!
//! ```rust,ignore
//! #[derive(SingleResource)]
//! struct PlayerEntity(Entity);
//!
//! #[derive(Resource)]
//! struct Window(...);
//! ```
//!
//! Here, `PlayerEntity` is a single resource which can be accessed using
//! `PlayerEntity::single_resource()`.
//! There is only one `PlayerEntity`.
//!
//!
//! However, there can be multiple of `Window`.
//! You can access each window later using literally any value.
//!
//! ## Systems
//!
//! Systems are just functions that take a galaxy.
//!
//! ```rust,ignore
//! fn my_system(galaxy: &Galaxy) {
//!     todo!();
//! }
//! ```
//!
//! ## Spawning an Entity
//!
//! ```rust,ignore
//! let player = galaxy
//!     .insert_entity()
//!     .insert(Player)
//!     .insert(SomeComponent)
//!     .insert(OtherComponent)
//!     .get_entity();
//! ```
//! 
//! This creates an entity, however, this entity is not accessible until the next update.
//!
//! ## Queries
//!
//! ```rust,ignore
//! //  With Entity
//! for (entity, (player, health, other)) in g.query::<(&Player, &mut Health, Option<&Other>)>().eiter() {
//!     //  ...
//! }
//!
//! //  Without Entity
//! for (player, health, other) in g.query::<(&Player, &mut Health, Option<&Other>)>().iter() {
//!     //  ...
//! }
//! ```
//!
//! ## Getting a Specific Entity
//!
//! ```rust,ignore
//! let entity_getter = galaxy.get_entity(entity).unwrap();
//!
//! entity_getter
//!     .insert(SomeComponent)
//!     .insert(OtherComponent)
//!     .remove(SomeOtherComponent);
//!
//! //  I don't see when you would use this.
//! let entity = entity_getter.get_entity();
//! ```
//!
//! Once again, both inserted and removed components don't show until the next update.
//!
//! ## Spawning a Resource
//!
//! ```rust,ignore
//! //  For single resources.
//! galaxy.insert_resource(PlayerEntity::single_resource(), PlayerEntity(player));
//!
//! //  For (generic) resources.
//! //  This associates the new window with the string `"My Window"`.
//! galaxy.insert_resource("My Window", Window(window));
//! ```
//!
//! Resources that are created are instantly available unlike with components.
//!
//! ## Accessing a Resource
//! 
//! ```rust,ignore
//! galaxy
//!     .get_resource::<PlayerEntity, _>(PlayerEntity::single_resource())
//!     .unwrap()
//!
//! galaxy
//!     .get_resource::<Window, _>("My Window")
//!     .unwrap()
//! ```
//!
//! ## Removing Stuff
//!
//! ```rust,ignore
//! galaxy.remove_entity(entity);
//!
//! galaxy.remove_resource::<Window, _>("My Window");
//! galaxy.remove_resource::<PlayerEntity, _>(PlayerEntity::single_resource());
//! ```
//!
//! ## Events
//!
//! ```rust,ignore 
//! #[derive(Event)]
//! struct KeyEvent {
//!     pub key: Key,
//! }
//! 
//! galaxy.insert_event(KeyEvent { key });
//!
//! for event in galaxy.get_events() {
//!     todo!()
//! }
//! ```
//!
//! Similar to components, inserted events don't appear until the next update.
//!
//! ## Game Over
//!
//! ```rust,ignore
//! galaxy.set_exit();
//! ```
//!

pub use mewo_ecs::*;
#[cfg(feature = "derive")]
pub use mewo_ecs_derive::*;
