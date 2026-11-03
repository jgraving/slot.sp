# Wi-Fi and RetroAchievements test build

This branch adds optional Wi-Fi and GBA achievement tracking. Both start off.
About moves to General; Wi-Fi takes its existing grid square. RetroAchievements is
under Gameplay. The grid still has eight entries.

## Connect

Open Settings → Wi-Fi. Turn Wi-Fi on, select a nearby WPA/WPA2 personal or open
network, and enter its password. Other network lets you enter a hidden SSID.
Enterprise networks and WPA3-only networks are not supported in this test build.

Use the D-pad to select keyboard keys and A to type. B deletes, X shifts, Y adds a
space, L1/R1 switches symbols, Select reveals a secret, Start submits and Menu cancels.

Credential storage offers two choices:

- **This session only** is the default. Wi-Fi and account credentials stay in memory
  and are forgotten on power-off. Nothing connects automatically after restarting.
- **Save encrypted** requires a passphrase of at least 12 characters, entered twice.
  After restarting, unlock it from Credential storage before using the saved account
  or network. Then turn Wi-Fi on and select the network. The key is never stored on
  the card. A forgotten unlock passphrase cannot be recovered; forget the saved
  credentials and enter them again.

Both settings screens use the same encrypted credential file. The main preferences
file contains only the achievement enable switch. The supplicant receives passwords
through its Unix control socket; it never saves networks to the card. The vault uses
Argon2id and authenticated ChaCha20-Poly1305 encryption. This protects a removed card
against someone without the passphrase; it does not protect an already unlocked,
running device. Choose a strong passphrase, not a short PIN.

Link play takes ownership of the radio and turns home Wi-Fi off. Reconnect manually
after link play. Cached achievements can continue during gpSP link play, but the
mGBA two-console cable path is excluded.

## Set up achievements

Open Settings → Gameplay → RetroAchievements. Enter the account username and
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
account by unlocking encrypted credentials after reboot, or enter the account again
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
