SRCFILES := $(wildcard src/*.rs)
CARGO := $(HOME)/.cargo/bin/cargo

$(CARGO):
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

all: cachesim

cachesim: $(SRCFILES) Cargo.toml
	$(CARGO) build --release
	cp target/release/cache_sim ./cachesim

submission: cachesim
	./bin/makesubmission.sh

grade: cachesim
	./bin/run_grader.py --fast

grade-full: cachesim
	./bin/run_grader.py

clean:
	rm -rfv test_results cachesim *-project1.tar.gz target

test:
	cargo build && ./target/debug/cache_sim LRU 65536 1024 64 < inputs/trace1

.PHONY: all submission clean grade grade-full