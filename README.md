# RP1

[![Current Crates.io Version](https://img.shields.io/crates/v/rp1.svg)](https://crates.io/crates/rp1)
[![Current Docs.rs Version](https://docs.rs/rp1/badge.svg)](https://docs.rs/rp1)

RP1 provides an easy way to quickly create a basic API using [Diesel] and
[Rocket] (v0.5). Using a macro attribute on a struct RP1 generates basic
REST-like API endpoints for [CRUD] operations.

The goal of RP1 is to get a working API layer for your application with as
little effort as possible while also remaining versatile. Don't want to use our
generated code? It should be easy to disable or replace part of our generation
without having to completely discard it. It should also be possible to re-use
some parts of RP1 in your own application, even if you don't use the generated
routes.

Writing an application using RP1 starts by defining your database schema using
diesel. Based on this schema and a model struct (one that would in normal
diesel usage only be intended for querying) RP1 will generate some routes and
handlers that you can directly plug into your rocket application. To get
started yourself, you should start with the crate level documentation in the
generated [docs].

## Feedback and improvement
Have any suggestions or made some (small) improvement? Do let us know!

[Diesel]: https://diesel.rs/
[Rocket]: https://rocket.rs/
[CRUD]: https://en.wikipedia.org/wiki/Create,_read,_update_and_delete
[docs]: https://docs.rs/rp1
