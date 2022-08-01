# Vibin

For when your last 2 brain cells need to vibe together at 4am (blazingly fast).

![cat vibing](cat.gif)

---

## Build & Run

```shell
cargo build --release
target/release/vibin <mp3 or ogg file>
```

## Build & Run (with bundled audio)

```shell
set VIBIN_BUNDLE=<mp3 or ogg file>
cargo build --release --features bundle-audio
target/release/vibin
```

## Controls

- The window is freely movable by dragging it.
- Right-click to play next item
- Mouse wheel to adjust volume
- Middle mouse button to close it

https://www.reddit.com/r/ProgrammerHumor/comments/jtnrlk/everyone_loves_pointers_right/?utm_source=share&utm_medium=web2x&context=3

## A note about build optimization

See [BuildOpt.md](BuildOpt.md).
