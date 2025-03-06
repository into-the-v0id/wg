FROM rustlang/rust:nightly-slim AS build

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Build app
COPY . .
RUN --mount=type=cache,target=/app/target --mount=type=cache,target=/root/.cargo cargo build --release && cp /app/target/release/wg /app/wg

FROM debian:stable-slim

WORKDIR /app
COPY --from=build /app/wg /app/wg

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN mkdir /data
ENV DB_FILE=/data/sqlite.db

CMD ["/app/wg"]
