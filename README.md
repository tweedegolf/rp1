# RP1
RP1 provides an easy way to quickly create a basic API using [Diesel] and
[Rocket] (v0.5). Using a simple attribute on a struct RP1 generates simple
REST-like API endpoints for [CRUD] operations.

In the world of rockets, RP-1 (Rocket Propellant-1) is the rocket fuel of
choice for a large number of first stages and first stage rocket boosters. But
for our use-case it could just as easily mean Rapid Prototype-1. The goal of
RP1 is to get a working API layer for your application with as little effort
as possible while also remaining versatile. Don't want to use our generated
code? It should be easy to disable or replace part of our generation without
having to completely discard it. It should also be possible to re-use some
parts of RP1 in your own application, even if you don't use the generated
routes.

## Feedback and improvement
Have any suggestions or made some (small) improvement? Do let us know!

[Diesel]: https://diesel.rs/
[Rocket]: https://rocket.rs/
[CRUD]: https://en.wikipedia.org/wiki/Create,_read,_update_and_delete
