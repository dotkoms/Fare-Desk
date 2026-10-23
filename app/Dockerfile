# syntax=docker/dockerfile:1
FROM rust:1-bookworm AS build

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY data ./data
RUN cargo build --release -p faredesk-api

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home faredesk \
    && mkdir -p /data \
    && chown faredesk:faredesk /data

COPY --from=build /src/target/release/faredesk-api /usr/local/bin/faredesk-api

WORKDIR /app
USER faredesk
ENV PORT=8080 \
    DATABASE_PATH=/data/faredesk.sqlite

EXPOSE 8080
VOLUME ["/data"]
CMD ["faredesk-api"]
