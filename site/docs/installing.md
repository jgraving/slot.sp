---
hide:
  - navigation
  - toc
---

# Install slot.sp

For a new card, first install **BaseOS**, the handheld’s operating system. Then extract **slot.sp-SD2.zip** and copy its contents onto the card used for games. The ZIP includes the launcher, starting folders and settings.

Choose the instructions for one or two SD cards below. If your card already runs slot or slot.sp, follow the **[update guide](updating.md)**.

<details class="way" id="one-sd-card" open markdown>
<summary>One SD card</summary>

<div class="install-steps" markdown>

1.  Write BaseOS to an SD card
    { .step-title role="heading" aria-level="3" }

    Download the latest release of
    [BaseOS](https://github.com/pvaibhav/BaseOS/releases/latest) and write the `.img` to an SD card with
    a tool such as [Rufus](https://rufus.ie) or [Raspberry Pi Imager](https://www.raspberrypi.com/software/).

    Insert this card into the SD card slot labeled `SD1`.

2.  Expand the storage
    { .step-title role="heading" aria-level="3" }

    After inserting the SD card, power on your device. You should see a BaseOS boot logo and a message saying that storage is being expanded.

    When expansion finishes, BaseOS reports that no frontend (game launcher) was found. Power off the device.

3.  Download the latest release
    { .step-title role="heading" aria-level="3" }

    Download [slot.sp-SD2.zip](https://github.com/jgraving/slot.sp/releases/latest/download/slot.sp-SD2.zip) from GitHub.

4.  Copy it onto your SD card
    { .step-title role="heading" aria-level="3" }

    Eject the SD card from `SD1` and connect it back to your computer. Extract the ZIP and copy its contents to the card’s top folder, so System and Games are directly inside it.

5.  Add your games
    { .step-title role="heading" aria-level="3" }

    Copy your game files into the matching `Games/GBA`, `Games/GBC` or `Games/GB` folder. Put label artwork in the matching folder under `Labels/`, using the same filename as the game with a `.png` extension.

    !!! warning "Extract game archives first"

        slot.sp reads `.gba`, `.gb` and `.gbc` game files. Extract `.zip` or `.7z` archives before copying them to the card.

6.  Put the card back in `SD1` and power on
    { .step-title role="heading" aria-level="3" }

    Open the [user guide](features/guide.md) for buttons, settings and features.

</div>

</details>

<details class="way" id="two-sd-cards" markdown>
<summary>Two SD cards</summary>

<div class="install-steps" markdown>

1.  Write BaseOS to an SD card
    { .step-title role="heading" aria-level="3" }

    Download the latest release of
    [BaseOS](https://github.com/pvaibhav/BaseOS/releases/latest) and write the `.img` to an SD card with
    a tool such as [Rufus](https://rufus.ie) or [Raspberry Pi Imager](https://www.raspberrypi.com/software/).

    Insert this card into the SD card slot labeled `SD1`.

2.  Download the latest release
    { .step-title role="heading" aria-level="3" }

    Download [slot.sp-SD2.zip](https://github.com/jgraving/slot.sp/releases/latest/download/slot.sp-SD2.zip) from GitHub.

3.  Copy it onto SD2
    { .step-title role="heading" aria-level="3" }

    Prepare your second SD card by formatting it to `FAT32` or `exFAT` with a `Master Boot Record (MBR)` partition scheme.

    Extract the ZIP and copy its contents to the second card’s top folder, so System and Games are directly inside it.

4.  Add your games
    { .step-title role="heading" aria-level="3" }

    Copy your game files into the matching `Games/GBA`, `Games/GBC` or `Games/GB` folder. Put label artwork in the matching folder under `Labels/`, using the same filename as the game with a `.png` extension.

    !!! warning "Extract game archives first"

        slot.sp reads `.gba`, `.gb` and `.gbc` game files. Extract `.zip` or `.7z` archives before copying them to the card.

5.  Put the second card into `SD2` and power on
    { .step-title role="heading" aria-level="3" }

    Open the [user guide](features/guide.md) for buttons, settings and features.

</div>

</details>
