# Physical UI motion

Menus retain opaque surfaces and native-sized content until their exit completes. Entry and exit use one spring anchored at the cartridge bay; the menu rises and expands as the retained cartridges retreat beneath the bay. Every submenu uses the same retained carousel position. Closing reverses both surfaces continuously, including an interrupted transition. Carousel momentum continues while the menu is open.

The analytic damped spring in `slot-ui/src/expressive.rs` gives matching trajectories at 30, 60 and 120 Hz, including dropped frames:

- Panel entry and exit: stiffness 800, damping ratio 1.0.
- Page travel and shared-container morph: stiffness 600, damping ratio 0.82.
- Focus, switches and sliders: stiffness 800, damping ratio 0.76.
- Pause-card expansion: stiffness 800, damping ratio 1.0.

The main menu and in-game settings use the same native icon tiles. Pause actions use the same grid, including the restricted child menu. A submenu changes the bounds of the existing container while opaque text and controls travel through it without stretching. Titles stay centered above the card; rows have separate outer and inner margins. Back preserves current position and velocity. Input remains active during motion. Device chrome remains anchored and occludes departing UI.

Settings notices and charging feedback also travel as opaque surfaces. The charging chip clips a level fill to its capsule, with contrasting native text on each side of the fill boundary. The top status bar follows Android's reading order: time at left, compact status symbols before a horizontal battery at right. Its cached native battery contains the numeric level and a charging bolt, with charging and low-battery colors. Status glyphs share one flat optical size and spring response.

Clock format is a presentation preference shared by status, editors, Night mode, parental bedtimes, game details and save/screenshot labels. Internal timestamps, time-zone offsets, durations and stored filenames retain their existing meaning.

Reference: [Android unified battery layout](https://android.googlesource.com/platform/frameworks/base/+/a6c764c76a44cd40dd48695512032b807092e19a/packages/SystemUI/src/com/android/systemui/battery/unified/BatteryLayersDrawable.kt) and [Material 3 Expressive motion](https://blog.google/products-and-platforms/platforms/android/material-3-expressive-android-wearos-launch/).
