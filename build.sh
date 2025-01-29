#!/bin/zsh
LAMBDA_ARCH="linux/arm64"
RUST_TARGET="aarch64-unknown-linux-gnu"
RUST_VERSION="latest"

BIN="lambda"

docker run --platform ${LAMBDA_ARCH} \
    --rm --user "$(id -u)":"$(id -g)" \
    -v "${PWD}":/usr/src/app -w /usr/src/app rust:${RUST_VERSION} \
  	cargo build --release --bin ${BIN} --target ${RUST_TARGET}

cp ./target/${RUST_TARGET}/release/${BIN} ./bootstrap && zip ${BIN}.zip bootstrap && rm bootstrap
