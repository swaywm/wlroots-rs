# Spawning a client

## `WAYLAND_DISPLAY`

Starting a client when the compositor starts up and having it be able to open a
channel to the compositor is very simple. Using the `WAYLAND_DISPLAY`
environment variable a client to know which (if any) Wayland compositor it
should communicate with.

`CompositorBuilder` does not set this environment variable automatically, it
must be done explicitly using the Rust standard library.

## Starting a program properly

In Rust the way a new process can be spawned is using the
[Command](https://doc.rust-lang.org/std/process/struct.Command.html) struct from
the stdlib. Using
[Command::spawn](https://doc.rust-lang.org/std/process/struct.Command.html#method.spawn)
an entirely separate process is spun out that does not depend on the compositor process.

Care must be taken that zombie processes are not created, so all
commands should be wrapped in a shell call.

Finally, the output of these programs probably should not be mixed with the
compositor logs. This can be rectified by setting stdout and stderr of these
programs to `/dev/null`.

Putting it all together, the startup command function looks like this:

```rust
{{#include 4-breaking-out-of-your-shell/main.rs:127:}}
```
