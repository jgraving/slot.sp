# 🌗 Day & night

[← Overview](README.md) · [All guides](GUIDE.md)

Save separate brightness and screen warmth settings for day and night. Switch with a shortcut, or set times for the handheld to switch automatically.

## Switch presets

Hold <kbd>SELECT</kbd> and press <kbd>X</kbd>. This works on the carousel and during a game, with the default controls outside kid mode.

slot.sp remembers the active preset when you restart. These brightness and warmth presets are independent of **General → UI theme**: switching Day/Night does not switch the Light/Dark appearance.

## Set up each preset

1. Switch to the preset you want to change.
2. Adjust brightness and warmth using the shortcuts below, or **Menu → Screen**.
3. Switch to the other preset and set it up the same way.

Each adjustment saves to the preset you are using.

| Control | Action |
| :--- | :--- |
| <kbd>SELECT</kbd> + <kbd>Up</kbd> | Raise brightness. |
| <kbd>SELECT</kbd> + <kbd>Down</kbd> | Lower brightness. |
| <kbd>SELECT</kbd> + <kbd>Left</kbd> | Reduce warmth. |
| <kbd>SELECT</kbd> + <kbd>Right</kbd> | Increase warmth. |

Keep both buttons held to adjust continuously after a short pause. Release either button to stop; a quick tap still changes one step.

The setting is named **Color temp** in both Screen and Night mode. Neutral preserves the original picture; increasing it adds a warmer tint, up to Warm 100%. In Screen it adjusts the active day or night preset. Inside Night mode it adjusts the saved night preset, applying immediately when night mode is active. The night preset starts at **22% brightness** and Warm 100%; the day preset starts with your existing display settings.

## Switch automatically

First set the clock in **Menu → General → Date & Time**. Then open **Menu → Screen → Night mode** from the carousel.

| Setting | What it does |
| :--- | :--- |
| **Automatic** | Turn scheduled switching on or off. It is off by default. |
| **Day begins** | Set when the saved day brightness and warmth take effect. |
| **Night begins** | Set when the saved night brightness and warmth take effect. |
| **Color temp** | Change the saved night warmth. It applies immediately if the night preset is active. |

Use Left/Right on a start time to change it by 15 minutes. Press A to edit Hours and Minutes separately. Day and night need different start times. Times follow the device clock and your chosen 12-hour or 24-hour format.

The shortcut still works while Automatic is on. Your manual choice lasts until the next scheduled change, including across sleep and restart. Changing the clock or schedule clears that manual override.

## Change the shortcut

Open **Menu → Device controls → Shortcuts → Night mode**, press <kbd>A</kbd> and record your preferred combination. Restoring the page's defaults brings back <kbd>SELECT</kbd> + <kbd>X</kbd>.

<details>
<summary><strong>Where the presets are saved</strong></summary>

The presets are stored in `Config/display.txt`. The menu and shortcuts update this file for you.

</details>

---

[Appearance & screen](DISPLAY.md) · [Buttons & shortcuts](CONTROLS.md)
