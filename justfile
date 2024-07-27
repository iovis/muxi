completions_dir := env_var("FDOTDIR") / "completions/muxi.fish"

# list recipes
default:
    @just --list

# Run tests with nextest
@test:
    cargo nextest run

# Install locally
@install:
    cargo install --path .

# Publish to creates.io
@publish: test
    cargo audit
    git push
    git push --tags  # cargo-dist
    cargo publish

# Run cargo-dist
dist:
    cargo dist init

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
@console:
    evcxr

# Profile with samply
profile args:
    cargo build --profile profiling
    samply record target/profiling/muxi {{ args }}
