# slot.sp website

Published at **https://jakegraving.com/slot.sp/** through GitHub Pages.

## Build

Use Python 3.13, FFmpeg, Rust 1.96 with the `wasm32-unknown-unknown` target, and wasm-pack 0.15.0. From the repository root:

```sh
python -m venv .venv-site
.venv-site/bin/pip install zensical==0.0.71 'fonttools[woff]==4.61.1' pillow==13.0.0
mkdir -p _studio
git clone https://github.com/jgraving/slot-cart-studio.git _studio/slot-cart-studio
git -C _studio/slot-cart-studio checkout "$(python -c 'import json; print(json.load(open("site/studio.json"))["ref"])')"
git clone https://github.com/BrandonKowalski/slot.git _studio/slot
git -C _studio/slot checkout "$(cat _studio/slot-cart-studio/slot.ref)"
(cd _studio/slot-cart-studio && wasm-pack build --release --target web --no-typescript --out-dir web/pkg)
.venv-site/bin/python tools/prepare-site.py
.venv-site/bin/zensical build --clean --strict -f site/zensical.toml
.venv-site/bin/python tools/assemble-studio.py
.venv-site/bin/python tools/check-site.py
```

The output is `site/public/`. The preparation step:

- Turns the root feature guides into website pages, adapting relative links and heading anchors.
- Converts the fork's existing README animations to small H.264 previews with WebP posters.
- Builds a self-hosted Google Sans Flex WOFF2 from the source pinned in the UI font manifest, verifying its SHA-256 first. Its license is included with the website font.

`site/studio.json` pins the revision of `jgraving/slot-cart-studio` published under `/slot.sp/studio/`. The assembly step checks that source revision and its renderer pin, includes the app license, and records both in `studio/build-info.json`. Update the pin after merging a tested change in the Studio repository.

Edit the root Markdown guides to update feature instructions. Generated pages, previews and fonts are ignored by Git. The homepage and shared layout live in `site/overrides/`; the fork's colors and typography live in `site/docs/assets/fork.css`.

## Publish

The `pages` workflow builds and checks pull requests. Merging a website or guide change into `main` rebuilds and deploys the site automatically. It can also run manually from Actions.

Repository **Settings → Pages → Source** must be **GitHub Actions**. No custom domain or deployment token is required. The inherited upstream `CNAME` is deliberately absent.

Checks cover the strict documentation build, local page links and anchors, CSS assets, project subpath, self-hosted font references playable 720×480 H.264 previews and the pinned Studio WebAssembly package.
