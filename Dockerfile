FROM node:22-alpine AS assets

WORKDIR /app/wg-web

COPY wg-web/package.json .
COPY wg-web/package-lock.json .
COPY wg-web/assets/ ./assets/
COPY wg-web/static/ ./static/

RUN --mount=type=cache,target=/app/wg-web/node_modules npm install && npm run build:sass

FROM rust:1-slim AS build

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

# Install assets
COPY --from=assets /app/wg-web/static /app/wg-web/static

# Build app
ARG CARGO_BUILD_ARGS="--release"
RUN --mount=type=cache,target=/app/target --mount=type=cache,target=/root/.cargo cd wg/ && cargo build $CARGO_BUILD_ARGS && cp /app/target/release/wg /app/wg/wg

FROM debian:stable-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y curl libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN mkdir /data
ENV DB_FILE=/data/sqlite.db

COPY --from=build /app/wg/wg /app/wg

CMD ["/app/wg"]
