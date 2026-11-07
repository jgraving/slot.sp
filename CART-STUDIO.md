# 🎨 Cart Studio

[← Overview](README.md) · [All guides](GUIDE.md)

Edit cartridge labels, colors, finishes and Game Boy shell shapes directly on your handheld.

Prefer a browser? Visit [Slot Cart Studio](https://studio.slot-cfw.fyi/).

## Open the editor

Press **Start** on a highlighted GB or GBC cartridge to open Cart Studio. On a GBA cartridge, press **Start** to open the emulator picker, then **X** for Cart Studio. You can also open it from **Settings → Gameplay → Cart Studio**.

The editor is unavailable during games, link play and kid mode.

## Change the look

Use the D-pad and the button hints to choose a label, shell color, finish or Game Boy cartridge shape. The custom color picker supports RGB and hex values. **B** goes back within the editor.

Colors, finishes, shapes and generated title labels work offline. With Wi-Fi connected, **Label** looks up the game's checksum in the upstream artwork catalog and offers scanned labels or logos when available. Your ROM stays on the card. A hack or uncommon revision may have no catalog match.

The handheld also fetches cartridge shell logos directly from the original project when Wi-Fi is available and caches them for offline use. Until they are available, shells show the lowercase `slot.sp` wordmark.

Existing custom PNGs on the card remain available. Rounded scan corners are cleaned for the cartridge preview while the original label file stays unchanged.

## Save your changes

Choose **Save & close** to save the changes and update the carousel. If saving fails, the editor stays open so you can retry.

Labels use the game's filename:

```text
Games/GBA/My Game.gba
Labels/GBA/My Game.png
```

Cartridge appearance choices are saved in `Labels/cart_shell.txt`. See [Games & artwork](LIBRARY.md#cartridge-colors) for the file format and instructions for adding your own PNGs.

---

The on-device editor adapts [upstream Slot](https://github.com/BrandonKowalski/slot) with slot.sp's menus, cartridge renderer and card layout.
