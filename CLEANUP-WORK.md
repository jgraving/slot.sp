# Demo media and editor cleanup

## Current scope

- Replace commercial demo labels with original geometric designs in website/GitHub previews and animations.
- Replace console branding on cartridge shells with slot.sp marks while preserving mould geometry and finishes.
- Remove public links and promotion for the browser editor; describe the on-device editor in the user guide.
- Retain the direct-address browser editor for local PNG imports, colors and finishes, without automatic image downloads.
- Retire the website artwork mirror and remove its build/cache restore steps. Publish an empty catalog for compatibility.
- Preserve on-device artwork lookup from its existing upstream source, saved labels and editing behavior.
- Add a maintainer Actions control to republish the empty catalog and clear old artwork caches and website bundles.

## Validation in progress

- Artwork-clear tests: 2 pass. Website CI routing: 6 pass.
- Browser editor JavaScript: 92 tests pass before the final deployment-only lookup switch; repeat after final changes.
- Native rendering works in this environment. An exploratory capture was invalidated when fixture names changed during recording; regenerate from the finished source.
- Rebuilding guide, homepage, interactive and README media from the current production renderer. No physical-device result claimed.
- No merge or website deployment has been performed.

## Deferred follow-up

After cleanup, compare the latest upstream on-device Cart Studio with our integrated version and adapt useful changes to our typography, menus, motion and controls in a separate PR. Cleanup is the current priority.
