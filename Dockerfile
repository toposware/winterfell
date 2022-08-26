FROM rust:latest AS base

ARG TOOLCHAIN_VERSION
ARG GITHUB_TOKEN

ENV CARGO_TERM_COLOR=always
ENV RUSTFLAGS=-Dwarnings
ENV RUST_BACKTRACE=1

RUN git config --global url."https://${GITHUB_TOKEN}@github.com/".insteadOf "https://github.com/"

RUN apt update && \
    apt install -y clang

RUN rustup toolchain install ${TOOLCHAIN_VERSION} && \
    rustup default ${TOOLCHAIN_VERSION} && \
    rustup target add wasm32-unknown-unknown

WORKDIR /usr/src/app

FROM base AS build-no-std
COPY . .
RUN cargo build --verbose --no-default-features

FROM base AS test
COPY . .
RUN cargo test

FROM base AS fmt
RUN rustup component add rustfmt
COPY . .
RUN cargo fmt --all -- --check

FROM base AS lint
RUN rustup component add clippy
COPY . .
RUN cargo clippy --all -- -D clippy::all -D warnings
