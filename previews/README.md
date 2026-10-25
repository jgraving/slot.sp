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

The showcase uses Emerald, Ruby and Sapphire in translucent green, red and blue shells. Metroid Fusion and WarioWare retain their standard dark shells. The fixture writes the same `cart_shell.txt` overrides used by the application.

The opening hero uses the production App insertion/ejection clock, SlotChrome geometry and compositor with a single Emerald cartridge. The background uses the production fluid shader with an eased two-second excursion and return so this short hero loops cleanly. It adds a wordmark rendered from the site's Google Sans Flex font and a restrained fixed lighting overlay. It is an art-directed product view, not AI-generated video or a multiplayer/gameplay recording. `make_hero_art.py` and `insertion_hero.rs` reproduce it.

To reproduce the hero after preparing the fixture above (requires FontTools and Brotli):

```sh
cp previews/insertion_hero.rs crates/slot/examples/insertion_hero.rs
cargo build -p slot --no-default-features --example insertion_hero
python3 previews/make_hero_art.py /tmp/slot-readme/out
target/debug/examples/insertion_hero /tmp/slot-readme/card /tmp/slot-readme/out
mkdir -p /tmp/slot-readme/hero
ffmpeg -v error -y -i /tmp/slot-readme/out/hero-insertion.mp4 -vf fps=20 /tmp/slot-readme/hero/%04d.png
python3 tools/opaque_preview_gif.py /tmp/slot-readme/hero previews/hero-insertion.gif
rm crates/slot/examples/insertion_hero.rs
```

The glass reveal is shared by the hero and device boot overlay through `slot::brand_motion`. `tools/build-brand-motion.py` bakes the real Google Sans Flex `opsz`, `wdth`, `wght`, `ROND` and `slnt` axes with damped letter motion, bevel lighting and a warm-white terminal frame into `assets/brand-motion.png`. Regenerate it with `python3 tools/build-brand-motion.py assets/brand-motion.png` (Pillow, NumPy, SciPy, FontTools and Brotli). The font source is the same site-hosted Google Sans Flex file used above; the font itself is not embedded in the application.

`boot_brand.rs` records the exact boot-overlay draw path against a wallpaper preview backdrop, rather than a recording of the whole firmware startup. The device overlay lasts at most 1.1 seconds while normal app updates continue, dismisses on a button press, skips automatic game resume, and is disabled with Static Gradient. The earlier firmware screen uses the still BMP. Its atlas GPU storage is released after the overlay ends.

The six-second hero starts insertion at 0.8 seconds and queues the glass mark after the cart clears the centre. Its 1.5-second font-space path visits fine/wide, dense/narrow and soft/rounded poses, then locks to the final brand mark. The mark clears before ejection. The poster PNG is captured during the settled-logo hold. A native assertion guards the cart/logo handoff, and the atlas regression checks that glass-phase interiors remain translucent.
