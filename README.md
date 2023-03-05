# Mewo Toolkit

Yet Another Rust ECS Game Engine.
Not ready for anything yet, but hopefully, it'll be usable soon!

There are more todos sprinkled everywhere
`grep -ri --exclude=README.md --exclude-dir=target TODO .`,
but I consider those low priority.

## Development Status

Crates are either
Usable (K),
borked (B),
incomplete (I),
untested (T),
or excluded due to broken dependencies (D).

> By "Usable", I mean that the crate has enough functionality to have a purpose
/ use.

| Create               | Functionality | Clippy |
| -------------------- | ------------- | ------ |
| `mewo_galaxy`        | K             | K      |
| `mewo_galaxy_derive` | K             | K      |
| `mewo_asset`         | B             | B      |
| `mewo_common`        | K             | K      |
| `mewo_tasker`        | K             | K      |
| `mewo_window`        | K             | K      |
| `termbird`           | K             | K      |
| `winbox`             | K             | K      |

### Additional Notes

- `mewo_gpu` has been moved [here](https://github.com/davnotdev/mepeyew).
- `mewo_asset` has not been tested and broke with the addition of plural resources.
