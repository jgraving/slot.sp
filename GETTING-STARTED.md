# ⬇ Installation & updates

[← Overview](README.md) · [All guides](GUIDE.md)

[Update an existing installation](#update-an-existing-installation) · [First installation](#first-installation) · [Add games](#add-games) · [Card folders](#card-folders)

Use the System ZIP to update an existing installation, or the SD2 ZIP for a new content card. A new installation also needs BaseOS, the handheld’s operating system.

## Choose your download

| Download | Use it for |
| :--- | :--- |
| **[slot.sp-System.zip](https://github.com/jgraving/slot.sp/releases/latest/download/slot.sp-System.zip)** | Updating a card that already runs slot or slot.sp. Contains the `System` folder. |
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

## Updates from the handheld

Open **General → Updates** while Wi-Fi is connected. Launcher updates target this fork's `jgraving/slot.sp` releases and stage its verified `slot.sp-System.zip`; choose **Restart to install** when ready. The updater keeps Config, Games, Saves and custom System additions. BaseOS updates are separate, offered only for the detected compatible target and adequate battery or charging. Installation is explicit. This integration is still in the OTA test draft and needs physical interruption/recovery testing before release.

**General → Automatic time** enables background UTC correction over a connected network. **Time zone** chooses the local UTC offset and applies immediately. The offset is manual, including daylight-saving changes. Selecting Updates or Automatic time while disconnected opens Network with a Wi-Fi-needed pill.

## File sharing

Open Settings → Network → File sharing and turn Sharing on. On a Mac, use Finder → Go → Connect to Server and enter the displayed `smb://<device IP>/SDCard` address. Choose Registered User, username `slot`, and the password shown by the device. Use **Set password** to choose your own. Both computers must be able to reach each other on the same network.

The share provides read/write access to the whole content card: Games, Saves, Labels, Wallpapers, Config, System and any other folders. Copy files directly to their intended folders; no import step is needed. Eject the share in Finder before turning Sharing off. The launcher rescans games and artwork when sharing stops; restart after changing launcher configuration or System files. Password-authenticated SMB3 encryption remains required. Turn sharing off before launching games, opening Cart Studio or installing updates, so the computer cannot modify those files while they are in use. Link play also stops sharing.

Press Start on a highlighted cartridge to open Cart Studio or choose its emulator (GBA). Automatic time disables manual date/time editing; time zone remains adjustable. Updates shows the installed launcher build as date + exact source hash, the installed BaseOS version and the BaseOS check result.
