default:
  just --list

test:
	cargo nextest run

install:
	cargo install --path .

publish: test
	cargo publish
