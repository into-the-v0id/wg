FROM node:22-alpine AS assets

WORKDIR /app/wg-web

COPY wg-web/package.json .
COPY wg-web/package-lock.json .
COPY wg-web/assets/ ./assets/
COPY wg-web/static/ ./static/

RUN --mount=type=cache,target=/app/wg-web/node_modules npm install && npm run build:sass

FROM rustlang/rust:nightly-slim AS build

WORKDIR /app

COPY . .

# Install assets
COPY --from=assets /app/wg-web/static /app/wg-web/static

# Build app
RUN --mount=type=cache,target=/app/target --mount=type=cache,target=/root/.cargo cd wg/ && cargo build --release && cp /app/target/release/wg /app/wg

FROM debian:stable-slim

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN mkdir /data
ENV DB_FILE=/data/sqlite.db

COPY --from=build /app/wg /app/wg

CMD ["/app/wg"]
