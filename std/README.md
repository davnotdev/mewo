# Standard Conventions

`mewo_galaxy` is just an ECS.
Perfect right?
Well, there are many questions that are left for the user to decide.
However, to prevent fragmentation and preserve my sanity, here are the answers to
some of those questions.

## Game Root

The game root directory should be called `app_root` and located in the same
directory as `Cargo.toml`.
For games with multiple crates, the game root should be located at the workspace
`Cargo.toml`.

The game root should contain the following:

```default
app_root/
    assets/
```

`mewo_asset` should automatically detect the game root location and set that as
the current working directory of the program.
