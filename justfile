# set dotenv-load  # Uncomment to load .env

alias d := docs
alias i := install
alias p := publish
alias t := test

completions_dir := env_var('FDOTDIR') / "completions/muxi.fish"

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
  cargo publish

# Generate and install completions
completions:
  cargo run -q -- completions fish > {{ completions_dir }}

# Compile and open docs for muxi and its dependencies
@docs:
  cargo doc --open
