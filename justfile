test-js:
    concurrently ./test1.sh ./test2.sh

test-js-kill:
    concurrently --kill-others "./test1.sh" "./test2.sh"

test:
    cargo r -- "./test1.sh" "./test2.sh"
test-kill:
    cargo r -- -k "./test1.sh" "./test2.sh"

ci: 
    cargo fmt --all -- --check
    cargo c
    cargo t
    cargo clippy
