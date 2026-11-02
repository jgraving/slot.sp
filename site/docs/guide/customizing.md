# Customization

Add your own cartridge labels, colors, wallpapers and game borders. [Cart Studio](../features/cart-studio.md) provides a browser editor for labels and cartridge appearance; the sections below describe the files on the card.

## Cart labels

Put a PNG in the matching `Labels/GBA`, `Labels/GB` or `Labels/GBC` folder. Give it the same name as the game: `Labels/GBA/Metroid Fusion.png` goes with `Games/GBA/Metroid Fusion.gba`.

| Cart | Shape | Suggested size |
| --- | --- | --- |
| GBA | 1.94:1 | 1280×661 |
| Game Boy | 1.12:1 | 1024×914 |
| Game Boy Color | 1.12:1 | 1024×914 |

Restart slot.sp after adding labels.

Use [Cart Studio](https://jakegraving.com/slot.sp/studio/) to find artwork, design labels and choose cartridge colors. The [Studio guide](../features/cart-studio.md) covers card access and ZIP export.

## Cart shells

You can also customize the color and appearance of the cart shell.

To pick your own, add a line to `Labels/cart_shell.txt`:

```
Metroid Fusion = auto 6a3fa0 solid
```

Start with the game’s filename without its extension, followed by three settings:

1. **Shape**: `auto`, `notched` or `rounded`. Only matters for Game Boy carts. `auto` uses the notched cart for original Game Boy games and the rounded one for Color-only games.
2. **Color**: six hex digits, no `#`.
3. **Finish**: `solid`, `clear` or `glitter`.

Keep all three options lowercase. A line slot.sp can't read is skipped.

## Wallpapers

Put a PNG in `Wallpapers/` and choose it in **Menu → General → Wallpaper**. **Random** picks one of these images at startup.

| Device | Suggested Size |
| --- | --- |
| RG SP, RG34XX, RG34XXSP | 720×480 |
| RG35XXSP, RG35XX Plus, RG35XX-H, RG28XX | 640×480 |

Other sizes are scaled to fill the screen and cropped from the center. 

The clock, status icons and button hints adapt their colors to the wallpaper. **Wallpaper Tint** darkens the image in Dark mode or lightens it in Light mode. See [Appearance & screen](../features/display.md#backgrounds-animation) for the built-in gradients and UI themes.

## Bezels

Put a PNG in `Bezels/GBA/`, `Bezels/GBC/` or `Bezels/GB/` and select it under **Screen → [system] → Bezel**.

A bezel is the border around the game screen. Leave the game area transparent so it remains visible:

| Device | Bezel size | Game area |
| --- | --- | --- |
| RG SP, RG34XX, RG34XXSP | 720×480 | 480×432 at 120, 24 |
| RG35XXSP, RG35XX Plus, RG35XX-H, RG28XX | 640×480 | 427×384 at 107, 48 |

GBA bezels only show on the 640×480 devices, where the game is 640×427 at 0, 27. On the 720×480 devices a GBA game fills the screen.


## Boot logo

Put a BMP at `Config/bootlogo.bmp` to replace the still image shown when the device powers on. The separate **General → Boot Animation** setting controls the moving slot.sp logo that follows.

slot.sp copies the file as it is, without resizing it, so make it the same size and format as the logo your device already shows.

Delete `Config/bootlogo.bmp` to go back to the slot.sp logo.

## BIOS

BIOS files contain the original console’s startup software. They are optional. If you have your own copies, put them in `BIOS/` to use the original startup sequence:

- `gba_bios.bin`
- `gbc_bios.bin`
- `gb_bios.bin`

If you link with mGBA, both devices need the same BIOS setup. See [Link play](../features/link-play.md).
