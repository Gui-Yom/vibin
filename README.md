# Vibin

https://www.reddit.com/r/ProgrammerHumor/comments/jtnrlk/everyone_loves_pointers_right/?utm_source=share&utm_medium=web2x&context=3

### Build
```
$ ./gradlew shadowJar # requires vlc to be installed
# copy config.vibin.json next to the built jar and customize it
$ java -jar build/libs/vibin-0.1.0.jar # or simply double-click
```

### Controls
 - The window is freely movable by dragging it.
 - Left-click to play next item
 - Mouse wheel to adjust volume
 - Middle mouse button to close it

### Side notes
Requires vlc to be installed as it uses libvlc under the hood.
