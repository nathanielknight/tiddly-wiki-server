# TiddlyWiki Server

[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](code_of_conduct.md) 
[![Matrix](https://img.shields.io/matrix/tws:conduit.nathanielknight.ca)](https://matrix.to/#/#tws:conduit.nathanielknight.ca)
[![Join the chat at https://gitter.im/tiddly-wiki-server/community](https://badges.gitter.im/tiddly-wiki-server/community.svg)](https://gitter.im/tiddly-wiki-server/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

This is a web server for [TiddlyWiki]. It uses TiddlyWiki's [web server
API] to save tiddlers in a [SQLite database]. It should come  with a
slightly altered empty TiddlyWiki that includes an extra tiddler store (for
saved tiddlers) and  the [TiddlyWeb plugin] (which is necessary to make use of
the web server API).

[TiddlyWiki]: https://tiddlywiki.com/
[web server API]: https://tiddlywiki.com/#WebServer
[SQLite]: https://sqlite.org/index.html
[TiddlyWeb plugin]: https://github.com/Jermolene/TiddlyWiki5/tree/master/plugins/tiddlywiki/tiddlyweb

## Motivation

TiddlyWiki 5 has a [NodeJS based web server] that re-uses much of the front-end
JavaScript for maximum compatibility. However, this server needs about 70 MB of
memory to start, and can easily consume 100 MB or more. This is fine for running
on a workstation, but a cheap VPS quickly gets crowded running services of this
size.

[NodeJS based webserver]: https://tiddlywiki.com/static/WebServer.html

In rudimentary benchmarks it looks like `tiddly-wiki-server` uses about 10 MB of
memory (with no optimizations), which I find much more manageable.


## Setup

To create a TiddlyWiki backed by this server:

1. Build or install the executable on your server (e.g. by checking out this
   repository and running `cargo install --path .`).
1. Set up the directory you want to run the server in: a. Copy the
   `empty.html.template` file into the directory.  b. Create a `files/` folder
   to hold [static files].
1. Run `tiddly-wiki-server`.


## Differences from TiddlyWiki

The initial page that this project serves has a few changes compared to the
empty wiki you can download from tiddlywiki.com/empty.html. It has:

* the [TiddlyWeb plugin] to let TiddlyWiki save data to the server, and
* any data that you entered or imported.
* no `noscript` section for browsers that disable JavaScript (this is
  considered a bug)

It was created by following this procedure:

1. Download an empty TiddlyWiki from tiddlywiki.com/empty.html
1. Add the TiddlyWeb plugin via the [plugin library]
1. Add a `script` element to the very end of the HTML document with
  - `class="tiddlywiki-tiddler-store"`
  - `type="application/json`
  - The contents `@@TIDDLY-WIKI-SERVER-EXTRA-TIDDLERS-@@N41yzvgnloEcoiY0so8e2dlri4cbYopzw7D5K4XRO9I@@`

[plugin library]: https://tiddlywiki.com/static/Installing%2520a%2520plugin%2520from%2520the%2520plugin%2520library.html

The server replaces the contents of the `script` tag with the saved tiddlers.
Since tiddlers can contain escaped (sometimes twice-escaped) code in various
programming and/or markup languages, creating a separate tiddler store is much
easier than dynamically modifying the core TiddlyWiki tiddlers.


## Contributing

The most valuable way to contribute to this project is currently testing: try to
setup a TiddlyWiki with it and see if it behaves the way you'd expect. The
server aims to have feature parity with the first-party NodeJS server; any
discrepancy is a potential bug, which I'd be very grateful to have reported!


## Code of Conduct

Contributors are expected to abide by the [Contributor Covenant](https://www.contributor-covenant.org/).
