# set dotenv-load  # Uncomment to load .env

alias i := install
alias p := publish
alias t := test

completions_dir := env_var('ZDOTDIR') / "completions/_muxi"

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
  cargo run -q -- completions zsh > {{ completions_dir }}
