# 🙂 Kid mode

[← Overview](README.md) · [All guides](GUIDE.md)

[Set up](#set-it-up) · [Kid menu](#the-kid-menu) · [Limits](#time--volume-limits) · [Approvals](#approve--remove-games) · [Parent code](#the-parent-code)

Choose which games children can play, set a time allowance and cap the volume. Children keep the regular system, Favorites and Recents carousels, filtered to the games you approve.

## Set it up

1. Set the handheld's clock in **Menu → General → Date & Time**.
2. Open **Menu → Kid Mode**.
3. Select **Approved games**. Press <kbd>A</kbd> on each game you want to allow, then <kbd>B</kbd> to return.
4. Choose a **Play limit**, set the **Time**, and adjust the **Volume limit**.
5. Turn **Kid mode → On**.
6. If you have not set a parent code, enter four buttons, press <kbd>START</kbd>, then enter the same four buttons again and press <kbd>START</kbd> to confirm.

You can also set the code separately through **Parent code → Set**. Kid mode starts off on a new installation.

## The kid menu

During a game, tapping <kbd>MENU</kbd> opens:

| Choice | What children can do |
| :--- | :--- |
| **Resume** | Return to the game. |
| **Volume** | Adjust volume within the parent's cap. |
| **Brightness** | Adjust screen brightness. |
| **Exit game** | Return to the carousel through the normal save-and-exit process. |
| **Parent controls** | Enter the parent code to change approvals or limits. |

Use <kbd>Up</kbd> / <kbd>Down</kbd> to select a row and <kbd>Left</kbd> / <kbd>Right</kbd> to adjust a value. <kbd>A</kbd> selects; <kbd>B</kbd> or <kbd>MENU</kbd> closes the menu.

On the carousel, the kid menu offers **Games**, Volume, Brightness and Parent controls. Children can browse, play and leave games without entering a code. Parent dialogs pause and mute the game.

## Time & volume limits

| Parent setting | How it works |
| :--- | :--- |
| **Play limit → Daily** | Counts active gameplay against an allowance for the day. Pauses and sleep do not use that allowance. Usage is kept across restarts and renews on the next calendar day. |
| **Play limit → Duration** | Starts a timed session when enabled or when its time settings change. Sleep and reboot do not extend the deadline. |
| **Play limit → Bedtime** | Allows play until the chosen time today. |
| **Time** | Daily/Duration: 5–240 minutes, in five-minute steps. Bedtime: a 24-hour clock, in 15-minute steps. |
| **Volume limit** | 10–80%, in ten-point steps. Starts at 50% and applies across audio outputs while kid mode is on. |

Settings save as you change them. Changing the volume cap does not restart the session timer. Normal parent adjustments keep an approved game paused in place, ready to resume.

When time is up, play stops and the carousel asks for a parent. Choose **Parents**, enter your code and adjust the time settings to allow more play.

## Approve & remove games

In **Parent controls → Approved games**, each game shows **Yes** or **No**. Press <kbd>A</kbd> to approve a game. To remove approval, press <kbd>A</kbd>, release it and press it again within three seconds. Moving away or waiting cancels the confirmation.

With kid mode off, <kbd>X</kbd> on a cartridge also toggles approval. Removal uses the same confirmation pattern. A **gold smiley at the top left** marks approved games.

Your full Favorites and Recents lists are kept. Turning kid mode off shows them again.

## The parent code

Use four presses from <kbd>A</kbd>, <kbd>B</kbd>, <kbd>X</kbd>, <kbd>Y</kbd> or the D-pad. Each press shows its button picture on screen.

| Button | Action |
| :--- | :--- |
| <kbd>A</kbd> / <kbd>B</kbd> / <kbd>X</kbd> / <kbd>Y</kbd> / D-pad | Enter a button in the code. |
| <kbd>SELECT</kbd> | Delete the last entry. |
| <kbd>START</kbd> | Submit the four entries. |
| <kbd>MENU</kbd> | Cancel code entry. |

To change the code, open **Parent controls**, unlock it and choose **Parent code → Change**. Enter and confirm the new code.

### Turn kid mode off

Open **Menu → Parent controls**, enter the code and change **Kid mode → Off**. Press <kbd>B</kbd> to return. There is no separate Apply step.

### Recover a forgotten code

Shut down and open the content card on a computer. Read:

```text
Config/kids/parent-code.txt
```

The recovery file contains the four buttons in plain text. Keep access to the card with the parent who manages the code.

---

[Buttons & shortcuts](CONTROLS.md) · [Games & artwork](LIBRARY.md) · [Audio & headphones](BLUETOOTH.md)
