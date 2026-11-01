# 🎨 Cart Studio

[← Overview](README.md) · [All guides](GUIDE.md) · **[Open Cart Studio](https://jakegraving.com/slot.sp/studio/)**

Find scanned labels and logos, design your own labels, and choose cartridge colors and finishes. Edit one game or several together, then save to the card or export a ZIP.

## Your files stay on your device

Cart Studio runs in your browser. Your game files, labels and edits are never uploaded to or stored on a Studio server.

It downloads game databases and artwork to find matches. Your changes stay in the browser until you choose **Write to card** or download a ZIP. Save before closing the tab to keep your edits.

## Open your card

1. Shut down the handheld and put its content card in your computer.
2. Open **[Cart Studio](https://jakegraving.com/slot.sp/studio/)**.
3. Choose **Get Started**, then the card's **top folder**, with `Games` and `Labels` inside it.

In Chrome or Edge, **Choose SD Card** allows Studio to save directly to the card. **Open folder for ZIP export** lets you work with a folder and download the changes instead. Other browsers offer folder import and ZIP export.

## Choose artwork

Studio downloads scanned labels and logos from the published artwork catalog. Studio identifies the games in `Games/GBA`, `Games/GBC` and `Games/GB`. Choose a cartridge to edit it.

| Choice | What it does |
| :--- | :--- |
| **Real Label** | Uses an available scanned label, or asks you to choose a label PNG. |
| **Logo Only** | Places an available logo on a colored background, or asks you to choose a logo PNG. |
| **Custom logo** | Uses your PNG as a logo, with a background you can adjust. |
| **Custom label** | Uses your PNG or scanned label as the full label. |
| **On the card** | Keeps the label already saved for that game. |

Use the **Game** selector if the automatic match needs changing. **Fill unlabelled carts** helps you choose artwork for games without labels.

Studio removes black or white backgrounds around rounded label scans. The cleaned corners stay transparent when you save. The preview shows how the label will look on a slot.sp cartridge, including solid, clear and glitter finishes.

## Change the look

The editor lets you change the label's background, cartridge color and finish. Use presets or pick your own color, then preview the result before saving. Finish buttons also work with the automatic shell color; you do not need to choose a new color first.

For your own label PNGs, use these suggested sizes:

| System | Size |
| :--- | :--- |
| Game Boy Advance | 1280 × 661 |
| Game Boy / Game Boy Color | 1024 × 901 |

## Change several cartridges together

Choose **Select carts**, then click or use Space/Enter to select cartridges. Selections stay selected when you switch system tabs. **Select this system** selects every cartridge on the current tab.

Choose **Label** or **Logo**, a shell color, a finish, or any combination. Changes preview immediately on the selected carts, including while you adjust a color. **Keep existing** leaves that property alone. If a label or logo is unavailable, Studio keeps that cartridge’s current artwork and reports it. Cartridge shapes stay as they are. Each cartridge’s editor has Undo.

The **Select carts** switch shows **On** while selecting. Turn it off or choose **Done** to return to editing individual carts. Save edits with **Write to card** or ZIP export.

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
