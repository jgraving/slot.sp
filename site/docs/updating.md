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
