# Development recovery log

Last updated: 2026-11-03 (Europe/Zurich).
Repository: jgraving/slot.sp
Active branch: `fix/ra-background-progress`
Active PR: https://github.com/jgraving/slot.sp/pull/190
Current implementation revision: `16ac0826febc8cc800c0de6d8f24fdee64a1c7b3`
Status: draft, not merged or released.

## Current state — PR #190 validation

PR #186 is merged. PR #190 contains the current requested follow-ups: phase-specific RA progress/counts and controller status, stable library passes, flatter Wi-Fi arcs, encrypted-only credentials with six-plus button PIN creation and legacy upgrade/migration, neutral adaptive adjustment/kid-time pills, and no stale “Credentials unlocked” notification on page entry. Separate four-button parent codes are unchanged.

- Runtime revision: 16ac0826febc8cc800c0de6d8f24fdee64a1c7b3. All six workflows pass, including all 135 core library tests, the complete launcher/UI regressions, native light/dark walkthroughs, both bundled cores, ARM64 packaging, scaling, blending, shortcuts and website checks.
- Local: 27 achievement tests passed (one deliberate public-endpoint test ignored); targeted UI/icon/meter/notice tests pass. Core tests pass except an unchanged control-socket test blocked by the local sandbox; it remains enabled in CI.
- Separate OTA/time draft: https://github.com/jgraving/slot.sp/pull/191 on feat/fork-ota-time, based on this PR. Automatic time is opt-in; updates target this fork's releases and preserve Config/games/saves. Physical BaseOS/recovery tests remain outstanding.
- No follow-up PR is merged; #186 was merged with user authorization. Detailed chronological checkpoints below include superseded requests; this section states the final requested behavior.

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
5. User's next-chat prompt: "Continue jgraving/slot.sp PR #190 and dependent PR #191. Read WORKLOG.md on fix/ra-background-progress and feat/fork-ota-time first."

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

User requests removal of unencrypted storage/selectors and a six-button minimum PIN. Keep existing short PINs decryptable for upgrade, gate Wi-Fi/RA until upgraded, migrate existing plaintext only after encrypted save succeeds, and leave separate parent codes at four buttons. Previous inline-selector implementation is superseded. Record before making these edits.

### Neutral meter steering and encrypted-only implementation

- Latest user steering: kid-time/volume/brightness/warmth pills must be neutral white/grey with adaptive ink; only charging stays green. Recorded and implemented in the shared meter palette.
- Removed storage selectors and all production plaintext writes. Wi-Fi/RA now require encrypted/unlocked credentials; forgetting PIN turns both off. PIN creation minimum is six, while unlock still tries legacy four/five prefixes for upgrade without storing a length hint. Legacy PIN authentication gates networking until replacement. Separate parental code stays four.
- Existing plaintext migration reads only after PIN authentication/creation, writes encrypted data before deletion, and gates services on failures. Added migration/no-PIN/legacy-upgrade regression coverage; historical storage-choice tests superseded. Guide sources and native walkthrough updated. Not yet compiled.

### First CI finding and follow-up ready to publish

- First RA revision compiled and all 134 core library tests passed. UI icon checks found the bundled font has no gamepad glyph. Replaced that glyph with vector artwork in the shared box instead of weakening the visibility tests.
- Encrypted-only and neutral meter edits are ready for a fresh CI run. Source-level whitespace check passes. Previous confirmation-prompt implementation has been removed under the user's superseding direction.

### Encrypted-only CI follow-up

- New minimum-PIN, legacy-upgrade, plaintext migration and network gating tests pass. First encrypted-only run compiled and passed 131/132 core tests. Remaining fixture assumed a Wi-Fi credential save was synchronous; it must wait for the encrypted worker before issuing the next input. Corrected fixture; no runtime change for this failure.

### Stale unlock notification report

Before editing: user reports Credentials unlocked appearing on entering Network and cannot tell whether Wi-Fi was running. Source stores the notice across page opens and changes its page key, causing it to replay. Clear old notices when opening settings pages and remove routine successful unlock text; preserve explicit locked, connection and failure feedback. This notice is not a Wi-Fi connection-state transition.

### Local validation available

- Standalone Rust 1.96 toolchain is now working in scratch with the system linker. All 19 targeted network tests pass locally, including six-button setup, legacy upgrade, cached-password isolation and stale-notice suppression.
- Added explicit encrypted-migration write/deletion failure regressions before final testing. Final UI/core/native CI is still required.

### Final RA source checkpoint before broader CI

- New stale-unlock notice regression and both plaintext-migration failure paths pass locally. Core suite: 134 tests pass; the remaining Unix control-socket test cannot bind in this local sandbox (Operation not permitted) and remains enabled in CI.
- Controller glyph renders as vector artwork; local icon test caught premultiplied edge-colour rounding, corrected by preserving flat RGB with antialiased alpha. Neutral meter checks pass at all tested fill levels/themes.
- Preserve the completed game count while artwork continues instead of clearing it to 0/0 on an empty rescan. Status-corner layout now covers simultaneous audio, mute, Wi-Fi, RA controller and parental icon.
- Next: full CI, production light/dark capture review and fresh device build. OTA/time remains in its own stacked draft branch.

### Current durable handoff

PR #190 runtime is 16ac082; full CI/render/package validation still running. PR #191 adds OTA/time separately. Latest user concern (stale unlock notices) is fixed and covered; no implication that Wi-Fi was disconnected. Keep both draft pending device tests and record final Actions links/artifacts before ending.

### PR #190 final validation

- Independently verified all six exact-revision Actions runs on 16ac082: launcher 41686624436; ARM64/package 41686624422; scaling 41686624187; blending 41686624018; shortcuts 41686624177; website 41686624335. Every run completed/success.
- Test build: https://github.com/jgraving/slot.sp/actions/runs/41686624422/artifacts/13257020164 (present/unexpired when checked).
- Native review: https://github.com/jgraving/slot.sp/actions/runs/41686624436/artifacts/13255334115.
- Locally ran production Network/PIN walkthroughs in light GLES/640 and dark desktop GL/720 and reviewed connected Wi-Fi and six-button hint captures. Reviewed neutral volume/kid-time captures; all adaptive neutral-colour and unchanged green-charging tests passed.
- Runtime code is unchanged after this validation. Later commits are recovery documentation only. No PR #190 merge requested; draft remains for handheld testing.
- A single recovery entry point now exists at CURRENT_WORK.md on this branch; AGENTS.md routes future agents to it. Main requires changes through a PR, so the entry point will reach main when this PR merges.

### Combined handoff complete

Dependent PR #191 also passes all six workflows on runtime 810f1a52d0e7913f6abcfe4ab0b4c6d8d46a21cd. Combined System artifact 13257574576 includes both PRs. CURRENT_WORK.md is the entry point for the next chat and links both detailed branch logs. No further runtime changes followed final verification; both PRs remain draft for handheld review.

### Active follow-up: PIN entry, parent cancellation and General network tools (2026-11-03)

Record before implementation:
- Cancelling device-PIN authentication for parental controls with Menu must return to the prior menu immediately, without leaving the blank Unlock with device PIN card requiring another B.
- RA progress must float above menus/card clipping, like the other notification/meter pills.
- Move Updates and Automatic time out of Network and place them directly after Security PIN at the top of General. Grey them out without an active Wi-Fi connection; A opens Network and shows a short Wi-Fi-required pill identifying the selected feature.
- PIN entry starts with zero dots, adds dots per press. During setup/confirmation, Done/Start is dim and changes state/colour until six minimum device buttons (four for a separate parent code) are entered. Start before the setup minimum shakes the existing entry without clearing it and shows a short minimum-length pill. Unlock must preserve silent variable-length verification and legacy four/five-button upgrade; do not disclose the stored PIN length.
- Shared parent/PIN/RA changes belong in #190 and must be carried into #191; General placement/connectivity changes belong in #191. Keep both drafts, run targeted regressions and native captures, then existing CI; no merge requested.

### RA retries / warm device report

Before edits: user clarifies Wi-Fi seems stable; RA alternates retry countdown and Artwork will retry, and the device becomes warm. Inspect badge retry scheduling and library scan cadence. Keep controller indicator steady while pending/retrying, distinguish artwork failures from account/API connectivity, avoid reauth/rescans for isolated artwork failures, reduce unneeded polling/work, and test retry backoff with fake transport. Hardware temperature cause is unverified without device telemetry. Existing UI/PIN/cancel fixes are locally compiling, not yet published or validated broadly.

### RA status clarification / optional live tests

- Further device reports: artwork retries/blinking appear after leaving settings; completed games then show 0/0. UI visibility does not own the HTTP worker. Found indicator gated only on instantaneous Syncing, game denominator initialized to only uncached games, and all request failures discarded verified auth. Isolated artwork failures already use a separate queue; extend its bounded backoff and throttle background requests, keep pending indicator steady, retain cached totals and use stable concise state labels plus separate retry cause/countdown.
- User asks about GitHub RA username/password secrets for CI. Add an optional manually dispatched smoke workflow with RA_TEST_USERNAME and RA_TEST_PASSWORD for trusted repository branches, isolated from PR checks. Tests may authenticate and fetch public metadata/artwork only; no awards, session creation, private response logs, tokens or uploaded credentials. Credentials must be added through GitHub, never chat. No live-account result claimed until secrets are provided and workflow succeeds.
- Local host compile and all 24 network tests pass (new early-Done/cancel/floating-overlay regressions included). Clock tests found stale expected UTC-shift behavior; corrected expectations because zone selection now preserves UTC and menu changes auto-apply. Parent/UI/achievement suites are still running. Do not merge yet.

### Consistent naming (before edits)

User requests one consistent term throughout UI/notifications. Use Device code and Parent code: code fits controller sequences and keyboard passphrases, avoids alternating Security PIN / device PIN / parent PIN. Parent method values: Device code / Separate code. Internal Rust identifiers and internal PIN_BUTTONS notice tokens can remain for compatibility, but user-visible headings, prompts, hints, notices and guide copy must use code. Update text assertions and production walkthroughs; preserve historical WORKLOG language as history.

### Implementation checkpoint before full CI

- Implemented direct parent device-code cancellation (adult returns to prior menu; child returns to child settings), zero initial dots, neutral incomplete dots, enabled Done only after the six-button setup minimum, and early-Done shake/pill without clearing. Separate four-button parent code retains automatic completion; it has no Done submission requirement. Silent variable-length unlock and legacy upgrade are preserved.
- Moved RA progress out of settings-card raster into a floating compositor layer. Pending/retrying work keeps the controller visible. Cached games count toward total after service restart; empty GBA libraries use words instead of 0/0. Concise stable sync states have separate phase counts, failure cause and retry timer.
- Badge retries now back off 30→60→…→300 seconds; successful artwork and library preparation are paced at three-second intervals. Non-authentication HTTP errors preserve verified account state; HTTP timeout is 15 seconds instead of 5. Parent device-code verification no longer recreates an existing RA service. Physical heat/radio behavior remains unverified.
- Settings date/time edits apply immediately; changing zone preserves UTC. Done closes rather than saves. Naming standardized to Device code / Parent code, including guides. Optional trusted-branch manual live smoke workflow added; secrets are not present or used here.
- Local validation: 137 core tests passed (one Unix socket bind test blocked by local sandbox, enabled in CI); 24 targeted network tests passed before final text standardization; all 29 achievement tests and all 61 UI library/quick-menu tests passed. All 43 child/parent UI tests and four parent storage tests passed after retaining the existing wrong-code-only dot shake. Clock suite 19/19 passes with updated zone semantics. New string-only changes will be rechecked in CI.
- Remaining: carry changes into #191, relocate Updates/Automatic time and expose Time zone in General, validate connected/disconnected routing and native light/dark captures, then all six CI checks and fresh package. Keep drafts. No physical BaseOS installation or live RA account test has run.

### Cached-artwork startup investigation

Before the next edit: user clarifies artwork was still downloading, despite game pass looking complete; also reports Starting/empty counts and warmth. Found Queue::new/scan synchronously parses all cached games and enqueue fully decodes cached PNGs, repeatedly for duplicate badge references. Move discovery/validation into bounded worker maintenance slices, deduplicate known names before disk/decode, keep queued work visible and publish state before expensive work. Preserve full corrupt-PNG validation, cache repair and fake-server tests; do not replace decoding with header-only guesses. Thermal cause remains unverified. Next native review must show separate game/art phases and no completion wording until all queued work is done.

### Bounded cache work and final verification in progress

Shared runtime checkpoint f3f5e012d7f663c36b2164447641dd0cc9db0f89 passed ARM64/package, scaling, blending and shortcut checks; native/website were still running when checked. A newer local revision adds bounded cached-artwork discovery/validation, file-fingerprint deduplication, early status publication and explicit Reading library/artwork states. Full PNG corruption checks are preserved. Retry causes distinguish server, invalid image and card writes; floating meter includes countdown. Completed cached-game totals survive restart; Up to date requires no pending artwork/uploads.

Local newer revision: 137 core tests pass (one explicitly ignored native-render fixture and one sandbox-blocked Unix socket test excluded); 30 achievement tests pass, two live-network tests remain explicitly ignored. Earlier full child/parent (43+4), clock (19) and UI (61) suites passed. Added ignored production-compositor RA fixture and CI capture step: game pass, artwork pass, retry and completion in both light GLES/dark GL. First capture review confirms pill is outside the card, completed games remain 900/900, and controller is present while artwork is pending. Final countdown capture rerender running. General relocation/connectivity tests on #191 pass (27 targeted network tests); its quick-menu suite now passes 58/58 with disabled-row expectations.

Do not infer actual hardware cooling, live RA-account success or firmware installation from these simulations. Both PRs remain drafts. Next: publish this checkpoint, synchronize #191, inspect final native captures and exact-head CI/build artifacts, update CURRENT_WORK.md.

### ARM harness correction and final shared revision

- CI on 3949bc4 found an ARM64 compile failure in the newly included Mesa review helper: eglGetProcAddress used *const i8, but C char is unsigned on that target. Use std::ffi::c_char. This is test-harness portability; normal launcher runtime compiled. Scaling/blending/shortcuts passed; remaining jobs were running.
- Explicitly classify rcheevos protocol Code values expired_token, invalid_credentials and access_denied as authentication failures, preserving account recovery while retaining verified auth for ordinary retries. Primary source checked: RetroAchievements/rcheevos src/rapi/rc_api_common.c. All 31 achievement tests pass locally; no live credentials used.
- Added native walkthrough cancellation/reopen coverage and repeatable clean mock-card setup. Final RA retry capture reviewed in dark GL: neutral floating pill says Artwork · retry 20s 175/800, with completed games 900/900 and steady controller glyph. Up-to-date capture has no progress pill or controller. General light render shows disabled network tools and zero-dot/grey-Done setup correctly.
- One local General walkthrough produced incomplete PNG files and failed its final marker assertion. A clean traced rerun passed and all PNGs validated; trace confirmed synchronous marker removal at the expected root. Temporary tracing was removed. Keep exact assertions and CI native review; do not weaken production parental behavior or claim asynchronous marker writes.
- Broad --no-default-features --tests check also selected an unrelated host-only input test; use the repository's existing feature-specific test commands. Required tests remain enabled in CI.
