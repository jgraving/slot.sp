# Troubleshooting

Check the section that matches the problem. Before changing card files, back up your saves and settings.

## A game is missing from the carousel

- Is it unzipped? slot.sp doesn't read `.zip` or `.7z`.
- Is it right in `Games/GBA`, `Games/GB` or `Games/GBC`, and not in a nested directory?
- Did slot.sp boot since you added it? Restart from Settings → About.
- Does it end in `.gba`, `.gb` or `.gbc`? GBA games only show up in `Games/GBA`.

## My save / save state is missing

If you renamed the game, check that its files in `Saves/` and `States/` have the same name. Saves, states and labels are matched to the game by filename.

If you switched a GBA game to the other emulator, your save states are still there. They are tied to the emulator that created them and show up again when you switch back. See [Save States](../features/playing.md#save-states-undo).

## The Read-Only Red Alarm

This red warning means slot.sp cannot write to the SD card. New progress is not being saved while the warning is visible.

| It says | Do this |
| --- | --- |
| Can't write to SD card | Back up the card, then check it for errors on a computer. |
| Press SELECT + START to repair | slot.sp can try to fix it. It checks the card and restarts. If this fails try plugging it into your computer and use the repair function. |
| Charge the battery to repair | The repair needs 20% battery, or the charger plugged in. |

If the warning returns, back up `Saves/` and `States/` and consider replacing the card.

## A label didn't change

slot.sp spots a new label with the same name and redraws it. If it still looks wrong, check the name matches the game exactly and restart.

## Bluetooth will not connect

Read the short error message at the bottom of the screen. Wake the headphones and try again if they did not answer. If pairing failed, forget the saved device and pair it again. See [Audio & headphones](../features/bluetooth.md#switch-back-or-turn-bluetooth-off).

## Light mode and night mode seem different

They control different things. **General → UI theme** selects Light or Dark menus. **Screen → Night mode** controls saved brightness and warmth, including an optional schedule. See [Day & night](../features/night-display.md).

## Reporting a bug

Please [open an issue on GitHub](https://github.com/jgraving/slot.sp/issues). It helps a lot if you attach:

- `Logs/slot.log`, the run you're on
- `Logs/slot.log.1`, the run before it. If slot.sp crashed and you rebooted, this is the one
- anything in `Logs/crash/`
- `Logs/slot-card.log`, if it's about the SD card or saves

Tell me which device you have, which version of slot.sp (it's on the About page) and what you did right before.
