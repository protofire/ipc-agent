# syntax=docker/dockerfile:1

# Build stage
FROM rust:1.68 as builder

RUN apt-get update && \
    apt-get install -y build-essential && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

RUN --mount=type=cache,target=$RUSTUP_HOME,from=rust,source=$RUSTUP_HOME \
    --mount=type=cache,target=$CARGO_HOME,from=rust,source=$CARGO_HOME \
    --mount=type=cache,target=target \
    cargo install --root output --path .


# Main stage
FROM debian:bullseye-slim

COPY --from=builder /app/output/bin/ipc-agent /usr/local/bin/ipc-agent

ENTRYPOINT ["ipc-agent", "daemon"]

EXPOSE 3030
