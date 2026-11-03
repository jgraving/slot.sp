---
title: Updating
hide:
  - navigation
  - toc
---

# Update slot.sp

Use the **System ZIP** to update a card that already runs slot or slot.sp. It contains the launcher, emulators and bundled files. Copy its `System` folder onto your existing card; your games, artwork, saves and settings stay in place.

**[Download slot.sp-System.zip](https://github.com/jgraving/slot.sp/releases/latest/download/slot.sp-System.zip)**

Starting with a new card? Follow the **[new installation guide](installing.md)** instead.

1. Back up your content card and shut down the handheld.
2. Download and extract **slot.sp-System.zip** using the link above.
3. Copy its `System` folder to the root of your content card, replacing the existing folder's files.
4. Safely eject the card, put it back in the handheld and power on.

Your `Games`, `Saves`, `States`, `Labels` and `Config` folders stay in place. With two cards, update **SD2**; with one card, update the content partition on **SD1**.

!!! tip

    The card’s top folder should contain `System`, `Games` and `Config`. Check that you have not created a nested `System/System` folder.

If you are moving from a version of slot older than 1.4.0, copy over the existing `System` folder without deleting it first. The launcher moves the old settings into `Config`.

[All downloads & release notes](https://github.com/jgraving/slot.sp/releases/latest) · [First installation](installing.md) · [User guide](features/guide.md)

## On-device updates

After installing a version that includes the updater, open **Settings → Network → Updates** while connected to Wi-Fi. Choose **Check for updates**, select **slot.sp** or **BaseOS**, then press **A** to download. **B** cancels a download. Wait for **Restart to install**, then restart from the normal power menu.

Launcher updates come from **jgraving/slot.sp** releases and use `slot.sp-System.zip`. The updater verifies the download and its product, repository, processor architecture and source revision before staging it. Installation replaces files inside `System`; your `Config`, games, saves, labels and wallpapers stay in their existing folders. If an installation is interrupted, keep the card in place and restart so the staged files can finish installing.

A BaseOS update is offered only for the detected model and an older installed version, when the BaseOS update tool is available. Its checksum and compatibility verdict must pass before the UI reports readiness. Plug in before installing BaseOS; keep power connected through its next-boot installation. Launcher and BaseOS updates are separate choices. The updater never replaces this fork with upstream slot.

**Network → Automatic time** sets the UTC clock in the background after Wi-Fi connects. It preserves your chosen UTC offset; it does not infer a timezone. You can turn it off to use the manual Date & time settings. Failed time requests retry while connected.
