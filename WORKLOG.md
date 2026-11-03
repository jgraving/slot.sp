# Development recovery log

Last updated: 2026-11-03 (Europe/Zurich).
Repository: jgraving/slot.sp
Active branch: `feat/182-network-achievements`
Active PR: https://github.com/jgraving/slot.sp/pull/186
Implementation revision tested after UI follow-up: `94f940e0e7c472b6b2f3ea9c506d4c8280f34c37`
Status: draft, not merged or released.

## Current state — active handheld follow-up

User authorized merging PR #186 and reported unclear RetroAchievements preparation progress (0%, 1%, 3%, then 2%, then 3%) with roughly 900 games. Before implementation:

- Trace preparation vs award sync, whether work survives leaving settings/gameplay, progress denominator/restarts, completion/failure and resumability. Explain verified behavior to user.
- Add a small controller status-bar indicator while background RA work is active; useful progress/count/status on RA settings with concise UI copy.
- Make settings/status-bar Wi-Fi arcs wider and lower.
- Investigate upstream OTA (including base OS) and automatic time; target jgraving/slot.sp releases, preserve custom setup. Keep upstream integration in a separate review PR; assess actual payload/install compatibility before enabling base OS updates.
- Merge existing tested PR #186 using the authorized current head, then implement follow-ups on new branch(es), test through CI and keep checkpoints pushed.
- No runtime work started in this follow-up yet. Prior completed state follows.

## Previous completed follow-up

All requested UI and audio changes are implemented and pushed. **All six CI workflows pass on 94f940e.** Native light/dark GLES and desktop GL walkthroughs pass, and the final warning, PIN, allowed-button hint, Network and filled-meter captures were visually reviewed.

- Test build: https://github.com/jgraving/slot.sp/actions/runs/41673591917/artifacts/13250356343
- Native captures: https://github.com/jgraving/slot.sp/actions/runs/41673591921/artifacts/13250905357
- Feature branch: `feat/182-network-achievements`; PR #186 remains draft, not merged or released.
- Next action: user handheld testing and live-account checks listed below. No known failing development check remains.
- Only recovery documentation has changed since the tested runtime/build revision 94f940e. Documentation checkpoints skip redundant CI. Compare that revision with the live PR head before resuming.
- Historical checkpoints follow. Treat this section as the latest state; do not repeat completed implementation/test work.

## Final verified behavior

- Short yellow warning-triangle / red circled-alert pills. Skipping unlock says “Saved credentials are locked.” Security and binding details live in the guide, rather than persistent small explanatory captions.
- Shared masked system/parent PIN cards. Variable button PIN unlock is automatic from four presses, with silent incorrect prefixes and responsive input. One worker request at a time; fast typing retains intermediate prefixes, and cancellation/deletion rejects stale results. Setup still uses Start to choose length and confirm. Finishing too early shows allowed button icons and “4+ presses” in a contextual pill.
- Filled volume/brightness/color-temperature capsules remain at the top; kid time is also filled. Foreground contrast changes at the animated fill boundary in both themes. Settings sliders remain unchanged.
- Color temp. label; focused Night mode warmth previews without persisting temporary day warmth, changing brightness or overriding scheduling. Leaving the row/menu restores the active setting.
- Vertical D-pad movement reaches incomplete pause-grid rows from every column. Network tile/submenu name and wider antialiased Wi-Fi arcs are in place.
- Manual speaker/wired/Bluetooth/USB output choices restore destination volume/mute. Disconnect volume is restricted to actual connection loss/removal.

## Final validation evidence

All results below are for 94f940e:

- Launcher tests and native captures: https://github.com/jgraving/slot.sp/actions/runs/41673591921 — success. Includes all 134 slot library tests and the complete UI/menu regressions.
- ARM64 test System package and both bundled cores: https://github.com/jgraving/slot.sp/actions/runs/41673591917 — success.
- Scaling: https://github.com/jgraving/slot.sp/actions/runs/41673591948 — success.
- Blending: https://github.com/jgraving/slot.sp/actions/runs/41673591886 — success.
- Shortcuts: https://github.com/jgraving/slot.sp/actions/runs/41673591899 — success.
- Website: https://github.com/jgraving/slot.sp/actions/runs/41673591994 — success. Build/links/desktop/phone navigation passed. The unchanged 3D preview's boot-frame observation initially timed out under software rendering; retrying only that job passed, with no website animation code change.
- Final artifact IDs: handheld 13250356343, native review 13250905357. Both were present and unexpired at completion; verify availability before linking later.
- Visual review covered short/red warning symbols, concise locked-credential feedback, clean shared PIN layouts, allowed-button hint, wider Wi-Fi arcs and two-tone meter text/icons in both themes. These are simulated-device renders, not physical hardware/network evidence.

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
- Initial takeover had no local checkout. A complete clone was subsequently recovered; changes and checkpoints are persisted on the GitHub feature branch. Scratch paths are not a durable handoff.

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

### Contextual button hint

- Pressing Start before four setup presses now shows the allowed controller icons with the brief “4+ presses” label in a transient pill. This makes the requested help reachable through actual handheld controls while leaving the card free of explanatory copy.
- Added that state to the native PIN walkthrough and checked the hint's width at the 640px panel size in regression coverage.
- Manual audio routing, meter geometry, warmth preview and grid behavior are otherwise unchanged. Next step remains CI and native visual review.

### Completion checkpoint

- Completed final native capture review and verified all six workflows on 94f940e.
- PR description rewritten around the final behavior and fresh build/capture links.
- Final checkpoint is documentation only; no runtime edit followed successful verification. Handheld/live-account testing remains the next step.

### Crash-recovery recheck (2026-11-03)

- Resumed from the user's pasted transcript and read live WORKLOG.md and AGENTS.md.
- Found that the previous chat had already completed the requested UI/audio implementation and final handoff. No duplicate runtime changes were needed.
- Verified live PR #186 is open, draft and mergeable at b0376c573998b5c9be67f729a6a6a3a77c76da2e. Its only change after tested implementation 94f940e is WORKLOG.md; its direct parent is that tested revision.
- Independently queried all six exact Actions runs listed above: every run is completed/success on full implementation SHA 94f940e0e7c472b6b2f3ea9c506d4c8280f34c37.
- Read the actual regression job log (126048125888): 134 slot library tests passed, zero failed, including manual_audio_choices_restore_destination_levels_and_mute; the complete regression and native rendering steps succeeded.
- Rechecked handheld artifact 13250356343: present, not expired, built from 94f940e; API expiry is 2026-11-10 16:28 UTC, although cleanup can remove it earlier.
- Prior chat's independent visual review is recorded above; this recheck verified CI/render completion and did not repeat PNG inspection.
- This session adds recovery documentation only. Next action remains handheld and live-account testing. PR stays draft; no merge/release or PDF requested.

### Follow-up investigation and added credential-selector request

- PR #186 merged with user authorization as 6d499951289899fbf2314ecee04d0f69a03a2b28. Follow-ups use fix/ra-background-progress; OTA/time integration will be separate.
- User reports preparation now advancing to 6%, with about 900 games. Source confirms the HTTP worker outlives settings and gameplay. It scans/prepares GBA files, downloads achievement data and badge art, and uploads pending unlocks. Work stops on shutdown/off/lock/sign-out; cache persists.
- Found the five-minute rescan replaces a still-active queue and denominator, explaining backward percentages. Combined metadata/art percentages also obscure the amount/type of work. Local unvalidated changes defer rescans until a pass finishes, publish phase-specific counts/errors/retry state, refresh the settings texture as progress changes, add controller status glyph and flatter Wi-Fi arcs.
- New user steering: Credentials must stay an inline selector; only choosing Unencrypted should request A-confirm / B-cancel in a pill with actual button artwork, no submenu or premature statement that credentials are already readable. B returns to Encrypted. Session only should also behave as a selector. Record this before implementing.
- Upstream OTA/time source located in BrandonKowalski/slot crates/slot/src/update.rs and net_time.rs. Upstream launcher payload naming/layout and version manifest differ from our System ZIP/build tags. BaseOS support delegates to baseos-update status and target/version detection; do not import upstream launcher or credential-file assumptions.
- No Rust toolchain locally; runtime validation must run in existing CI. Current local edits have not been compiled or published yet.

### RA / storage implementation ready for first CI

- Implemented fixed active library pass (no five-minute queue replacement), independent game/artwork counts, pending-upload and failed-game counts, retry/error reporting, settings reraster on progress changes, and a controller status icon while the worker is syncing.
- Credentials now previews Unencrypted inline and requires direct A-confirm/B-cancel in a persistent notification pill with actual button artwork. B restores Encrypted; Session only applies inline. Existing encrypt-before-delete error handling retained. Added regression coverage for counts/rescans, confirmation cancellation/commit, and confirmation-pill lifetime. Native walkthrough updated for direct A confirmation.
- Wi-Fi arcs now share a wider, shallower silhouette in settings and status. Updated RETROACHIEVEMENTS.md (source for generated site guide).
- First compiler/test results still pending; no successful validation claimed. Next: open follow-up PR, fix CI, inspect light/dark production captures. Investigate upstream OTA/time compatibility separately while CI runs.

### Superseding user decision: encrypted credentials only

Before starting the next edits: user proposes removing unencrypted storage entirely, requiring a device PIN, and raising the minimum button PIN to six. Adopt this simpler flow: remove all credential-storage selectors and new plaintext/session-only writes; Wi-Fi/RA require an encrypted, unlocked vault. Existing shorter PINs must remain decryptable for upgrade, but network access waits for a six-plus PIN replacement. Existing plaintext credentials must migrate only after successful vault creation/unlock and encrypted save, then delete plaintext. Preserve failure handling and separate four-button parental codes. The prior inline-selector work is superseded. PR #190 first CI is still running. OTA/time investigation remains separate.
