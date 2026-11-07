# 🎨 Cart Studio

[← Overview](README.md) · [All guides](GUIDE.md)

Edit cartridge labels, colors, finishes and Game Boy shell shapes directly on your handheld.

Prefer a browser? Visit [Slot Cart Studio](https://studio.slot-cfw.fyi/).

## Open the editor

Press **Select** on any highlighted cartridge to open Cart Studio. Select’s volume and brightness shortcuts still work. **Start** also opens Studio on GB and GBC cartridges. On a GBA cartridge, press **Start** to open the emulator picker, then **Select** for Cart Studio. You can also open it from **Settings → Gameplay → Cart Studio**.

The editor is unavailable during games, link play and kid mode.

## Change the look

Use the D-pad and the button hints to choose a label, shell color, finish or Game Boy cartridge shape. The label printer shows the actual stickers before you apply them; choosing one moves it onto the cartridge. The custom color picker supports RGB and hex values. **B** goes back within the editor.

Colors, finishes, shapes and generated title labels work offline. With Wi-Fi connected, **Label** looks up the game's checksum in the upstream artwork catalog and offers scanned labels or logos when available. Labels that look the same as the current sticker are omitted. After a failed lookup, go back and reopen **Label** to retry. Your ROM stays on the card. A hack or uncommon revision may have no catalog match.

The handheld also fetches cartridge shell logos directly from the original project when Wi-Fi is available and caches them for offline use. Until they are available, shells show the lowercase `slot.sp` wordmark.

Existing custom PNGs on the card remain available. Rounded scan corners are cleaned for the cartridge preview while the original label file stays unchanged.

## Save your changes

Choose **Save** on the options card to save changes and keep editing. **B** closes the editor immediately when everything is saved. With unsaved changes, choose **Save and close**, **Discard changes**, or **Keep editing**. A failed save keeps the editor open for retry.

Labels use the game's filename:

```text
Games/GBA/My Game.gba
Labels/GBA/My Game.png
```

Cartridge appearance choices are saved in `Labels/cart_shell.txt`. See [Games & artwork](LIBRARY.md#cartridge-colors) for the file format and instructions for adding your own PNGs.

---

The on-device editor adapts [upstream Slot](https://github.com/slot-cfw/slot) with slot.sp's menus, cartridge renderer and card layout.
