# Wi-Fi and RetroAchievements test build

This branch adds optional Wi-Fi and GBA achievement tracking. Both start off.
About moves to General; Wi-Fi takes its existing grid square. RetroAchievements is
the first entry under Gameplay. The grid still has eight entries.

## Connect

Open Settings → Wi-Fi. Turn Wi-Fi on, select a nearby WPA/WPA2 personal or open
network, and enter its password. Other network lets you enter a hidden SSID.
Enterprise networks and WPA3-only networks are not supported in this test build.

Use the D-pad to select keyboard keys and A to type. B deletes, X shifts, Y adds a
space, L1/R1 switches symbols, Select reveals a secret, Start submits and Menu cancels.

## Security PIN and saved credentials

Wi-Fi is one menu: On/Off, Reconnect, Credential storage, Scan networks, Other network,
then nearby networks. Each network has a signal icon and its detected security type.
Scanning uses the same progress indicator as Bluetooth. WPA3-only, enterprise, WEP
and enhanced-open networks are shown but cannot be joined in this build.

**General → Security PIN** sets the shared unlock secret for Wi-Fi and
RetroAchievements. Choose **Button code** or **Passphrase**, then **Set Security PIN**:

- **Button code:** enter 16 presses using A/B/X/Y, the D-pad and shoulder buttons,
  then repeat them. The last press submits automatically. Select deletes a press;
  Menu cancels. Held buttons do not repeat into the code. The dots hide your sequence.
- **Passphrase:** use the keyboard to enter at least 12 characters, then repeat it.
  A long, hard-to-guess passphrase gives stronger protection than a predictable code.

This Security PIN is separate from the four-button parental-controls code. No recovery
copy is written to the card. Remember it or keep it somewhere safe yourself.

**Credential storage** is an inline Left/Right picker in both Wi-Fi and
RetroAchievements. The choice is shared:

- **Session only** is the default. Wi-Fi passwords and account credentials stay in
  memory and disappear on power-off.
- **Encrypted** saves them under your Security PIN. Set the PIN in General first;
  choosing Encrypted without one shows that instruction. Creating a PIN alone does
  not enable credential saving.

After a restart with Wi-Fi or RetroAchievements enabled and encrypted credentials
saved, an unlock card appears after startup. One successful unlock covers both
features for that session. Wi-Fi reconnects after the code is accepted; there is no
second Connect press. Turning on Wi-Fi, choosing Reconnect, or enabling achievements
also prompts if the saved credentials are locked. Menu cancels the card and leaves
the connection waiting. Nothing connects using locked credentials.

Switching back to Session only asks before removing the saved credential copy. It
keeps your Security PIN and the current session. **General → Security PIN → Forget
Security PIN** removes the PIN and its saved credentials after confirmation; this is
also the reset path if you forget the code. Pending achievements and game saves are
kept. Enter the Wi-Fi/account details again afterward.

Both features share one encrypted file. Its key is never stored on the card. The
supplicant receives passwords through its Unix control socket and never saves them
to the card. The vault uses Argon2id and authenticated ChaCha20-Poly1305 encryption.
It protects a removed card against someone without your secret, not a device that
is already running and unlocked. A short PIN or an obvious repeated button pattern
is easier to guess; the button option requires 16 presses for this reason.

Link play takes ownership of the radio and turns home Wi-Fi off. Reconnect manually
after link play. Cached achievements can continue during gpSP link play, but the
mGBA two-console cable path is excluded.

## Set up achievements

Open Settings → Gameplay → RetroAchievements, the first Gameplay entry. Enter the account username and
password, enable achievements and choose Sign in / sync. A successful sign-in
replaces the account password in memory with the emulator login token. If encrypted
storage is unlocked, that token is saved encrypted too.

Tracking starts with the next inserted game. Reinsert a game after changing account
or enabling tracking. GB/GBC, hardcore mode and leaderboards are outside this draft.
Resume, save states, rewind and fast-forward remain available in casual/softcore mode.

Leave the handheld connected while the backend prepares the GBA library. The status
shows Preparing / syncing, then Up to date. Unsupported ROM hashes have no rules to
prepare. Firmware time must be correct for verified HTTPS; use General → Date & Time
if sign-in fails after a cold start. Certificate verification is never disabled.

Prepared games can earn achievements without internet. Unlocks are saved before their
notification appears, remain pending across restarts and upload later. Re-enable the
account by entering the shared Security PIN after reboot, or enter the account again
for session-only storage. Save files and local achievement caches are independent of
credential storage. Sign out and disabling tracking preserve pending unlocks.

**Deferred upload acceptance needs live testing.** RetroAchievements' official FAQ
still does not promise fully offline sessions. Local tracking and fake-server tests
are not proof of server acceptance. The implementation always submits softcore awards.

## Install and test

Download the `slot.sp-network-System` artifact from the PR's **Network and achievements
test build** workflow. Extract it, then extract `slot.sp-network-System.zip` and copy
its System folder over the card's System folder. Back up the card first. This is a
manual PR build, not a published release or automatic update.

Test a prepared game offline: trigger an achievement, power off normally, restart,
unlock the account, reconnect and check the account website. Also check normal game
launches with both features off, both GBA cores, rewind, state loading and link play.
Keep Saves/RetroAchievements when backing up: it contains pending unlocks.

The networking code needs the OS image's wpa_supplicant, wpa_cli, ip, rfkill and
udhcpc tools. Actual hardware association, DHCP, TLS clock behavior, real-core RAM
mapping, battery cost and live server acceptance are the main device checks.
