# ⬇ Installation & updates

[← Overview](README.md) · [All guides](GUIDE.md)

[Update an existing installation](#update-an-existing-installation) · [First installation](#first-installation) · [Add games](#add-games) · [Card folders](#card-folders)

## Choose your download

| Download | Use it for |
| :--- | :--- |
| **[slot.sp-System.zip](https://github.com/jgraving/slot.sp/releases/latest/download/slot.sp-System.zip)** | Updating an installation that already runs slot. Contains the `System` folder. |
| **[slot.sp-SD2.zip](https://github.com/jgraving/slot.sp/releases/latest/download/slot.sp-SD2.zip)** | A new content card, with the folder structure and starting settings. |

## Update an existing installation

1. Back up your content card and shut down the handheld.
2. Download and extract **slot.sp-System.zip**.
3. Copy its `System` folder to the root of your content card, replacing the existing folder's files.
4. Safely eject the card, put it back in the handheld and power on.

Your `Games`, `Saves`, `States`, `Labels` and `Config` folders stay in place. With two cards, the content card is **SD2**.

> [!TIP]
> When you open the card, you should see `System` alongside `Games` and `Config`. Avoid putting the new folder inside the old one as `System/System`.

## First installation

slot.sp runs on **BaseOS**. Use the [BaseOS installation instructions](https://slot-cfw.fyi/installing/) to prepare the operating-system card, and use this fork's **slot.sp-SD2.zip** for the game-launcher files.

For a two-card setup:

1. Install BaseOS on **SD1** using the installation guide.
2. Format **SD2** as FAT32 or exFAT, with an MBR partition table.
3. Extract **slot.sp-SD2.zip** directly onto SD2.
4. Add your games, insert both cards and power on.
5. Set the date, time and UTC offset when asked.

The ZIP downloads contain files to copy onto the content card. BaseOS is the separate image you write to the operating-system card. The installation guide also covers a one-card setup.

## Add games

Place uncompressed game files directly in the appropriate folder:

| System | Folder | File type |
| :--- | :--- | :--- |
| Game Boy Advance | `Games/GBA/` | `.gba` |
| Game Boy | `Games/GB/` | `.gb` or `.gbc` |
| Game Boy Color | `Games/GBC/` | `.gb` or `.gbc` |

Extract `.zip` or `.7z` archives before copying games. Keep games directly in these folders, rather than in subfolders. Games and BIOS files are supplied by you.

To add cover artwork, give the PNG the same name as the game:

```text
Games/GBA/Metroid Fusion.gba
Labels/GBA/Metroid Fusion.png
```

Restart after adding games or artwork. You can restart from **Menu → About**, then <kbd>X</kbd>.

## Card folders

| Folder | Contents |
| :--- | :--- |
| `Games/` | Game files, organized by system. |
| `Labels/` | Game artwork and cartridge appearance settings. |
| `Saves/` | The game's own saves. |
| `States/` | Save states and resume snapshots. |
| `Screenshots/` | Captured pictures, organized by system and game. |
| `BIOS/` | Optional `gba_bios.bin`, `gbc_bios.bin` and `gb_bios.bin`. |
| `Wallpapers/` | Your background PNGs. |
| `Bezels/` | Your game-screen borders. |
| `Config/` | Settings, controls, Favorites, Recents and parent-code recovery. |
| `System/` | The launcher, emulators and bundled licenses. |

Keep a game's filename consistent when moving its artwork or saves: these files are matched by name.

<details>
<summary><strong>Verify a download or identify a build</strong></summary>

The [release page](https://github.com/jgraving/slot.sp/releases/latest) includes `SHA256SUMS` and `build-info.json`. The first lets you verify the ZIP's checksum; the second identifies the source commit used for that release.

On Linux, with the ZIP and `SHA256SUMS` in the same directory:

```sh
sha256sum --ignore-missing -c SHA256SUMS
```

</details>

---

Next: [Buttons & shortcuts](CONTROLS.md) · [Games & artwork](LIBRARY.md)
