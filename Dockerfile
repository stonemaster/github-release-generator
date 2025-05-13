FROM ubuntu:24.04 AS builder

##
## Builder part
##

RUN apt-get update && \
  apt-get install -y libssl-dev pkg-config build-essential curl

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.85.1 -y

WORKDIR /app

COPY Cargo.toml ./
COPY Cargo.lock ./
COPY src ./src

RUN bash -c 'ls -lah; source ~/.cargo/env; cargo test && cargo build --release'

##
## Deployment part
##

FROM ubuntu:24.04

RUN apt-get update && \
  apt-get install -y libssl-dev pkg-config ca-certificates git && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/github-release-generator ./github-release-generator

RUN git config --global --add safe.directory '/data'

ENTRYPOINT ["/app/github-release-generator"]
CMD ["--help"]
