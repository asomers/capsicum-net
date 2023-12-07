#! /bin/sh -e
#
# Generate a code coverage report
#
# Requirements:
# sudo pkg install grcov
# cargo install grcov
# rustup component add llvm-tools-preview
#
# Usage:
# coverage.sh

export LLVM_PROFILE_FILE="capsicum-net-%p-%m.profraw"
export RUSTFLAGS="-Cinstrument-coverage"
TOOLCHAIN=nightly
cargo +$TOOLCHAIN build --all-features
cargo +$TOOLCHAIN test --all-features

grcov . --binary-path $PWD/target/debug -s src -t html --branch \
	--ignore-not-existing \
	-o ./coverage/
