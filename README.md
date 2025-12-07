# Function Programming Server

## How to

### Dev shell

```
nix develop
# inside:
cargo run
```

`RUST_LOG=debug cargo run`

### Run (with cargo, using your checkout)

```
nix run
# or
nix run .#
```

### Build

```
nix build          # builds .#default
ls result/bin
# → your built binary (name from Cargo.toml)
```
