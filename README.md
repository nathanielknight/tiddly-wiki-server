# TiddlyWiki Server

[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](code_of_conduct.md) 

This is an efficient, low-maintenance web server for [TiddlyWiki]. It uses the [web server
API] provided by the [TiddlyWeb plugin] to save tiddlers in a [SQLite database]. It's written in Rust.

[TiddlyWiki]: https://tiddlywiki.com/
[web server API]: https://tiddlywiki.com/#WebServer
[SQLite database]: https://sqlite.org/index.html
[TiddlyWeb plugin]: https://github.com/Jermolene/TiddlyWiki5/tree/master/plugins/tiddlywiki/tiddlyweb

## Running the Server

The easiest way to run `tiddly-wiki-server` is with Docker Compose. You can grab
the [compose file](./docker-compose.yml) from this project and then start a
server with 

```sh
docker compose up
```

To run the server directly,

1. Build or install the executable (e.g. by checking out this repository and
   running `cargo install --path .`).
1. Run the server:
   ```sh
   tiddly-wiki-server --bind 0.0.0.0 --port 3032 --dbpath ./tiddlers.sqlite
   ```

If the database doesn't exit, `tiddly-wiki-server` will create and initialize
it.

See the `tiddly-wiki-server --help` for instructions on changing the bound
address, port, database path, etc.

## Motivation

TiddlyWiki 5 has a [NodeJS based web server] that re-uses much of the front-end
JavaScript for maximum compatibility. However, this server needs about 70 MB of
memory to start, and can easily consume 100 MB or more. This is fine for running
on a workstation, but a cheap VPS quickly gets crowded running services that size.

[NodeJS based web server]: https://tiddlywiki.com/static/WebServer.html

In rudimentary benchmarks it looks like `tiddly-wiki-server` uses about 10 MB of
memory, which I find much more manageable. It's also easier to deploy!


## License

This project is made available under [The Prosperity Public License 3.0.0],
which gives you broad permissions for:

* personal and not-for-profit use
* reading and modifying the source code
* trying out the software commercially

but _doesn't_ let you build a business on the author's work.

If you're uncertain if your use case might be infringing or you want to use it
under a different license, reach out to @natknight.


[The Prosperity Public License 3.0.0]: https://prosperitylicense.com/versions/3.0.0

## Differences from TiddlyWiki

The initial page that this project serves has a few changes compared to the
empty wiki you can download from http://tiddlywiki.com/empty.html. It has:

* the [TiddlyWeb plugin] to let TiddlyWiki save data to the server, and
* any data that you entered or imported.
* no `noscript` section for browsers that disable JavaScript (this is
  considered a bug)

This modified wiki was created by following this procedure:

1. Download an empty TiddlyWiki from tiddlywiki.com/empty.html
1. Add the TiddlyWeb plugin via the [plugin library]
1. Add a `script` element to the very end of the HTML document with
  - `class="tiddlywiki-tiddler-store"`
  - `type="application/json`

[plugin library]: https://tiddlywiki.com/static/Installing%2520a%2520plugin%2520from%2520the%2520plugin%2520library.html

The empty wiki is then embedded in the `tiddly-wiki-server` binary; when the
page is loaded, it inserts the stored tiddlers into the empty wiki and serves
it. The result isn't _exactly_ what you'd get by loading the content into a
regular TiddlyWiki and saving it, but it has all the same features, including
that you can always download a copy and have a full, working TiddlyWiki with all
of your tiddlers.

## Contributing

The most valuable way to contribute to this project is currently testing: try to
setup a TiddlyWiki with it and see if it behaves the way you'd expect. The
server aims to have feature parity with the first-party NodeJS server; any
discrepancy is a potential bug, which I'd be very grateful to have reported!

## Code of Conduct

Contributors are expected to abide by the [Contributor Covenant](https://www.contributor-covenant.org/).
