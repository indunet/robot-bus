# Tool nodes

Executable utility nodes that sit on top of the `robot-bus` SDK. They are **workspace members**, not part of the core library crate, so heavy / system dependencies (FFmpeg, etc.) stay out of the default SDK build and crates.io package.

## Layout

```text
nodes/
├── README.md                 # this file
└── <snake_name>/
    ├── Cargo.toml            # package name: robot-bus-<kebab-name>
    ├── src/
    │   ├── main.rs           # clap CLI → run()
    │   ├── lib.rs            # library surface for unit tests
    │   └── …                 # node logic
    └── config/
        └── example.yaml      # sample parameters
```

## Add a new tool node

1. Create `nodes/<snake_name>/` with the layout above.
2. In that crate's `Cargo.toml`, set `name = "robot-bus-<kebab-name>"` and depend on the workspace SDK:

   ```toml
   robot-bus = { path = "../.." }
   ```

3. Add the path to the root workspace `members` in `/Cargo.toml` (keep `default-members = ["."]` so plain `cargo test` does not require node-specific system deps).
4. Put system / optional heavy deps only in the node crate.
5. Optional: add a `just` recipe (see root `justfile`).

## Build / run

```bash
# Core SDK only (default)
cargo test

# Specific tool node (may need system packages)
cargo run -p robot-bus-image-encoder -- --params nodes/image_encoder/config/example.yaml
```

See each node's README or `config/example.yaml` for parameters and system requirements.
