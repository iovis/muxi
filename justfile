bin := file_name(justfile_directory())
completions_dir := env("FDOTDIR") / "completions/muxi.fish"

# list recipes
default:
    @just --list

# Run tests with nextest
@test:
    cargo nextest run

# Install locally
@install:
    cargo install --path .

# Publish to crates.io
@publish: test
    cargo audit
    git push
    git push --tags  # cargo-dist
    cargo publish

# Run cargo-dist
dist:
    dist init

# Generate and install completions
completions:
    cargo run -q -- completions fish > {{ completions_dir }}

# Compile and open docs for muxi and its dependencies
@docs:
    cargo doc --open

# Open project in Github
open:
    gh repo view --web

# Open an evcxr console
console:
    # Use `:dep .` to load current crate
    @evcxr

# Upgrade dependencies
upgrade:
    cargo upgrade --incompatible allow

# Profile with samply
profile *args:
    cargo build --profile profiling
    samply record target/profiling/{{ bin }} {{ args }}

# Debug with rust-lldb
debug *args:
    cargo build
    rust-lldb -Q -- target/debug/{{ bin }} {{ args }}
