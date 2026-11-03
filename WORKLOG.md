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

## Active follow-up: simpler UI (2026-11-03)

User requested:
- Short warning text with graphical severity icons. Skipped unlock notice should describe locked credentials generally, without naming services.
- Remove persistent button-PIN explanations; show allowed button icons in a notification only on invalid input.
- Filled volume, brightness, color-temperature and kid-time HUD pills; adaptive foreground contrast as fill changes. Keep these at the top and keep settings sliders unchanged.
- Settings label "Color temp." or "Color temperature" if it fits. Preview color temperature while its night-display slider is selected; restore configured scheduling behavior when leaving it.
- Ensure in-game screenshot/gallery second-row items are easy to reach with the D-pad.
- Rename Wi-Fi submenu to Network.

Current step: recover a local checkout and inspect rendering/input paths. No runtime edits yet. Continue existing PR #186; do not merge or release.

### Follow-up scope and implementation checkpoint

- Local clone recovered successfully; work is now in the actual repo checkout.
- Added user steering: silently auto-check button PIN prefixes from four presses onward; correct code continues immediately, failed checks preserve entry and never disclose stored length. Keep setup confirmation explicit.
- Remove small explanatory copy from settings cards; use brief contextual notices and keep details in the website guide.
- Wi-Fi artwork should retain radio arcs (user reversed the solid-shape request), but be wider and cleaner.
- Implementation plan: shared filled-meter artwork with contrast-aware text/icons, asynchronous prefix verification that accepts input while checking, focus-only warmth preview without writing temporary values to state, and spatial D-pad navigation for incomplete grid rows.
- Source audit found the grid currently ignores vertical moves from a column absent on the next row. Fix by selecting the nearest visible tile in that row.
- No Rust toolchain is installed in this environment. Build, regression tests and production native captures will run through existing CI on the pushed revision.

### Implementation ready for CI

- Shared filled-capsule artwork now covers volume, brightness, warmth and remaining kid time. Foreground contrast follows the animated fill boundary; settings sliders remain sliders. HUD capsules stay at the top.
- Added graphical warning triangle / circled alert and shorter security feedback. Invalid PIN buttons trigger a transient row of permitted button icons.
- System PIN unlock checks on a worker while input stays responsive; failed checks preserve input without errors/spinners, and successful checks continue automatically. Rapid typing coalesces to the newest complete input after the in-flight check rather than accumulating expensive Argon2 jobs. Cancellation/deletion invalidates stale successful results.
- Separate parent code now uses the shared masked-dot card. Its existing four-button semantics remain.
- Removed persistent PIN/security/confirmation explanations and binding captions; binding capture gets a short contextual pill. Details moved into the guides.
- Night warmth preview is a read-only compositor override while the slider is focused; it cannot write temporary warmth into day settings or override scheduling.
- Incomplete grid rows now accept vertical D-pad movement from every column, choosing the nearest visible tile. Network naming and wider antialiased Wi-Fi arcs applied.
- Added/updated regression coverage for async PIN editing/cancellation, complete grid-row navigation, severity symbols, meter geometry/contrast, native preview generators and warmth restoration.
- Next: push implementation through GitHub connector (native Git push lacks authentication here), run existing CI, fix failures, inspect fresh native captures, then update PR/test-build link. Runtime changes not yet validated by Rust compiler in this environment.

### Audio steering and first CI checkpoint

- UI implementation pushed as 6d1132e46ba99420d639c7d67eb27143ef32ecb5. All six existing workflows started; results pending.
- User additionally requested manual output selection restore the destination's saved volume, rather than applying Volume on disconnect.
- Found two causes: Bluetooth treated a cleared manual route as a lost device, and USB applied disconnect volume even when unplugged=false. Limited fallback-volume handling to actual connection loss/removal; added regression for speaker/headphone/USB/Bluetooth levels and mute restoration.
- Parent PIN cards now share the system PIN layout, per additional user steering. Guide and test labels updated.
- Next: publish audio correction and inspect compile/test outcomes. Keep PR draft.

### First CI result

- Runtime code compiles. Initial regression run passed 132 of 133 library tests; the only failure still asserted the old requirement that unlock waits for Start.
- Updated that raw-input regression to require automatic continuation without Start. Async PIN editing/cancellation and existing credential-storage tests passed in the same run.
- Audio manual-route correction pushed as 9e50275b45a0721fbd695f8d1fc80ef5d6895346; checks on that revision started. Next push includes the updated stale test expectation.

### Regression follow-up

- The next run passed 133/134 library tests. The remaining audio test manually set speaker volume but left its previous disconnect mute flag set; corrected its fixture to model an actual unmuted adjustment. The new explicit route-volume/mute regression passed.
- Strengthened automatic PIN checking: newest input is prioritized, with intermediate typed prefixes retained for silent verification. This ensures a correct PIN cannot be skipped if extra presses arrive while Argon2 is running. Still one bounded worker request at a time; stale results are ignored after cancellation/deletion.
- Added a regression for fast input containing the correct PIN followed by extra presses while checking. Shortened deletion warnings using “unencrypted copy” rather than jargon.
- Next: rerun all required checks, then inspect native output. All changes remain in draft PR #186.

### Parent PIN visual regression checkpoint

- 5824739 passes all 134 slot library tests, including automatic PIN prefixes/cancellation, storage failure paths and manual audio destination volume/mute.
- The broader run reached parent UI coverage and caught one pixel test still sampling the old 64px button-keycap slots. Updated it to sample the shared 32px masked-dot positions, and explicitly compare the parent/system PIN card bodies.
- No runtime change needed for that failure. Next CI run should continue into the UI meter/notice tests and native renders.

### Meter test cleanup

- Static review found a toast regression comparing draw-command counts. A filled capsule uses fewer quads than the old slider, so replaced that assertion with the actual contract: an ordinary toast suppresses the top meter.
- Corrected remaining guide references from punctuation warnings to the new graphical alerts. Runtime source is unchanged from 5824739; all 134 library tests already pass on that implementation.

### Settings-overlay coverage

- The broader scaling run passed parent-code coverage and reached an old settings test expecting a standalone HUD icon texture. Updated its fixture to supply the new empty/filled capsule textures and assert that those draw over settings.
- Runtime source remains unchanged from 5824739. Latest checks are now exercising the filled-meter UI and full native captures; no hardware/live-account claims are made.
