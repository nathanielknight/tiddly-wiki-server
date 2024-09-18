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
# Empty dir to COPY --chown as nonroot
RUN mkdir empty

FROM gcr.io/distroless/cc-debian12
COPY --from=builder --chown=nonroot:nonroot /app/target/release/tiddly-wiki-server /srv/tiddly-wiki-server

USER nonroot
COPY --from=builder --chown=nonroot:nonroot /app/empty /data

# App parameters
WORKDIR /srv/
EXPOSE 3032
ENV TWS_DBPATH="/data/tiddlers.sqlite3"
ENV TWS_BIND="0.0.0.0"

ENTRYPOINT ["/srv/tiddly-wiki-server"]
