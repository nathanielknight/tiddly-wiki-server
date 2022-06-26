# Tiddlywiki Server

This is a web backend for [TiddlyWiki]. It uses TiddlyWiki's [web server
API] to save tiddlers in a [SQLite database]. It should come  with a
slightly altered empty TiddlyWiki that includes an extra tiddler store (for
saved tiddlers) and  the `$:/plugins/tiddlywiki/tiddlyweb` plugin (which is
necessary to make use of the web backend).

[TiddlyWiki]: https://tiddlywiki.com/
[web server API]: https://tiddlywiki.com/#WebServer
[SQLite]: https://sqlite.org/index.html


## Motivation

TiddlyWiki 5 has a [NodeJS based webserver] that re-uses much of the front-end
JavaScript for maximum compatibility. However, this server needs about 70 MB of
memory to start, and can easily consume 100 MB or more. This is fine for running
on a workstation, but a cheap VPS quickly gets crowded running services of this
size.

[NodeJS based webserver]: https://tiddlywiki.com/static/WebServer.html

In rudimetnary benchmarks it looks like `tiddly-wiki-server` uses about 10 MB of
memory (with no optimizations), which I find much more manageable.


## Setup

To create a Tiddlywiki backed by this server:

1. Build or install the executable on your server (e.g. by checking out the repo
   and running `cargo install --path .`).
1. Set up the directory you want to run the server in:
  a. Copy the `empty.html.template` file into the directory.
  b. Create a `files/` folder to hold [static files].
1. Run `tiddly-wiki-server`.


## Contributing

The most valuable way to contribute to this project is currently testing: try to
setup a Tiddlywiki with it and see if it behaves the way you'd expect. The
server aims to have feature parity with the first-party NodeJS backend; any
discrepancy is a potential bug, which I'd be very grateful to have reported!
