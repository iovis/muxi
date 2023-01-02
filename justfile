default:
  just --list

test:
	cargo nextest run

install:
	cargo install --path .

publish: test
	cargo publish

completions:
	cargo run -q -- completions zsh > "$ZDOTDIR/completions/_muxi"
