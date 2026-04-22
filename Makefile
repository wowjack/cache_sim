CARGO := $(HOME)/.cargo/bin/cargo
SRCFILES := $(wildcard src/*.rs)

all: cachesim

$(CARGO):
	wget -qO- https://sh.rustup.rs | sh -s -- -y

cachesim: $(CARGO) $(SRCFILES) Cargo.toml
	$(CARGO) build --release
	cp target/release/cache_sim ./cachesim

submission: cachesim
	./bin/makesubmission.sh

grade: cachesim
	./bin/run_grader.py --fast

test: cachesim
	./cachesim LRU 1024 128 2 CUSTOM 1 < ./inputs/trace5

clean:
	rm -rfv test_results cachesim *-project2.tar.gz target

.PHONY: all submission clean grade grade-full