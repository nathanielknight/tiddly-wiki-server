FROM rust:1.77 AS builder
WORKDIR /usr/src/
RUN cargo new tiddly-wiki-server
WORKDIR /usr/src/tiddly-wiki-server
# Dummy build (for cacheinng deps)
COPY Cargo.toml Cargo.lock .
RUN cargo fetch
RUN cargo build --release
# Project build
COPY . .
RUN cargo install --path . --root /usr/local/cargo

FROM debian:bookworm-slim
COPY --from=builder /usr/local/cargo/bin/tiddly-wiki-server /srv/tiddly-wiki-server
COPY ./empty.html.template /srv/empty.html.template

WORKDIR /srv/
EXPOSE 3032
CMD ["/srv/tiddly-wiki-server"]
