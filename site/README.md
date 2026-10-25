# slot.sp website

Published at **https://jgraving.github.io/slot.sp/** through GitHub Pages.

## Build

Use Python 3.13 and install FFmpeg, then:

```sh
python -m venv .venv-site
.venv-site/bin/pip install zensical==0.0.71 'fonttools[woff]==4.61.1' pillow==13.0.0
.venv-site/bin/python tools/prepare-site.py
.venv-site/bin/zensical build --clean --strict -f site/zensical.toml
.venv-site/bin/python tools/check-site.py
```

The output is `site/public/`. The preparation step:

- Turns the root feature guides into website pages, adapting relative links and heading anchors.
- Converts the fork's existing README animations to small H.264 previews with WebP posters.
- Builds a self-hosted Google Sans Flex WOFF2 from the source pinned in the UI font manifest, verifying its SHA-256 first. Its license is included with the website font.

Edit the root Markdown guides to update feature instructions. Generated pages, previews and fonts are ignored by Git. The homepage and shared layout live in `site/overrides/`; the fork's colors and typography live in `site/docs/assets/fork.css`.

## Publish

The `pages` workflow builds and checks pull requests. Merging a website or guide change into `main` rebuilds and deploys the site automatically. It can also run manually from Actions.

Repository **Settings → Pages → Source** must be **GitHub Actions**. No custom domain or deployment token is required. The inherited upstream `CNAME` is deliberately absent.

Checks cover the strict documentation build, local page links and anchors, CSS assets, project subpath, self-hosted font references and playable 720×480 H.264 previews.
