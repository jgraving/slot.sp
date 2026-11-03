# Wi-Fi and RetroAchievements test build

This branch adds optional Wi-Fi and GBA achievement tracking. Both start off.
About moves to General; Network takes its existing grid square. RetroAchievements is
the first entry under Gameplay. The grid still has eight entries.

## Connect

Open Settings → Network. Turn Wi-Fi on, select a nearby WPA/WPA2 personal or open
network, and enter its password. Other network lets you enter a hidden SSID.
Enterprise networks and WPA3-only networks are not supported in this test build.

Use the D-pad to select keyboard keys and A to type. B deletes, X shifts, Y adds a
space, L1/R1 switches symbols, Select reveals a secret, Start submits and Menu cancels.

## Security PIN and saved credentials

Network is one menu: On/Off, Reconnect, Credential storage, Scan networks, Other network,
then nearby networks. Each network has a signal icon and its detected security type.
Scanning uses the same progress indicator as Bluetooth. WPA3-only, enterprise, WEP
and enhanced-open networks are shown but cannot be joined in this build.

**General → Security PIN** is the first General setting. It sets the device PIN used
to unlock saved Wi-Fi and RetroAchievements credentials. Choose **Button code** or
**Passphrase**, then **Set Security PIN**:

- **Button code:** enter six or more presses using A/B/X/Y, the D-pad and shoulder
  buttons. You choose the length; press **Start** to finish, then repeat the sequence
  and press Start again. Select deletes a press; Menu cancels. Held buttons do not
  repeat. Only masked dots are shown, even for long PINs. During unlock, a correct sequence is accepted automatically without Start. Incorrect prefixes stay silent and remain editable; continue typing or use Select to delete. No stored PIN length is displayed.
- **Passphrase:** use the keyboard to enter at least 12 characters, then repeat it.
  Choose a long, unpredictable sequence or passphrase for stronger protection.

PINs shorter than eight button presses get a yellow triangle notification because they
are easier to guess. There is no recovery copy of the device PIN on the card.

Wi-Fi and RetroAchievements require an encrypted vault protected by the device PIN. There are no plaintext or session-only storage choices. Set and unlock the PIN before enabling either service. Existing four- or five-button PINs still decrypt the vault, but must be replaced with six or more presses before network services start. Long unpredictable secrets are stronger than short patterns.

Old plaintext files from previous builds are imported only after PIN creation or authentication. The encrypted save must succeed before the old file is deleted. Failure keeps network services off and reports the problem. Deletion on flash is not forensic erasure.

After a restart with either feature enabled and encrypted credentials saved, an
animated unlock card appears. One successful unlock covers both features for that
session and resumes the requested connection or account action.

**Startup access** under Security PIN chooses between **PIN optional**
(the default, allowing games without a PIN) and **Require PIN**. With games allowed, Menu skips startup unlock and
a pill says “Saved credentials are locked.” Saved services become available after unlocking. With Require
PIN, the launcher stays on the unlock card until the correct PIN or passphrase is
entered, even when Wi-Fi and achievements are off. This is a launcher access lock:
the SD card, games and saves remain readable. Only the saved credentials are encrypted.
The startup lock does not protect against modified software or physical tampering.

**Forget Security PIN** removes the vault and saved credentials
after confirmation. If parental controls use the device PIN, choose a separate parent
code first. Pending achievements and game saves are kept.

The key is never stored on the card. Argon2id derives it from the entered secret;
authenticated ChaCha20-Poly1305 encrypts the vault. A short PIN is a convenience lock:
six presses from 12 buttons give only 2,985,984 combinations, which a person with a
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

### PIN entry and feedback

Button PINs accept A, B, X, Y, the four D-pad directions and L1, R1, L2, R2.
Start finishes setup and confirmation; Select deletes, and Menu cancels when startup access allows it.
Unsupported input briefly shows the permitted button icons. The separate four-button parent code uses
the same masked card as the device PIN. It completes after four presses; the device PIN has no fixed length.

Yellow triangle pills indicate a caution, such as a short PIN. Red circled alerts indicate a more serious
condition, such as credentials saved without encryption. Short messages stay in the interface; this guide
explains the consequences. The wider Wi-Fi arcs show signal strength in network lists and a connection
in the status bar.

## Preparation and background sync

Signing in and preparing games are separate steps. The first pass checks each GBA ROM in `Games/GBA`, caches its achievement rules and server unlocks, and downloads badge artwork. A library of hundreds of games can take a long time: work is deliberately paced and requests can time out. The game-data and artwork phases show their own counts; artwork totals grow as games are discovered. Opening Settings does not restart preparation.

The controller in the status bar means RetroAchievements is working in the background. The settings page shows game data, artwork, pending uploads and games that need another attempt. A connection or account error shows retry status rather than pretending preparation is still moving. Preparation continues on the shelf and during play while slot is running. It pauses when the device sleeps, shuts down, or loses internet; downloaded data and earned unlocks persist. Signing out or turning achievements off stops the worker. A restart uses cached data and prepares remaining or changed games; it also refreshes old data weekly. Newly played games take priority over the background library.

Completed game preparation makes cached tracking available offline. It does not mean every game has achievements, and it is separate from whether the server accepts deferred unlock uploads.
