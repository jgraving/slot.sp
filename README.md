<div align="center">

<picture>
  <source media="(prefers-reduced-motion: reduce)" srcset="previews/hero-static.svg">
  <img src="previews/hero-glass.gif" width="480" height="320" alt="slot.sp — a cartridge slides into place and the glass logo resolves">
</picture>

**A little shelf. A favorite game. One satisfying click.**

Game Boy · Game Boy Color · Game Boy Advance

**[Download System update](https://github.com/jgraving/slot.sp/releases/latest/download/slot.sp-System.zip)** · [All downloads](https://github.com/jgraving/slot.sp/releases/latest) · [First-time setup](https://slot-cfw.fyi/installing/)

</div>

**slot.sp** is an SP-focused fork of [Brandon T. Kowalski’s slot](https://github.com/BrandonKowalski/slot). Browse your games as physical cartridges, slide one into the bay, and play. An SP-inspired shell, soft clicks, flowing color and thoughtful controls make the handheld feel like a small object built for your collection.

## Pick a cart. Slot it in.

<p align="center">
<img src="previews/readme-carousel.gif" width="48%" alt="Larger cartridges moving across the Graphite carousel">
<img src="previews/readme-core-picker.gif" width="48%" alt="Choosing mGBA or gpSP on the cartridge circuit board">
</p>
<p align="center">
<img src="previews/readme-link.gif" width="48%" alt="Link-cable interface connecting at the EXT.1 port">
<img src="previews/readme-shutdown.gif" width="48%" alt="The textured power switch slides to OFF before the display fades">
</p>

*Interface captures from the production renderer. The link preview demonstrates the interface; it is not a multiplayer hardware test.*

## Made for the SP

- **A shell that feels at home.** Graphite, Platinum, Pearl Blue and Pearl Pink finishes, a sculpted cartridge bay, matching EXT.1 port and an animated power switch. Choose a finish in **General → Shell Color**.
- **Your collection, up close.** Large cartridges, Recents and Favorites, with the same shelves for GB, GBC and GBA. Rounded label scans get automatic background cleanup on device; original PNGs stay untouched.
- **Motion you can tune.** Flowing or Static Gradient wallpapers, the original soft carousel click with a **Carousel Sounds** toggle, and an optional **Boot Animation**. Verified supported panels can use 120 Hz UI refresh while games retain their native timing.
- **Comfort after dark.** **Select + X** switches between saved day/night brightness and warmth presets. Each preset remembers your adjustments.
- **Listen your way.** Speaker, wired headphones, Bluetooth and detected USB audio, with separate volume/mute settings per output. Saved Bluetooth headphones can reconnect automatically.
- **Keep your place.** Screenshot gallery, save/load undo, periodic saves and configurable insertion/ejection state behavior inherited from slot. Choose mGBA or gpSP through the cartridge’s circuit-board core picker.

## A shelf they can make their own

Kid mode keeps the familiar **Recents, Favorites and system carousels**, showing only games you approve. There is no extra Kids shelf, and your own Favorites and Recents stay intact.

1. Open **Menu → Kid Mode**. On first use, this opens the settings screen so you can configure everything before setting a code.
2. Open **Approved games** and press **A** to toggle each game. Approved cartridges show a **gold smiley in the top-left corner**. With kid mode off, X on a cartridge also toggles approval, without a permanent carousel hint.
3. Choose **Daily allowance**, **Play for** or **Play until**, then adjust the time and volume limit.
4. Select **Start kid mode**. If you have not set a parent code, enter four buttons using **A/B/X/Y or the D-pad**, press **Start**, and enter the same code again to confirm. Each press fills a slot with the actual button artwork and updates the entry count. **Select** deletes the last button; **Menu** cancels. You can also choose **Set parent code** separately from settings.

| Limit | How it works |
| --- | --- |
| Daily allowance | Counts active gameplay. Usage persists across restarts; the allowance renews on a new calendar day. |
| Play for | Sets a deadline from when a parent starts or applies the session. Sleep and reboot do not extend it. |
| Play until | Sets a bedtime for today using the handheld’s clock. |
| Volume limit | Defaults to **50%**, adjustable from **10–80%**, across audio outputs. |

When time runs out, slot.sp uses its save paths, returns to the shelf and asks for a parent. Parent dialogs pause and mute gameplay. Settings and potentially disruptive shortcuts require parent access; approved-game browsing stays simple. Set the handheld’s clock correctly before enabling limits.

To change limits or leave kid mode, open the menu and enter your code. **Applying a duration starts a new deadline; it does not reset today’s daily usage.**

**Forgot the code?** Shut down, put the content card in a computer and read `Config/kids/parent-code.txt`. The recovery code is intentionally stored in plain text, so anyone with access to the card can read it. Kid mode starts off; damaged policy data blocks play until a parent resolves it.

## Install or update

Downloads are public and do not need a GitHub account.

| Download | Choose this when… |
| --- | --- |
| **[slot.sp-System.zip](https://github.com/jgraving/slot.sp/releases/latest/download/slot.sp-System.zip)** | You already have slot installed and want to update. Contains `System` only. |
| [slot.sp-SD2.zip](https://github.com/jgraving/slot.sp/releases/latest/download/slot.sp-SD2.zip) | You are setting up a content card and need the starter folders and defaults. |
| `SHA256SUMS` · `build-info.json` | You want to verify a download or identify its exact source commit. Find both on the [release page](https://github.com/jgraving/slot.sp/releases/latest). |

### Updating an existing card

1. Back up your card and shut down the handheld.
2. Extract **slot.sp-System.zip** onto the root of the content card (**SD2**), replacing the existing `System` files.
3. Keep your games, saves and configuration folders in place. Safely eject the card, reinsert it and boot.

### Starting fresh

Follow the [BaseOS and slot installation guide](https://slot-cfw.fyi/installing/), using this fork’s **SD2 ZIP** for the content card. These downloads are folder packages, not bootable OS images.

Put uncompressed games in `Games/GB`, `Games/GBC` or `Games/GBA`, with label images in the matching `Labels` folders. **No ROMs or BIOS files are included. `.zip` and `.7z` games are not supported.** Existing card paths and the `System/slot` launcher remain compatible.

## Useful controls and settings

| Action | Where to find it |
| --- | --- |
| Switch day/night presets | **Select + X** |
| Take an in-game screenshot | **Menu + X** |
| Approve a game for kid mode | **X** on its cartridge, with kid mode off |
| Change shell or wallpaper | **General → Shell Color / Wallpaper** |
| Disable carousel clicks or the boot animation | **Carousel Sounds / Boot Animation** in settings |
| Configure lid-close, sleep and shutdown behavior | Power settings; see [Power options](POWER-OPTIONS.md) |

Shortcuts can be remapped. Kid mode restricts shortcuts that could change settings, alter saves or open parent screens.

[Audio and Bluetooth](BLUETOOTH.md) · [Day/night display](NIGHT-DISPLAY.md) · [Power options](POWER-OPTIONS.md) · [Shell finishes](previews/rgsp-shell-finishes.png)

## Hardware and testing

Code releases run an ARM64 build and device regression checks before publication. Release assets include checksums, source identification and emulator provenance. Automated checks do not replace testing on a handheld.

The fork focuses on the SP. Upstream also supports RG34XX, RG34XXSP, RG35XXSP, RG35XX Plus, RG28XX and RG35XX-H; fork additions have not all been tested on every model. Unsupported panels retain normal refresh rates. USB-C headphone playback has been reported working on RG SP; Bluetooth volume synchronization remains experimental. Physical link play and deep sleep/wake need device-specific testing.

The latest kid-mode changes still need RG SP testing. See [audio testing and limitations](BLUETOOTH.md#tested-devices-and-known-limitations).

## Credits

**slot.sp by Jake Graving.** Based on [slot by Brandon T. Kowalski](https://github.com/BrandonKowalski/slot), with original skeuomorphic artwork by [MarkusMaribu](https://github.com/MarkusMaribu). Emulation by [mGBA](https://mgba.io) and [gpSP](https://github.com/libretro/gpsp). BaseOS and the fork’s adapted high-refresh panel support come from [Prashant Vaibhav](https://github.com/pvaibhav/slot).

Development and documentation are AI-assisted, with human direction and review. See upstream’s [credits](site/docs/credits.md), [upstream sync history](UPSTREAM-SYNC.md) and [changelog](CHANGELOG.md).

[GPL-3.0-or-later](LICENSE). Device packages include emulator licenses, provenance, patches and gpSP source under `System/licenses`.
