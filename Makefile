.PHONY: build release run dev test test-sequential format coverage install uninstall clean help

# Default target
all: build

## help: Display this help message
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^## [-a-zA-Z0-9_]+:' Makefile | sed -e 's/^## //' | awk 'BEGIN {FS = ": "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'

## build: Build the project in debug mode
build:
	cargo build

## run: Build and run the project locally
run: build
	./target/debug/asum $(ARGS)

## dev: Build and run the project locally (alias for run)
dev: run

## release: Build the project in release mode
release:
	cargo build --release

## test: Run all tests in parallel
test:
	cargo test

## test-sequential: Run all tests sequentially (recommended for isolated environments)
test-sequential:
	cargo test -- --test-threads=1

## format: Run code formatting (cargo fmt) and linting (clippy)
format:
	@chmod +x ./bin/format.sh
	./bin/format.sh

## coverage: Run tests and generate HTML coverage report
coverage:
	@chmod +x ./bin/coverage.sh
	./bin/coverage.sh

## install: Install the pre-compiled binary
install:
	@chmod +x ./bin/install.sh
	./bin/install.sh

## uninstall: Uninstall the binary
uninstall:
	@chmod +x ./bin/uninstall.sh
	./bin/uninstall.sh

## clean: Clean build artifacts and coverage reports
clean:
	cargo clean
	rm -rf ./coverage
