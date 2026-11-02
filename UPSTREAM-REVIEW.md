# Upstream review — 2 November 2026

Review targets:

- BrandonKowalski/slot through `0b95d32fea8b8c429e7539d5e448ebf88ff47ef8`.
- BrandonKowalski/slot-cart-studio through `71bf6aae7ca0215f0cb98c8c5971da2db3035699`.
- Fork baselines: slot.sp `2e53a8b709bff77e82a3349d9bda41d83519b140` and
  Cart Studio `7908e952da31c46d94d9aa0d05e7168af9d6aeec`.

This is a selective integration. These PRs propose changes for review; neither
project's main branch, release channel or published site is updated by this review.

## Prepared independently

| Kind | Project | Change | Review |
| --- | --- | --- | --- |
| Bug fix | slot.sp | Decode RetroArch RZIP v1 battery saves; preserve damaged and unsupported imports | [#177](https://github.com/jgraving/slot.sp/pull/177) |
| Bug fix | slot.sp | mGBA's blank first Game Boy frame after LCD enable | [#178](https://github.com/jgraving/slot.sp/pull/178) |
| Feature | Cart Studio | Distinguish same-title region/hack variants by filename, including bulk selection | [#24](https://github.com/jgraving/slot-cart-studio/pull/24) |
| Feature | Cart Studio | Read downloaded WebP artwork and export device-compatible PNGs | [#25](https://github.com/jgraving/slot-cart-studio/pull/25) |

Each code PR starts from its project's main branch. The two slot.sp PRs append
separate provenance entries to UPSTREAM-SYNC.md; merging both may require combining
those entries. No code changes depend on the other PRs. Validation results and
remaining hardware checks are recorded in the individual PR descriptions.

## Major choices before the next integration

| Area | Upstream changes | Proposed integration boundary |
| --- | --- | --- |
| Wi-Fi and card tools | Saved networks, scanning, password keyboard/retry, art fetch/cache, on-device cart editing, Gamebit browser card manager | Separate feature PRs for Wi-Fi setup, artwork/editor tools and Gamebit. Preserve Bluetooth/USB audio routing, fork menu cards and kid-mode restrictions. Bring dependent keyboard, browser-upload and folder-protection fixes with the feature they fix. |
| OTA | Slot and BaseOS update checks, downloads, cancellation and launch-script handling | Separate from Wi-Fi. Slot updates must use **jgraving/slot.sp** release assets, never install upstream firmware over this fork. Decide whether to offer BaseOS updates as well. Retain current manual System ZIP workflow. |
| Clock | City/time-zone and summer-time choices, 12-hour display, first-boot flow, network time | Separate clock PR with migration and tests for scheduled display overrides, play history and kid-time accounting. Do not substitute a clock screen without those checks. |
| GBA Video | GBAV folders/platform on device and a matching Studio tab | Coordinated device and Studio PRs. Preserve old platform/settings identities; do not repin Studio to upstream Slot. |
| Display options | Shimmerless, saturation/contrast, scale-specific bezels and vertical placement | Keep separate from bug fixes. Review how these fit the fork's screen cards and native-game/high-refresh pacing. Existing scaling and frame-blending review PRs remain held. |

No new network server, OTA installer, clock migration or GBAV platform is introduced
by the four prepared code PRs.

## Other feature batches available for review

These are not included in the prepared bug fixes. Each should have its own feature
PR with the associated upstream follow-up fixes and documentation:

- **Reset Game and Undo Reset:** optional remappable shortcut, initially unbound;
  preserve kid-mode restrictions, link-session rules and the existing undo timeout.
- **Boktai solar sensor:** clock-derived sunlight with optional up/down bindings;
  integrate against the fork's clock rather than assuming the deferred rewrite.
- **Local game guides:** viewer, wrapping and revised controls; retain fork button
  hints, pause-menu layout and kid-mode policy.
- **Shelf wallpaper:** asynchronous art loading and cache invalidation; integrate
  with existing light/dark wallpapers without restoring the removed legacy blur option.
- **Shortcut grouping:** preserve the fork's Day / Night shortcut, button icons,
  binding conflicts and kid-mode exclusions.

## Deliberately retained fork behavior

- The current slot.sp wordmark, website, page motion, Material-style UI cards,
  cartridge geometry, metallic shell finishes and gold favorite star remain.
  Upstream's red-heart favorite and visual/typography replacements are not pulled in.
- Native-game/high-refresh pacing and kernel CPU-governor behavior remain. The
  prior review already adapted the audio, fast-forward and brightness fixes through
  upstream `729c61d`; these are not duplicated.
- Long-toast behavior is already covered by the fork's full-size text tests.
  Upstream's new network-specific toast variants arrive only with their features.
- Studio retains search, explicit match approval, bulk styling, lazy previews,
  cached checksums, progress estimates, save completion, local-file privacy and
  same-origin artwork. Upstream header/footer links and visible build hashes are
  not restored.
- Studio's renderer stays pinned to jgraving/slot.sp. Upstream `6278fa6` and
  `71bf6aa` change its shared renderer/pin; transplanting those wholesale would
  replace fork artwork and decoder behavior. The WebP adaptation uses the current
  pinned modules instead.
- Existing drafts [#110](https://github.com/jgraving/slot.sp/pull/110) (scaling),
  [#111](https://github.com/jgraving/slot.sp/pull/111) (frame blending) and
  [#113](https://github.com/jgraving/slot.sp/pull/113) (ZIP design) stay unmerged.
  RZIP battery-save decoding does not enable zipped ROMs.
- Upstream's Bun/build changes belong to its new Gamebit frontend. They are not
  added to the current fork build in isolation. New docs and fixes for features
  absent from this fork stay with their corresponding future feature PRs.
