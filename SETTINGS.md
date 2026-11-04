# Settings reference

Tap <kbd>MENU</kbd> on the carousel to open the settings grid. During a game, tap <kbd>MENU</kbd> and choose **Settings** to open the same settings grid. Changes save as you make them.

Use the D-pad to select a tile, <kbd>A</kbd> to open its card and <kbd>B</kbd> to return. Inside a card, Up/Down chooses a row; Left/Right changes its value. Follow the on-screen hints for editors and confirmations.

| Tile | Use it for | Instructions |
| :--- | :--- | :--- |
| **General** | Device code, Updates, Automatic time, Time zone, Recents, Favorites order, clock format, date/time and About. | [Clock & time](POWER-OPTIONS.md#set-the-clock) · [Device code](#device-code) |
| **Display** | Brightness, screen warmth, night schedules, UI theme, wallpaper, SP color, game names and carousel motion, plus screen effects, palettes and borders for each system. | [Screen](DISPLAY.md#screen-effects--borders) · [Day & night](NIGHT-DISPLAY.md) |
| **Sound** | Choose an audio output, pair headphones, set volume after a disconnect and optionally pause when audio disconnects, and toggle carousel sounds. | [Audio & headphones](BLUETOOTH.md) |
| **Gameplay** | Automatic resume and saving on exit, fast-forward speed and sound, rewind, turbo and rumble where supported. RetroAchievements is the first entry. | [Playing & saving](PLAYING.md) |
| **Power & lid** | Lid behavior, sleep, shutdown timer and startup animation. | [Sleep & power](POWER-OPTIONS.md) |
| **Device controls** | Game buttons, carousel buttons, shortcuts and the on-device reference. | [Buttons & shortcuts](CONTROLS.md) |
| **Parental controls** | Approved games, daily allowance, session timer or bedtime, maximum volume and either a separate parent code or the device code. | [Kid mode](KID-MODE.md) |
| **Network** | Connect and scan networks after setting and unlocking the Device code. | [Wi-Fi & achievements](RETROACHIEVEMENTS.md) |

![The complete settings home](previews/current/light/settings.png)

## Device code

**Device code** is first in **General**. Choose a button sequence of six or more presses (Start finishes setup) or a keyboard passphrase of at least twelve characters. Credentials are always encrypted. Wi-Fi and RetroAchievements stay disabled without a code; existing shorter codes must be upgraded before network access. Six- and seven-button codes get a short warning because longer unpredictable secrets are stronger.

**Startup access** can require the code to use the launcher or allow games while
credentials stay locked. Skipping optional startup unlock shows “Saved credentials are locked.” The SD card and games remain accessible; only credentials are encrypted.
Parental controls can use this code or a separate four-button parent code.
See [Code setup, storage and reset](RETROACHIEVEMENTS.md#device-code-and-saved-credentials).

About is also under General; press <kbd>X</kbd> there to restart the launcher.

## During a game

Settings float over the game screen. Screen opens display settings for the current system; open **Display → Night mode** from the carousel to edit the automatic schedule. Tap <kbd>MENU</kbd> for **Resume**, **Settings** and **Exit game**. **Save states**, **Screenshots** and undo choices appear when available.

In kid mode, **Settings** opens four tiles: Sound, Screen, Buttons & shortcuts and Parental controls. See the [kid menu](KID-MODE.md#the-kid-menu).

## Light, dark, day and night

**Display → UI theme** changes the interface palette. **Display → Night mode** changes saved brightness and warmth settings, with an optional schedule. They are independent.

## Status icons and notices

The battery indicator shows the percentage. A lightning bolt inside it means the charger is connected. A slashed speaker means the active output is muted or its volume is zero. Bluetooth and headphones icons show the audio connection. The Wi-Fi icon appears alongside them while connected. A controller marks background RetroAchievements work. Yellow triangle pills warn about short codes. Wi-Fi and RetroAchievements require an encrypted vault and a code of at least six button presses (or a passphrase).

Connecting a charger briefly shows a charging notice and plays a tone. Short notices also report audio connections, setting changes and errors. An error includes a next step, such as waking the headphones and trying again.

The clock, status icons and button hints change between light and dark colors to stay readable over custom wallpapers. These color changes do not change your selected UI theme.

## About

![The About sticker with project, artwork and typeface credits](previews/current/light/about.png)

**Updates** needs a Wi-Fi connection. **Automatic time** corrects UTC in the background and disables **Date & Time** while it is on. You can turn it off while disconnected. **Time zone** still applies your local UTC offset, including any daylight-saving adjustment.

## Network game uploads

On a trusted Wi-Fi network, open **Settings → Network → Network file sharing** and turn **Sharing** on. Select **Password** to show the generated sharing password. Sign in with username **slot** and that password.

- **macOS:** in Finder, choose **Go → Connect to Server** (⌘K), enter the `smb://` address shown on the handheld and select **Registered User**. Choose **Games**, **Labels**, or **Wallpapers**. To open Games directly, append `/Games` to the address.
- **Windows:** open **SLOTSP** in File Explorer's Network view, or enter `\\DEVICE-IP\Games`.
- **Linux:** open the discovered SMB server or enter `smb://DEVICE-IP/Games` in your file manager.

These shares write directly to the corresponding folders on the content card. Use **GB**, **GBC**, or **GBA** under **Games** and **Labels**. Finish copying and turn sharing off before launching a game or opening Cart Studio. The carousel refreshes when sharing stops. Changes and deletions through SMB apply to the card immediately. If you have files left in the old Inbox, choose **Import old Inbox** once to move them into the library.

Sharing requires authenticated, encrypted SMB3. Saves, Config, System and the rest of the card stay outside the shares. **New password** disconnects existing sessions. Devices must be able to reach each other on the local network.

Sharing turns **off after reboot by default**. **Keep on after reboot** can restore it after the Device code unlocks your vault and the same saved Wi-Fi network reconnects. A short warning pill appears when enabling this option. Network changes revoke sharing; it is also paused during gameplay, Cart Studio, link play, kid mode, sleep and shutdown. A dropped connection requires enabling it again. Use a trusted home network; the saved network name does not identify a router cryptographically.

The build includes Samba and its private runtime; it does not require installing a different BaseOS. The matching **slot.sp-network-sources.zip** download contains corresponding source and dependency provenance.


### Wi-Fi connection progress and diagnostic reports

After Wi-Fi joins the router, **Getting an IP address…** keeps the progress
indicator active while DHCP finishes. This can take ten seconds or more. A
usable IPv4 address completes the connection; after 30 seconds without one,
slot shows **No IP address · retry**. A late address
still recovers automatically. Updates stay unavailable until the address arrives.

If connecting fails, leave the device on the affected network and choose
**Settings → Network → Save Wi-Fi report**. This is the final row; from the
Wi-Fi toggle you can press Up to reach it. Wait for **Saved Diagnostics/wifi-NNNN.txt**,
then shut down normally and read that file from the **Diagnostics** folder on
the slot SD card using your computer. Attach the newest file to the issue along
with your handheld model and what happened. Collection takes at most about
12 seconds for command probes, plus the SD write; it works without Internet.

Reports contain build identity and fixed status fields for association, IP
presence, routing, resolver configuration and DHCP/watcher processes. They
exclude network names, IP/MAC addresses, passwords, account tokens and raw logs.
Nothing is uploaded automatically, no connection settings are changed, and each
export creates a new file. Delete reports from the card when no longer needed.
