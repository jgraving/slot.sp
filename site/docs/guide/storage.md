# Storage reference

Games, artwork, saves and settings are separate folders on your content card. With two cards, this is the card in `SD2`; with one card, it is the content partition on `SD1`.

## The folders

| Folder                                  | What's in it                        |
|-----------------------------------------|-------------------------------------|
| `Games/GBA`, `Games/GB`, `Games/GBC`    | Your games                          |
| `Labels/GBA`, `Labels/GB`, `Labels/GBC` | Cart labels                         |
| `Saves/`                                | Progress saved by the game itself |
| `States/`                               | Save states                         |
| `Screenshots/`                          | Your screenshots                    |
| `BIOS/`                                 | Optional BIOS files                 |
| `Wallpapers/`                           | Your wallpapers                     |
| `Bezels/GBA`, `Bezels/GB`, `Bezels/GBC` | Your bezels                         |
| `Config/`                               | Your settings                       |
| `Logs/`                                 | Logs, for when something breaks     |
| `System/`                               | slot.sp itself                         |

slot.sp makes any of these that are missing when it boots, except `Screenshots/`, which appears with your first screenshot.

## Games

| Folder      | Takes         |
|-------------|---------------|
| `Games/GBA` | `.gba`        |
| `Games/GB`  | `.gb`, `.gbc` |
| `Games/GBC` | `.gb`, `.gbc` |

The folder decides which carousel a game goes on.

!!! warning "Extract game archives first"

    slot.sp does not read `.zip` or `.7z` files.

Put games right in the folder. slot.sp doesn't look inside subdirectories.

## Names matter

slot.sp matches a game’s save files, artwork, cartridge appearance and emulator choice by filename.

If you rename `Metroid Fusion.gba`, rename its save in `Saves/GBA` too, or slot.sp won't find it.

## Saves and states

- Battery saves are `Saves/<system>/<game>.sav`.
- Save states are in `States/<system>/<emulator>/<game>/`. Each state has a PNG next to it for the save-state browser.

Periodically back up `Saves/` and `States/`. Keep a backup before changing or replacing the card.

## Config

Most of what's in `Config/` is written by slot.sp when you change a setting.

Use the on-device menus to change settings. These files are listed for backups and troubleshooting.

| File                                            | What it holds                                      |
|-------------------------------------------------|----------------------------------------------------|
| `slot.state`                                    | Your settings and where you left off               |
| `controls.txt`, `shortcuts.txt`, `carousel.txt` | Your button layout                                 |
| `favorites.txt`, `recents.txt`                  | Your Favorites and Recents                         |
| `selected_core.txt`                             | Which emulator each GBA game uses                  |
| `video_mode.txt`                                | Which Game Boy games you've stretched              |
| `cart_shell.txt`                                | Cart colors, see [Customization](customizing.md). A `Labels/cart_shell.txt` overrides it |
| `bootlogo.bmp` | Your custom startup still image. |
| `display.txt` | Day/night brightness and warmth presets and schedule. |
| `power.txt` | Sleep, lid and power settings. |
| `sound.conf` | Volume on disconnect and Pause on disconnect. |
| `play-history.txt` | Recorded play time and launch history. |
| `kids/` | Kid-mode approvals, limits, usage and the plain-text parent-code recovery file. |

To reset settings, shut down and back up `Config/` before removing it. Removing this folder also resets button mappings, Favorites, clock settings and parental controls. You will need to set the clock again at the next boot.

## Updating

Updates only replace `System/`. Everything else on this list is yours and stays put.

See [Updating](../updating.md) for more.
