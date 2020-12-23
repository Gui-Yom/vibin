# Vibin
https://www.reddit.com/r/ProgrammerHumor/comments/jtnrlk/everyone_loves_pointers_right/?utm_source=share&utm_medium=web2x&context=3

### Build
The build has only been tested on windows atm. On Windows, we use prebuilt binaries for libmpv.
They can be found here : https://sourceforge.net/projects/mpv-player-windows/files/libmpv/.
To generate a .lib from a dll : https://stackoverflow.com/questions/9946322/how-to-generate-an-import-library-lib-file-from-a-dll
```
# Decompress the prebuilt binaries in a known place (I use prebuilt/mpv/64) and set MPV_SOURCE to it.
# The build should find them in the MPV_SOURCE/<arch> directory.
$ set MPV_SOURCE=prebuilt/mpv
# Other problem ! The prebuilt binaries you just downloaded don't contain mpv.lib file for the linker.
# I used msvc lib tool to generate it from the dll and the def.
$ lib /def:mpv.def /out:mpv.lib /machine:x64
$ cargo build --release
# the binary is found at ./target/release/vibin.exe
```
You can't just run it because cargo has not copied the mpv dll next to our artifact. Just copy mpv.dll next to it.

### Supported formats
 - About everything ffmpeg supports. (subject to change because ffmpeg is really big)

### Controls
 - The window is freely movable by dragging it.
 - Left-click to play next item
 - Mouse wheel to adjust volume
 - Middle mouse button to close it
