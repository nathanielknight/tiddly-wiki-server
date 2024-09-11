FROM rust:1.81 AS base
RUN cargo install cargo-chef --version '^0.1'

FROM base AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/tiddly-wiki-server /srv/tiddly-wiki-server
COPY ./empty.html.template /srv/empty.html.template

WORKDIR /srv/
EXPOSE 3032

# Default server parameters
ENV TWS_DBPATH="/srv/data/tiddlers.sqlite3"
ENV TWS_BIND="0.0.0.0"

ENTRYPOINT ["/srv/tiddly-wiki-server"]
