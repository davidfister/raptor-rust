.PHONY: build test benchmark install-ghz

build:
	cargo build --release --locked

test:
	cargo test --locked

benchmark:
	./scripts/benchmark.sh

install-ghz:
	mkdir -p .tools
	GOBIN="$(CURDIR)/.tools" go install github.com/bojand/ghz/cmd/ghz@latest