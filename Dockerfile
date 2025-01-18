FROM rustlang/rust:nightly-slim AS build

WORKDIR /app
COPY . .

RUN --mount=type=cache,target=/app/target cargo build --release && cp /app/target/release/wg /app/wg

FROM alpine:latest

WORKDIR /app
COPY --from=build /app/wg /app/wg

CMD ["/app/wg"]
