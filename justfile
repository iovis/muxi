# set dotenv-load  # Uncomment to load .env

completions_dir := env_var('FDOTDIR') / "completions/muxi.fish"

# list recipes
default:
  @just --list

# Run tests with nextest
alias t := test
@test:
  cargo nextest run

# Install locally
alias i := install
@install:
  cargo install --path .

# Publish to creates.io
alias p := publish
@publish: test
  cargo publish

# Generate and install completions
completions:
  cargo run -q -- completions fish > {{ completions_dir }}

# Compile and open docs for muxi and its dependencies
alias d := docs
@docs:
  cargo doc --open

# Open an evcxr console
alias c := console
@console:
    evcxr
