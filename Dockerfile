FROM rustlang/rust:nightly-slim AS build

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Build cargo dependencies
RUN cargo init --bin .
COPY Cargo.toml .
COPY Cargo.lock .
RUN --mount=type=cache,target=/app/target --mount=type=cache,target=/root/.cargo cargo build --release

# Build app
COPY . .
RUN --mount=type=cache,target=/app/target --mount=type=cache,target=/root/.cargo cargo build --release && cp /app/target/release/wg /app/wg

FROM debian:stable-slim

RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=build /app/wg /app/wg

RUN mkdir /data
ENV DB_FILE=/data/sqlite.db

CMD ["/app/wg"]
