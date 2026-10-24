# README animation captures

These previews use the current production frontend, artwork, animation timing and OpenGL compositor at 720 × 480. Synthetic 256-byte ROM headers create browseable cartridges with original cartridge-label artwork; they are not playable games and contain no commercial ROM data.

`capture.rs` adapts the existing `record` example to a surfaceless Mesa context on Linux. `egl_surface.rs` supplies the context without changing application code. Carousel, core-picker and shutdown clips run the actual frontend with scripted inputs. `link_capture.rs` calls the production cable renderer with a deterministic pick → working → linked → unplug sequence. It demonstrates interface artwork, not network connectivity or multiplayer gameplay.

## Reproduce

Install the repository's Rust toolchain, Mesa OpenGL/EGL, FFmpeg and Python with Pillow. Run from the repository root:

```sh
mkdir -p /tmp/slot-readme/out
cp previews/capture.rs crates/slot/examples/readme_capture.rs
cp previews/egl_surface.rs crates/slot/examples/egl_surface.rs
cp previews/link_capture.rs crates/slot/examples/link_capture.rs
cp previews/fixture.rs crates/slot/examples/readme_fixture.rs
cargo build -p slot --no-default-features --example readme_capture --example readme_fixture
target/debug/examples/readme_fixture /tmp/slot-readme/card
python3 previews/prepare_art.py /tmp/slot-readme/card
LIBGL_ALWAYS_SOFTWARE=1 target/debug/examples/readme_capture /tmp/slot-readme/card previews/readme.script /tmp/slot-readme/out
for clip in carousel core-picker link shutdown; do
  mkdir -p "/tmp/slot-readme/$clip"
  ffmpeg -v error -y -i "/tmp/slot-readme/out/$clip.mp4" -vf fps=20 "/tmp/slot-readme/$clip/%04d.png"
  python3 tools/opaque_preview_gif.py "/tmp/slot-readme/$clip" "previews/readme-$clip.gif"
done
rm crates/slot/examples/readme_capture.rs crates/slot/examples/egl_surface.rs crates/slot/examples/link_capture.rs crates/slot/examples/readme_fixture.rs
```

GIFs use a shared palette, opaque frames and 50 ms frame intervals. The carousel returns to its starting cartridge, the picker closes before restarting, and the cable returns to its initial pose. Shutdown retains the actual switch/fade timing, followed by a black pause before the next demonstration. Inspect generated clips before replacing the README assets.

Label scans: LaunchBox Games Database and its contributors; original game artwork belongs to its publishers. Exact source pages and image URLs are listed in `art-sources.json`. Used only for the interface previews.
