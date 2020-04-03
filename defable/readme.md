# Defable

## Building

```
cargo build
```

Must use `i686-pc-windows-msvc` target.

## Packaging

Install [`cargo-wix`](https://github.com/volks73/cargo-wix) with

```
cargo install cargo-wix
```

From this project's directory, use it with

```
cargo wix --nocapture
```

Then see `target/defable-*.msi` at the root directory

Note its **not** in the project's target `target` directory. This is a safely ignored issue with `cargo-wix` in multi-crate projects.