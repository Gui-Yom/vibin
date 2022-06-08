## Build optimization

The totally unscientific build benchmark.

### Debug

| opts                                | full (secs) | incremental (secs) | gif load (ms) | audio load (ms) | Comment               |
|-------------------------------------|-------------|--------------------|---------------|-----------------|-----------------------|
| opt-level=0                         | 41          | 3-                 | 3100          | 24000           | No way I can use this |
| opt-level=1                         | 77          | 3+                 | 340           | 780             | Yay                   |
| opt-level=2                         | 85          | 3++                | 230           | 500             | Slow                  |
| opt-level=0 (all deps opt-level=2)  | 122         | 3++                | 730           | 700             | Slow                  |
| opt-level=0 (some deps opt-level=2) | 43          | 3                  | 1050          | 1000            | Yay                   |

### Release build

| opts                       | build (secs) | binary size (Ko) | gif load (ms) | audio load (ms) |
|----------------------------|--------------|------------------|---------------|-----------------|
| opt-level=3 lto=thin cu=4  | 59           | 4934             | 240           | 450             |
| opt-level=3 lto=thin cu=8  | 58           | 4887             | 240           | 470             |
| opt-level=3 lto=thin cu=16 | 60           | 4893             | 300           | 470             |
| opt-level=3 lto=thin cu=32 | 63           | 4895             | 240           | 470             |
| opt-level=3 lto=no cu=8    | 62           | 4842             | 270           | 500             |

### Notes

`gif load` measure is pretty stable because the gif is included in the binary at compilation, so we don't measure a
filesystem load.

Build time measure is probably the worst of all since it spans a lot of cpu time and involves a lot of filesystem
operations. It can vary by +-3secs, probably because I build on an HDD.

Build times in release mode are weird, isn't the `codegen-units` option supposed to reduce build times ? it actually
seems to make them longer.
