# Development recovery log

Last updated: 2026-11-03 (Europe/Zurich).
Repository: jgraving/slot.sp
Active branch: `feat/182-network-achievements`
Active PR: https://github.com/jgraving/slot.sp/pull/186
Implementation revision verified at takeover: `3f8c0d7eb240338e897e931a6d403268e9b95510`
Status: draft, not merged or released.

## Start here after a crash

1. Read this file and AGENTS.md, then inspect the live PR head, diff, and checks. Do not assume a previous chat's scratch files survive.
2. Continue on the existing feature branch unless the task requires otherwise. Do not repeat completed work.
3. Update this log before a substantial operation and after each meaningful result. Commit and push checkpoints so recovery does not depend on the current chat or local disk.
4. Record exact revisions, test results, failures, remaining work, and artifact links. Never record credentials, PINs, tokens, or private account information.
5. User's next-chat prompt: "Continue jgraving/slot.sp PR #186. Read WORKLOG.md and AGENTS.md on feat/182-network-achievements first."

## User requirements retained from the attached crashed-chat transcript

- "Convert to PDF" is ChatGPT UI text, not a request. Do not create a PDF.
- Security PIN first in General; shared by Wi-Fi and RetroAchievements.
- Variable-length button PIN (at least four presses), Start submits, repeat during setup. Keyboard passphrase is an alternative.
- Yellow ! pill for button PINs shorter than eight presses.
- Encrypted credential storage by default; picker also offers Session only and Unencrypted.
- Unencrypted storage requires explicit button confirmation and a red ! warning.
- Switching to encrypted storage must save encrypted data before removing plaintext and interrupted-write files; show deletion confirmation only when removal succeeds.
- Missing PIN opens Security PIN settings directly with the pill: "This sets a device PIN for encrypted credentials."
- Startup choice: require PIN to access launcher, or allow games with encrypted credentials locked. Skipping optional unlock warns Wi-Fi and RetroAchievements are unavailable.
- Launcher startup lock does not encrypt the SD card, games, or saves.
- Parent authentication may use a separate parent code or the device PIN; shared PIN must be verified freshly even when credentials are already unlocked.
- One Wi-Fi menu: inline credentials picker, scan spinner matching Bluetooth, icons and encryption labels beside network names.
- Connected Wi-Fi status icon alongside existing status indicators.
- RetroAchievements first in Gameplay.
- Display combines screen and UI appearance; Boot Animation in Power & lid; Carousel sounds in Sound.
- Guide must explain the choices and realistic security limits.

## Already implemented before takeover

The PR description at 3f8c0d7 records all requirements above as implemented, including failure handling and guide updates.
Credential vault: Argon2id + authenticated ChaCha20-Poly1305; PIN and derived key are not stored on the card.
Short PINs remain vulnerable to offline guessing. No secure-boot or physical-tamper protection claim.
Wi-Fi uses a supplicant control socket; passwords are not shell arguments or saved supplicant config.
GBA softcore RetroAchievements includes cached rules and account-separated offline ledgers. GB/GBC, hardcore, and leaderboards are out of this draft's scope.

Previous chat reported 128/129 library tests initially; the single parent-PIN failure was a test setup issue corrected before the final green run. A first-save plaintext directory creation issue was also fixed.

## Verified in this takeover

2026-11-03:
- Read the attached crashed-chat transcript; no PDF task exists.
- Read live PR metadata: open, draft, mergeable, head 3f8c0d7; base main at 8555f4721313dbeac384b6f656d10b9edda38697.
- Queried actual Actions results for 3f8c0d7: all six completed successfully.
  - Launcher quality of life checks: https://github.com/jgraving/slot.sp/actions/runs/41636192155
  - Network and achievements test build: https://github.com/jgraving/slot.sp/actions/runs/41636192211
  - Pages: https://github.com/jgraving/slot.sp/actions/runs/41636192136
  - Scaling: https://github.com/jgraving/slot.sp/actions/runs/41636192276
  - Blending: https://github.com/jgraving/slot.sp/actions/runs/41636192142
  - Shortcut guide: https://github.com/jgraving/slot.sp/actions/runs/41636192260
- Test build artifact verified present and not expired:
  https://github.com/jgraving/slot.sp/actions/runs/41636192211/artifacts/13231395464
  Name: slot.sp-network-System; implementation 3f8c0d7; created 2026-11-03 12:34 UTC.
  API expiry is 2026-11-10 12:34 UTC, but separate cleanup may remove it sooner. Recheck availability before sharing later.
- No existing AGENTS.md or recovery log found in the feature branch tree.
- Created WORKLOG.md in commit bcb7173205679294044bc56a823f946fc29f1281 and AGENTS.md in commit 6705fde707382d98e14238e5ce38a040ab0f0f59 on the feature branch.
- Inspected network.rs, network_vault.rs, and the production network_review walkthrough source at 3f8c0d7.
- Confirmed source paths for default encryption, variable-length PINs, short-PIN warnings, explicit plaintext confirmation, encrypt-before-delete migration, failure reporting, direct PIN settings navigation, skipped startup warning, mandatory startup lock, connected Wi-Fi indication, and fresh parent-PIN verification.
- Read the actual regression job log (job 125917901394): 131 slot library tests passed with zero failures; the named network and vault regressions all passed.
- Verified both native network walkthrough invocations completed: light GLES at 640x480 and dark desktop GL at 720x480; rendering and artifact upload steps succeeded.
- Independently downloaded the final native-review artifact and visually inspected 24 PNGs (12 screens in each theme): storage picker, connected status icon, network icons/security labels, short-PIN warning, plaintext confirmation and deletion, skipped startup unlock, required startup unlock, long PIN entry, shared parent method/prompt, and separate parent method.
- Reviewed captures show the requested controls and messages rendered in both themes. No additional implementation fix identified in this targeted review.
- Takeover changes are documentation only: WORKLOG.md and AGENTS.md. Existing test binary and runtime source remain at implementation revision 3f8c0d7.
- Development verification is complete; next action is user handheld/live-account testing using the linked build. Keep this log current when test feedback arrives.
- Current environment has no checked-out repo. Recovery documents are written directly to the existing GitHub branch through the connector, so they survive a chat crash.

## Remaining work and release gates

- Native-render job details, targeted source/test coverage review, and independent light/dark PNG inspection completed.
- Physical handheld tests remain: Wi-Fi association/DHCP, reconnect after reboot, required/optional PIN startup, both parent-code methods, warning pills, credential migration, frame pacing/battery cost, link-radio handoff.
- Live-account tests remain: login and acceptance of deferred/offline awards. Do not claim offline award acceptance is guaranteed.
- Final native GLES and desktop GL light/dark captures independently reviewed in this takeover. These are simulated-device renders, not evidence of real radio/network/account behavior.
- Keep PR draft for device testing. No merge or release requested in this takeover.
