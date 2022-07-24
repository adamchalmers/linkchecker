cargo build -q --release && hyperfine --warmup 1 -- ./run.sh
