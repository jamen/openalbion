# OpenAlbion: format

A Rust library for parsing and serializing the many bespoke file formats used by Fable: The Lost Chapters.

This library doesn't manage file I/O or the assets. That is left to a higher level system in the engine or tools.

# Tests

You can run tests can be ran against the game files with the following command

```sh
FABLE_PATH=<path> cargo test
```
