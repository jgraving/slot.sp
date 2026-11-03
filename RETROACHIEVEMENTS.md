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

**General → Security PIN** is the first General setting. It sets the device PIN used
to unlock saved Wi-Fi and RetroAchievements credentials. Choose **Button code** or
**Passphrase**, then **Set Security PIN**:

- **Button code:** enter four or more presses using A/B/X/Y, the D-pad and shoulder
  buttons. You choose the length; press **Start** to finish, then repeat the sequence
  and press Start again. Select deletes a press; Menu cancels. Held buttons do not
  repeat. Only masked dots and the number of presses are shown, even for long PINs.
- **Passphrase:** use the keyboard to enter at least 12 characters, then repeat it.
  Choose a long, unpredictable sequence or passphrase for stronger protection.

PINs shorter than eight button presses get a yellow **!** notification because they
are easier to guess. There is no recovery copy of the device PIN on the card.

**Credential storage** is an inline Left/Right picker shared by Wi-Fi and
RetroAchievements:

- **Encrypted** is the default for a new setup. Credentials are saved under the
  device PIN. If one is needed, the Security PIN page opens directly with a pill
  explaining that this sets a device PIN for encrypted credentials.
- **Session only** keeps credentials in memory and forgets them at power-off.
- **Unencrypted** saves readable credentials on the SD card. A red **!** warning
  explains this. Nothing is written until you select the confirmation row and
  press **A**. The red warning also appears after confirming the change.

Switching from Unencrypted to Encrypted saves the encrypted copy first, deletes the
plaintext file, and shows a pill confirming deletion. If saving or deletion fails,
the change reports the error instead of claiming success. Deleting a file on flash
storage is not a guarantee of forensic erasure of older data.

After a restart with either feature enabled and encrypted credentials saved, an
animated unlock card appears. One successful unlock covers both features for that
session and resumes the requested connection or account action.

**Startup access** under Security PIN chooses between **Allow games without PIN**
(the default) and **Require PIN**. With games allowed, Menu skips startup unlock and
a pill says Wi-Fi and RetroAchievements are unavailable until unlocked. With Require
PIN, the launcher stays on the unlock card until the correct PIN or passphrase is
entered, even when Wi-Fi and achievements are off. This is a launcher access lock:
the SD card, games and saves remain readable. Only the saved credentials are encrypted.
The startup lock does not protect against modified software or physical tampering.

Switching to Session only asks before deleting saved credentials; the PIN and current
session are kept. **Forget Security PIN** removes the vault and saved credentials
after confirmation. If parental controls use the device PIN, choose a separate parent
code first. Pending achievements and game saves are kept.

The key is never stored on the card. Argon2id derives it from the entered secret;
authenticated ChaCha20-Poly1305 encrypts the vault. A short PIN is a convenience lock:
four presses from 12 buttons give only 20,736 combinations, which a person with a
copy of the card can try offline. Longer unpredictable secrets make guessing harder.
Neither option protects credentials on a device that is already running and unlocked.
The supplicant receives passwords through its control socket and does not save them.
A Wi-Fi status icon appears beside the audio indicators while actually connected.

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
