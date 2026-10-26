# 🎨 Cart Studio

[← Overview](README.md) · [All guides](GUIDE.md) · **[Open Cart Studio](https://jakegraving.com/slot.sp/studio/)**

Find artwork for your games, design labels and choose cartridge colors and finishes in your browser.

## Your files stay on your device

Cart Studio is a **static website**. The editor runs in your browser, on your device. Your games, labels and edits are never uploaded to or stored on a Studio server.

It downloads game databases and artwork to find matches. Your changes stay in the browser until you choose **Write to card** or download a ZIP. Save before closing the tab to keep your edits.

## Open your card

1. Shut down the handheld and put its content card in your computer.
2. Open **[Cart Studio](https://jakegraving.com/slot.sp/studio/)**.
3. Choose the card's **top folder**, with `Games` and `Labels` inside it.

In Chrome or Edge, **Choose SD Card** allows Studio to save directly to the card. **Open folder for ZIP export** lets you work with a folder and download the changes instead. Other browsers offer folder import and ZIP export.

## Choose artwork

Studio identifies the games in `Games/GBA`, `Games/GBC` and `Games/GB`. Choose a cartridge to edit it.

| Choice | What it does |
| :--- | :--- |
| **Real Label** | Uses an available scanned label, or asks you to choose a label PNG. |
| **Logo Only** | Places an available logo on a colored background, or asks you to choose a logo PNG. |
| **Custom logo** | Uses your PNG as a logo, with a background you can adjust. |
| **Custom label** | Uses your PNG or scanned label as the full label. |
| **On the card** | Keeps the label already saved for that game. |

Use the **Game** selector if the automatic match needs changing. **Fill unlabelled carts** helps you choose artwork for games without labels.

Rounded black or white scanner backgrounds are cleaned automatically, using the same process as the handheld. Cleaned corners remain transparent when you save a label. Previews use the current slot.sp cartridge artwork, including textured plastic and clear-shell interiors.

## Change the look

The editor lets you change the label's background, cartridge color and finish. Use presets or pick your own color, then preview the result before saving.

For your own label PNGs, use these suggested sizes:

| System | Size |
| :--- | :--- |
| Game Boy Advance | 1280 × 661 |
| Game Boy / Game Boy Color | 1024 × 901 |

## Save your changes

### Direct card access

Choose **Write to card** when the changes are ready. Safely eject the card after the write finishes.

### ZIP export

Download the ZIP and extract its contents onto the card's root. Merge the `Labels` folder when asked. The export contains the label files and cartridge appearance settings.

Put the card back in the handheld and restart slot.sp to load the new artwork.

### Files on the card

Labels follow the game's filename:

```text
Games/GBA/Metroid Fusion.gba
Labels/GBA/Metroid Fusion.png
```

Cartridge appearance choices are saved in `Labels/cart_shell.txt`. See [Games & artwork](LIBRARY.md#cartridge-colors) for the file format.

---

Cart Studio for slot.sp is based on [Brandon T. Kowalski's Cart Studio](https://github.com/BrandonKowalski/slot-cart-studio). [Source code](https://github.com/jgraving/slot-cart-studio) · [Games & artwork](LIBRARY.md) · [Installation & updates](GETTING-STARTED.md)
