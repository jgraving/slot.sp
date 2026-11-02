---
title: Questions
hide:
  - navigation
  - toc
---

# Questions

### What is slot.sp?

A game launcher by Jake Graving, based on [slot by Brandon T. Kowalski](https://github.com/BrandonKowalski/slot). Games appear as cartridges on a carousel. Choose one and press A to play. The [user guide](features/guide.md) covers saving, controls, appearance, audio and parental settings.

### Which games can I play?

Game Boy, Game Boy Color and Game Boy Advance. Supply your own game files; games are not included. Copy them into the [matching folders](features/getting-started.md#add-games), extracting any ZIP or 7z archives first.

### Which handheld is this for?

The Anbernic RG SP is the main device used for this project. Other devices supported by the underlying slot and BaseOS projects may also work. Check the [BaseOS installation instructions](https://slot-cfw.fyi/installing/) for your handheld.

### Can I update from slot?

Yes. Download **[slot.sp-System.zip](https://github.com/jgraving/slot.sp/releases/latest/download/slot.sp-System.zip)** and follow the [update guide](updating.md). Replace the System files on your existing card. Your games, artwork, saves and settings stay in place.

### How does saving work?

Use the game's own save menu as usual. slot.sp also saves a snapshot for resuming, and you can keep manual save states. There is a 30-second undo after saving or loading a state. See [Playing & saving](features/playing.md#save-resume).

### How are Favorites different from Recents?

Favorites contains every game you star. Sort it by name, total recorded playtime or last played. Recents keeps only the last ten games you played. See [Games & artwork](features/library.md#favorites-order).

### Does Light mode change at night?

The menu theme and display presets are separate. **General → UI theme** selects Light or Dark. **Screen → Night mode** manages brightness and screen warmth, with optional day and night start times. See [Day & night](features/night-display.md).

### Can I change settings during a game?

Yes. Tap MENU, then choose Settings. The same settings grid opens over the game screen. In kid mode, it opens the shorter kid settings menu. See the [settings reference](features/settings.md).

### Can I use Bluetooth headphones?

Yes, on supported handhelds. Bluetooth, USB audio, wired headphones and the speaker each remember their volume and mute setting. You can choose the volume after a disconnect and optionally pause the game. See [Audio & headphones](features/bluetooth.md).

### What does kid mode change?

Children see only approved games in the usual system, Favorites and Recents carousels. Set a daily allowance, session timer or bedtime, plus a maximum volume. A four-button parent code protects the settings. The kid menu still lets children resume or leave a game and adjust brightness and volume within the limit. See [Kid mode](features/kid-mode.md).

### Can I change the buttons?

Yes. **Device controls** has separate pages for game buttons, carousel controls and shortcuts. On-screen hints use your chosen buttons. You can also read them in the on-device Buttons & Shortcuts reference. See [Buttons & shortcuts](features/controls.md).

### Can I use my own artwork?

Yes. Add cartridge labels, choose shell colors and finishes, or use custom wallpapers and game borders. [Cart Studio](features/cart-studio.md) can edit one cartridge or a group in your browser. The [customization reference](guide/customizing.md) explains the card folders and file formats.

### Where can I report a problem or suggest a change?

[Open an issue in jgraving/slot.sp](https://github.com/jgraving/slot.sp/issues). Include your handheld model, the version shown in About, and the steps that caused the problem.
