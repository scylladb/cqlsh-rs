# Learnings — sp21a-uds-proxy

## `is_socket()` requires explicit trait import on stable Rust
`FileType::is_socket()` lives in `std::os::unix::fs::FileTypeExt`. Even inside
`#[cfg(unix)]` code the trait must be brought into scope with
`use std::os::unix::fs::FileTypeExt;` — otherwise the compiler cannot find the
method despite the impl being present.

## Moving a String into an `async move` closure and using it after
If a `String` is moved into a `tokio::spawn(async move { … })` closure and
cloned inside the loop, the variable is no longer available after the
`tokio::spawn` call. Clone before spawning when you need the value for logging
or further use:
```rust
let path_for_log = socket_path.clone();
let join_handle = tokio::spawn(async move { /* uses socket_path */ });
tracing::debug!("{path_for_log}");
```

## Dropping a JoinHandle does NOT abort the spawned task
`drop(join_handle)` only drops the handle — the task continues running.
To cancel on drop, store `join_handle.abort_handle()` and call
`.abort()` in the `Drop` impl.

## `std::os::unix::net::UnixListener::incoming()` blocks the thread
Use the blocking std `UnixListener` in a `std::thread::spawn` for a simple
echo server in tests.  Tokio's `UnixListener` requires `#[tokio::test]` context
and is harder to use as a bare test helper.
