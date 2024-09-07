FROM rust:1.81 AS base
RUN cargo install sccache --version '^0.7' && \
    cargo install cargo-chef --version '^0.1'
ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

FROM base AS planner
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef prepare --recipe-path recipe.json

FROM base AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/tiddly-wiki-server /srv/tiddly-wiki-server
COPY ./empty.html.template /srv/empty.html.template

WORKDIR /srv/
EXPOSE 3032
CMD ["/srv/tiddly-wiki-server"]
