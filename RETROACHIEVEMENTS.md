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

## Device code and saved credentials

Network is one menu: On/Off, Reconnect, Credential storage, Scan networks, Other network,
then nearby networks. Each network has a signal icon and its detected security type.
Scanning uses the same progress indicator as Bluetooth. WPA3-only, enterprise, WEP
and enhanced-open networks are shown but cannot be joined in this build.

**General → Device code** is the first General setting. It sets the device code used
to unlock saved Wi-Fi and RetroAchievements credentials. Choose **Button code** or
**Passphrase**, then **Set Device code**:

- **Button code:** enter six or more presses using A/B/X/Y, the D-pad and shoulder
  buttons. You choose the length; press **Start** to finish, then repeat the sequence
  and press Start again. Select deletes a press; Menu cancels. Held buttons do not
  repeat. Only masked dots are shown, even for long codes. During unlock, a correct sequence is accepted automatically without Start. Incorrect prefixes stay silent and remain editable; continue typing or use Select to delete. No stored code length is displayed.
- **Passphrase:** use the keyboard to enter at least 12 characters, then repeat it.
  Choose a long, unpredictable sequence or passphrase for stronger protection.

codes shorter than eight button presses get a yellow triangle notification because they
are easier to guess. There is no recovery copy of the device code on the card.

Wi-Fi and RetroAchievements require an encrypted vault protected by the device code. There are no plaintext or session-only storage choices. Set and unlock the code before enabling either service. Existing four- or five-button codes still decrypt the vault, but must be replaced with six or more presses before network services start. Long unpredictable secrets are stronger than short patterns.

Old plaintext files from previous builds are imported only after code creation or authentication. The encrypted save must succeed before the old file is deleted. Failure keeps network services off and reports the problem. Deletion on flash is not forensic erasure.

After a restart with either feature enabled and encrypted credentials saved, an
animated unlock card appears. One successful unlock covers both features for that
session and resumes the requested connection or account action.

**Startup access** under Device code chooses between **Code optional**
(the default, allowing games without a code) and **Require code**. With games allowed, Menu skips startup unlock and
a pill says “Saved credentials are locked.” Saved services become available after unlocking. With Require
code, the launcher stays on the unlock card until the correct code or passphrase is
entered, even when Wi-Fi and achievements are off. This is a launcher access lock:
the SD card, games and saves remain readable. Only the saved credentials are encrypted.
The startup lock does not protect against modified software or physical tampering.

**Forget Device code** removes the vault and saved credentials
after confirmation. If parental controls use the device code, choose a separate parent
code first. Pending achievements and game saves are kept.

The key is never stored on the card. Argon2id derives it from the entered secret;
authenticated ChaCha20-Poly1305 encrypts the vault. A short code is a convenience lock:
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
account by entering the shared Device code after reboot, or enter the account again
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

### code entry and feedback

Button codes accept A, B, X, Y, the four D-pad directions and L1, R1, L2, R2.
Start finishes setup and confirmation; Select deletes, and Menu cancels when startup access allows it.
Unsupported input briefly shows the permitted button icons. The separate four-button parent code uses
the same masked card as the device code. It completes after four presses; the device code has no fixed length.

Yellow triangle pills indicate a caution, such as a short code. Red circled alerts indicate a more serious
condition, such as credentials saved without encryption. Short messages stay in the interface; this guide
explains the consequences. The wider Wi-Fi arcs show signal strength in network lists and a connection
in the status bar.

## Preparation and background sync

Signing in and preparing games are separate steps. The first pass checks each GBA ROM in `Games/GBA`, caches its achievement rules and server unlocks without downloading every achievement badge. Game-data requests remain paced; a large library or slow connection can still take time. Opening Settings does not restart preparation.

The controller in the status bar means RetroAchievements is working in the background. The settings page shows game data, optional artwork, pending uploads and games that need another attempt. A connection or account error shows retry status rather than pretending preparation is still moving. Preparation continues on the shelf and during play while slot is running. It pauses when the device sleeps, shuts down, or loses internet; downloaded data and earned unlocks persist. Signing out or turning achievements off stops the worker. A restart uses cached data and prepares remaining or changed games; it also refreshes old data weekly. Newly played games take priority over the background library.

Completed game preparation makes cached tracking available offline. It does not mean every game has achievements, and it is separate from whether the server accepts deferred unlock uploads.

### Reading background sync

The controller stays visible while game data, enabled artwork or unlock uploads remain queued, including retry waits. It disappears when no work remains. Settings shows a stable sync state, separate game/artwork counts, and a retry countdown with its cause. Completed cached games still count after a restart; an empty GBA library says “No GBA games”. An artwork retry does not mean Wi-Fi disconnected or that cached achievements stopped working. Badge failures back off from 30 seconds to five minutes and respect longer server Retry-After instructions (including HTTP dates); other sync work can continue. Preparation is paced to reduce background load, but a large first scan still reads ROMs and uses the radio.

For optional live testing, add `RA_TEST_USERNAME` and `RA_TEST_PASSWORD` in GitHub Settings → Secrets and variables → Actions. Prefer a dedicated test account. Run **RetroAchievements live smoke test** manually on a reviewed repository branch. It checks login, a read-only game lookup and a public badge download/decode; it never creates a game session or submits awards. Normal pull-request tests use fake servers and do not receive those secrets. Live smoke tests cannot measure handheld radio reliability, CPU load or temperature.

Date & Time changes made in Settings apply immediately; Start/Done only closes the page. Changing the UTC offset changes local display time while preserving UTC. First-boot clock setup still uses Done to continue. The offset is a manual choice and does not automatically follow daylight-saving changes.

**Achievement artwork** in the RetroAchievements settings is optional and defaults to **Off**, including after updating an existing installation. It controls both badge downloads and badge images in unlock notifications; achievement rules, offline tracking and unlock uploads work with it off. Use **D-pad Left/Right** on the paired-chevron **Download for** selector (A does nothing) to choose **Current game**, **Favorites & recents** (plus the current game), or **Entire library**. The entire-library option is an explicit choice and can download thousands of badges. Favorites/recents changes are picked up during normal library rescans. Cached badges remain on the card when artwork is disabled and can be reused later.

Enabled artwork appears beside the achievement title, points and description in the unlock pill. Images are decoded off the UI thread, fitted without cropping or stretching, and reduced with alpha-aware filtering; small images are not enlarged. Missing/corrupt/uncached artwork falls back to a text notification immediately. A badge downloaded after an unlock is available for future notifications; earning achievements never waits for images.

Only the chosen scope is discovered and validated in bounded worker batches. Duplicate unchanged badges are not decoded for each referencing game. Successful badge downloads are serial and paced at one per second, independently of the three-second game-data queue. Failures retain backoff, and server Retry-After cooldowns survive reconnects and scope changes. Retry feedback distinguishes artwork-server, invalid-image and card-write failures. A retry countdown does not erase progress.

### Browse the current game's achievements

With RetroAchievements enabled and a GBA game loaded, open **Achievements** directly in the pause menu. It is hidden when the RetroAchievements toggle is Off or the game is unsupported. The existing **Settings → Gameplay → RetroAchievements → Browse current game** route is also available. The list uses that account's cached game definitions and combines server unlocks with locally earned unlocks waiting to upload, so it also works offline. No images or game data are downloaded just because the list is opened. Cached badges are reused; missing images show a lock or earned checkmark.

Use Up/Down to browse, Left/Right to filter All / Locked / Earned, and L/R to move a page. The expanded list shows five rows with padded game information. A opens the selected achievement's cached badge, full title, points, earned state and description; long descriptions scroll with Up/Down or L/R. Left/Right moves between achievements in the detail view. B returns to the list, then to the pause menu or RetroAchievements settings, depending on where you opened it. A game without cached data explains that it must be prepared online first; opening the browser never enables Wi-Fi or starts a game session.
