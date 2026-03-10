# Stage 1: Build rambler from source
FROM golang:1.26-trixie AS rambler-builder
RUN git clone https://github.com/elwinar/rambler.git /build
WORKDIR /build
RUN go build -o /rambler .

# Stage 2: Build rumbler from source
FROM rust:1.94-trixie AS rumbler-builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

# Stage 3: Test runner
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    jq postgresql-client ca-certificates parallel libncurses5-dev libncursesw5-dev \
    && rm -rf /var/lib/apt/lists/*

COPY acceptance_tests/lib/bats-core /tmp/bats-core
RUN /tmp/bats-core/install.sh /usr/local && rm -rf /tmp/bats-core

COPY --from=rambler-builder /rambler /usr/local/bin/rambler
COPY --from=rumbler-builder /build/target/release/rumbler /usr/local/bin/rumbler

WORKDIR /tests

COPY acceptance_tests/ ./
ENV BATS_LIB_PATH="/usr/lib/bats:/tests/lib"

ENTRYPOINT ["bats", "--timing", "--formatter", "pretty"]
