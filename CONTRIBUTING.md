# Contributing

Build prerequisites, commands and day-to-day development live in
[README.md](README.md) — start there.

## Vendored `muda` patch (sync burden)

`src-tauri/vendor/muda` is a vendored copy of `muda 0.19.3` with a one-line
local patch (`platform_impl/macos/icon.rs`: mark custom menu-item icons as
template images so macOS tints them per menu appearance), wired via
`[patch.crates-io]` in `src-tauri/Cargo.toml`.

When upgrading `muda` (or Tauri, which pulls it in transitively):

1. Copy the new upstream source over `src-tauri/vendor/muda`.
2. Re-apply the template-image patch (see `Cargo.toml` comment for the file).
3. `cargo test`, rebuild, and visually check the tray menu in both light and
   dark appearance — the patch is appearance-sensitive and has no automated
   test.
4. Commit the refreshed vendor tree together with the `Cargo.lock` bump.
