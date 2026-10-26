# Troubleshooting

## A game is missing from the carousel

- Is it unzipped? slot.sp doesn't read `.zip` or `.7z`.
- Is it right in `Games/GBA`, `Games/GB` or `Games/GBC`, and not in a nested directory?
- Did slot.sp boot since you added it? Restart from Settings → About.
- Does it end in `.gba`, `.gb` or `.gbc`? GBA games only show up in `Games/GBA`.

## My save / save state is missing

You probably renamed the game. Saves, states and labels follow the file name, so rename the matching files in `Saves/` and `States/` too.

If you switched a GBA game to the other emulator, your save states are still there. They are tied to the emulator that created them and show up again when you switch back. See [Save States](../features/playing.md#save-states-undo).

## The Read-Only Red Alarm

The red alarm means slot.sp couldn't write to your SD card.

Your progress isn't being saved while it's up.

| It says | Do this |
| --- | --- |
| Can't write to SD card | Something went wrong with the card. |
| Press SELECT + START to repair | slot.sp can try to fix it. It checks the card and restarts. If this fails try plugging it into your computer and use the repair function. |
| Charge the battery to repair | The repair needs 20% battery, or the charger plugged in. |

If it keeps coming back, the card might be on its way out and you should back up your `Saves/` and `States/` while you can.

## A label didn't change

slot.sp spots a new label with the same name and redraws it. If it still looks wrong, check the name matches the game exactly and restart.

## Reporting a bug

Please [open an issue on GitHub](https://github.com/jgraving/slot.sp/issues). It helps a lot if you attach:

- `Logs/slot.log`, the run you're on
- `Logs/slot.log.1`, the run before it. If slot.sp crashed and you rebooted, this is the one
- anything in `Logs/crash/`
- `Logs/slot-card.log`, if it's about the SD card or saves

Tell me which device you have, which version of slot.sp (it's on the About page) and what you did right before.
