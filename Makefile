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

grade-full: cachesim
	./bin/run_grader.py

clean:
	rm -rfv test_results cachesim *-project1.tar.gz target

.PHONY: all submission clean grade grade-full