# Image de dev : toolchain Rust complète, aucune installation locale requise.
FROM rust:1.99.1-alpine@sha256:7e9e2d068a107a2adafd19930011178d14a78326053cd4c3a9f7ad3fd049557d
RUN apk add --no-cache musl-dev && rustup component add clippy rustfmt
ENV CARGO_TARGET_DIR=/target
WORKDIR /src
