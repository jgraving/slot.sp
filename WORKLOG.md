# Development recovery log

PR #215 merged as `7b4b9b20149c57d063f8aeaf6e63d7c49aaf6ae0` after all six workflows passed on `4c775ddfd7f4686818dbf65c578ff46de22e6fc8`; the merge tree equals the tested tree. Native Wi-Fi and corrected appearance-heading frames were inspected. This Studio checkout incorporates that verified main tree; runtime files merged cleanly and both recovery logs are retained. Publish on PR #216 and require fresh combined-source CI, plus inspection of the corrected anchored chrome and picker/Save captures, before merging Studio.

## Cart Studio motion and affordances — 2026-11-07

Separate branch `fix/studio-motion-affordances` from main `50a9c233`; Wi-Fi/OTA is independently published as PR #215. User explicitly requests separate PRs and authorizes push, PR creation and merge only after green checks. Add directional Finish/Shape chevrons and opening chevrons, anchor cartridge/title across submenus, separate rising card from the moving/shrinking highlighted cartridge, let only side cartridges retreat downward, improve printer spring motion and caption spacing. Next: separate cached layers, add motion regressions/native captures and validate exact source in CI. Hardware testing remains pending.

Scope refinements: the option card occupies only the right column; the cartridge and title float on the left. Add an explicit Save row that stays in the editor. Root B closes clean edits immediately, otherwise asks Save and close / Discard changes / Keep editing. Preserve pending-label completion and automatic saving on power/suspend paths. Added independent full-resolution cart texture and a shelf fallback so worker timing cannot make the cartridge pop into existence.

## Wi-Fi progress and OTA identity — 2026-11-07

Branch `fix/wifi-progress-ota` from main `50a9c233` (PR #214 is merged; the older routing note below is historical). User requests a radiating status-bar Wi-Fi icon during connection and reliable same-day OTA detection. Existing OTA uses source ancestry, not the displayed date, but silently suppresses diverged builds. Next: add cached GPU-animated Wi-Fi waves through join/DHCP; strengthen full-source build identity and diverged-release comparison; add regressions, publish a PR and inspect exact-source CI. Physical device state and the user’s installed source are unknown.

Implementation: one cached dot/wave atlas animates outward via GPU alpha during starting, joining and DHCP, including outside the Network page; connected icon is steady and errors/off stop it. OTA now embeds the full source SHA, watches branch-ref/GitHub SHA changes for Cargo cache invalidation, and resolves diverged tips using canonical source commit timestamps rather than publication dates. An older/equal divergent source is not offered; missing comparison metadata fails visibly. Added same-day/rollback/comparison-failure, icon lifecycle and traveling-wave regressions; existing native Wi-Fi captures now include joining. Local Rust is unavailable, so compilation/native validation must run in CI. `git diff --check` passes.


Local validation: Wi-Fi credential boundaries, Wi-Fi diagnostics (1), OTA recovery (7), shell update suite and whitespace checks pass. The shell suite initially could not create `/tmp`; reran successfully with a workspace TMPDIR. No Rust compilation or native rendering result is claimed. Separate shelf/HUD atlases preserve in-game contrast. Automatic approval review rejected `git push -u origin fix/wifi-progress-ota` because publication needs explicit user authorization; do not bypass it with connector writes. Finish locally, then ask to push and run CI. Changes remain on this local branch until authorized.


User explicitly approved publishing/PR creation and merge when green. Published as PR #215 at `ffd2fb9`; ARM64 network/device/package, scaling, blending, shortcuts and Pages pass. Launcher/native workflow failed and needs log inspection before merge. Follow-up request: make the RetroAchievements background-activity icon slowly pulse; change draw alpha after shared status transforms, preserving cached textures and the existing activity gate. Cart Studio is separately tracked in PR #216.

## Upstream on-device Studio integration — 2026-11-07

### Device scanned-label follow-up

User reports that a latest test-build chat crashed and scanned labels do not appear on device. The native ureq worker is not subject to browser CORS. Live public art index returned 12,494 entries (1,550,369 bytes), and a sample GBA scanned-label WebP returned HTTP 200. Existing `art::fetch` requests only the selected game's image assets after downloading the shared index, but previously parsed the entire index into a JSON tree plus CRC map and used a 15-second global timeout. The device failure itself is unverified without a hardware log.

On PR #214, change catalog lookup to stream one entry at a time, reuse a valid cache younger than 24 hours, and allow 45 seconds for the index and selected game's assets. Preserve CRC aliases, offline fallback to a cached index, original-label rollback, and normal no-match behavior. Add a lookup regression, run available checks and exact-head CI, then report the build and remaining physical-device check. Do not claim the crash is conclusively fixed by a simulated test.

Validation on source `7e7bd80`: ARM64 network/device/package workflow 42386654413 passes and uploaded test System artifact 13580455335 (expires 2026-11-08 15:36 UTC). Shortcut 42386654411, scaling 42386654340, blending 42386654406 and Pages 42386654342 pass. Launcher/native render 42386654368 is still in progress at this checkpoint. The only local checks available here are `tools/test-studio-artwork.py` (2 passed) and `git diff --check`; Rust is absent locally. PR body now links the build and asks for a real scanned-label device test and sanitized diagnostics if it fails. Preserve this exact runtime-source attribution if a Markdown-only checkpoint advances HEAD.

Branch `feat/upstream-device-studio` from main `9d36c36`. Compare original project `slot-cfw/slot` at `0fe8f5cf` and adapt useful on-device Studio behavior to slot.sp's card, typography, hints, motion and cartridge layers. Existing Studio model already includes latest palette selection and title fallback; preserve offline title editing and original-color restoration fixes. Next: implement visual sticker choices, duplicate suppression and apply feedback; audit targeted OS fixes, add regression/native captures, push a reviewable PR and verify exact-source CI. Do not bulk-merge upstream's Collections, Gamebit web server, ROM-renaming sweep or emulator/core migration into this UI change. Preserve artwork cleanup, direct-download shell marks, sharing lifecycle and save rollback. Physical networking/handheld tests remain pending.

Implementation checkpoint: PR #214 is draft. Select now opens Studio, the redundant quick-controls feature/renderer/texture uploads are deleted, and modifier chords/in-game Select remain. Added worker-built sticker atlas and GPU-only paper-feed/label-transfer motion in our native card, duplicate filtering by decoded pixels, failed-artwork retry and one reusable HTTP agent per lookup. Studio power/save completes any pending choice before writing. Added control/modal/save/duplicate and geometry tests and routed the production Studio review through physical Select. Local SMB/artwork Python checks and diff checks pass. First remote build `b2d396c` caught an omitted public export of the existing label-well geometry helper; fixed it without duplicating geometry. Fresh exact-source CI remains required. Rust toolchain is absent locally; no local Rust/native or physical-device result is claimed.

Audit: upstream palette current-color selection and title fallback are already in our Studio state. Our existing original-color restoration, offline editing, bounded RetroArch RZIP saves and analytic shelf springs are retained. New Collections, Gamebit/RomM, Gambatte/default-core migration, extra mDNS responder and automatic ROM filename rewrites need independent data/protocol/core compatibility review; they are not silently imported into this Studio update. No ZIP ROM support is implemented.

## GB/GBC pause menu layering — 2026-11-05

Branch `fix/gb-pause-menu-layering` from main `8a3b7f62`, fixing issue #209. The shared menu wrapper appends hardware after the pause card, so tall GB/GBC cartridges cover Resume/Settings/Exit. Move ordinary paused-game hardware below the card without changing About shell choreography or slot back/interior/front order. Add draw-order regression coverage and production-compositor captures for GB/GBC on light/dark, 640/720, GLES/GL; publish a reviewable PR and tag the reporter in a follow-up with verified results. Physical handheld testing remains pending.

Validation: all 51 in-game-menu tests and 70 related quick-menu/session/accessibility tests pass locally. The native production-frontend/compositor matrix passes all 24 combinations (GBA/GB/GBC × light/dark × GLES/GL × 640/720), generating 96 captures. Restoring the original main renderer makes the draw-order test fail (`cart covers menu ink`) and the actual GB pixel comparison fail (`cartridge covers card at y=257`); the fixed renderer passes. Existing About carriage/depth tests pass. Native GB pause and GBC Settings captures were visually inspected. The game backdrop is a synthetic fixture, not a commercial ROM.

PR #210 contains the fix, animation-frame regression, mandatory native pixel checks and `previews/issue-209/pause-menu-review.webp` for inline review. CI on runtime source `29ec34e2457836ea8f1b9382826d7d6752cdc155` is running at this checkpoint; do not claim completed remote validation or a physical handheld test. The screenshot/worklog follow-up changes no runtime source. User authorized a tagged reply to issue #209; link the PR and real UI preview, distinguish local validation from running CI, and ask the reporter to confirm on hardware. No merge was requested.

## Restore homepage section layout — 2026-11-05

Branch `fix/restore-homepage-flow` from `04dc308`. Remove the added Cart Studio screenshot showcase from the homepage. Keep the interactive hero, feature grid, and original Cart Studio section in their established order. Removed the standalone showcase and unused images. Strict site build passed, and the generated homepage has the hero, feature grid, and existing Cart Studio section in order.

## Faster 3D preview checks — 2026-11-05

Branch `ci/quick-3d-preview` from `889db1b`. Keep stable desktop and mobile 3D checks automatic, make timing-sensitive playback and WebKit capture manual, and narrow model CI routing. Automatic CI covers desktop and phone WebGL, opacity, hit targets, lid, finish, and reduced motion. Manual Pages runs retain timed playback and Chromium/WebKit captures. Six routing tests, JavaScript syntax, and diff checks passed locally. Browser verification runs in CI.

## Website navigation and launcher previews — 2026-11-05

Branch `fix/website-motion-and-features`, based on `3b7fa80`. Stabilize the shared header and guide navigation, refresh feature copy, and show current launcher screenshots. PR #206 covers a shared, more stable header, shorter page slides, fixed guide topic selection, and current launcher captures. The feature guides cover Cart Studio, RetroAchievements, file sharing, on-device updates, and Wi-Fi reports. The strict site build and six CI-routing tests passed locally. Browser validation runs in the website workflow. Local Chromium could not launch in this workspace.

## Cart Studio shortcut and footer — 2026-11-04

Active branch `fix/cart-studio-hint`, based on main `dd5a6b1`. Remove the Start cartridge settings card and restore direct cartridge controls. Keep the GBA Cart Studio hint in the picker footer, clear of the cartridge. Start now opens Cart Studio directly on GB/GBC and the existing emulator picker on GBA. X opens Cart Studio from that picker, with the hint in the same footer row as Cancel, Swap, and Choose.

- Passed 217 focused tests for app behavior, bindings, settings, legends, and the shortcut reference.
- Ran the Cart Studio workflow through the production frontend and visually checked the footer at 640×480 in light mode (GLES) and 720×480 in dark mode (GL).

## Device and settings follow-up merged — 2026-11-04

[PR #202](https://github.com/jgraving/slot.sp/pull/202) merged as `b38b33b437b687b6f0f5762bb48d12a77da7d0dc`. All six workflows passed on `5073bf02d10bebfab03bbd2a7530a5990c196ab7` and the merge introduced no file changes relative to that source.

- Final CI passed for [Blending regression checks](https://github.com/jgraving/slot.sp/actions/runs/41896927078), [Shortcut guide regression checks](https://github.com/jgraving/slot.sp/actions/runs/41896927050), [Scaling regression checks](https://github.com/jgraving/slot.sp/actions/runs/41896927071), [pages](https://github.com/jgraving/slot.sp/actions/runs/41896927077), [Network and achievements test build](https://github.com/jgraving/slot.sp/actions/runs/41896926957), [Launcher quality of life checks](https://github.com/jgraving/slot.sp/actions/runs/41896927014).
- Restored library and ROM-header scanning after sharing stops, repaired File sharing guide links, and aligned regression tests with the current menu order and cached or uncached RetroAchievements retry status.
- Physical SMB and live RetroAchievements retests remain pending.

## Merged network/settings usability — 2026-11-04

PR [#203](https://github.com/jgraving/slot.sp/pull/203) merged as `ef57a6c91a87eca52d939dd5c5ea3e73c3d58477`. All six workflows passed on exact source `96ef3abdbe29c826d095a47ace6cef3731d2b290`, and the merged tree equals the tested tree `94746b4c567995ef3917a87a61784e0f23cb3651`. No runtime edits followed validation. Default brightness is Select + Up/Down; Start + Up was only a deliberately remapped test fixture.

- Full content-SD read/write sharing is available at `smb://<IP>/SDCard`, username `slot`, with the device-displayed password and encrypted SMB3. There is no upload/import UI. All content-card folders are exposed; outside-card symlinks remain blocked. Eject in Finder, then turn sharing off before games, Cart Studio or OTA; these wait until the worker has fully stopped. Stopping refreshes additions/removals in the game library.
- Network file sharing label/disclosure, update spinner, date/source version, installed BaseOS/check result, BaseOS discovery without a PATH-dependent status gate, automatic-time dimming/notification and offline disable, and Start → highlighted cartridge's Cart Studio/Emulator menu are implemented.
- Wi-Fi checks the actual wlan0 IPv4 address; DHCP callback executable mode and watcher startup errors are fixed. RA reconnect wakes artwork retries and reports distinct transport failures. Diagnostics add sanitized clock/BaseOS/sharing/RA facts. No physical Mac or handheld reproduction, RG34xxSP cause, or live RA account success is claimed.
- Final CI: [ARM64/server/device/package](https://github.com/jgraving/slot.sp/actions/runs/41868823745), [launcher/native](https://github.com/jgraving/slot.sp/actions/runs/41868823799), [website](https://github.com/jgraving/slot.sp/actions/runs/41868823804), [scaling](https://github.com/jgraving/slot.sp/actions/runs/41868824890), [blending](https://github.com/jgraving/slot.sp/actions/runs/41868823789), [shortcuts](https://github.com/jgraving/slot.sp/actions/runs/41868823819): all success. ARM64 includes real encrypted full-card transfers, outside-card isolation, 168 launcher unit tests, 32 RA tests, device compilation, both cores and package validation. The sandbox-blocked socket test passes in CI; existing live-account tests remain ignored.
- Complete light/GLES640 and dark/GL720 native menus and motion checks pass, including Studio, sharing, clock, update and long-address captures. Local light screenshots were inspected. Local disk exhaustion prevented additional local captures; final CI captures are generated, not claimed independently inspected.
- [Test System + corresponding sources](https://github.com/jgraving/slot.sp/actions/runs/41868823745/artifacts/13342804811) and [native captures](https://github.com/jgraving/slot.sp/actions/runs/41868823799/artifacts/13344214469) were present/unexpired at merge; one-day retention. The System ZIP is the update; sources are separate.
- The post-merge [Build and release workflow](https://github.com/jgraving/slot.sp/actions/runs/41872586861) succeeded and published [build-2026.11.04-ef57a6c](https://github.com/jgraving/slot.sp/releases/tag/build-2026.11.04-ef57a6c) from the merged commit.
- Earlier OTA #198 and Wi-Fi diagnostics #200 are merged. The [issue #182 reply](https://github.com/jgraving/slot.sp/issues/182#issuecomment-6633575440) already requested the RG34xxSP reporter's sanitized report; do not send duplicate replies without new evidence.


## Earlier PR completion checkpoint — 2026-11-04

User authorized finishing and merging the outstanding device/settings work and its handoff, and explicitly excluded ZIP-support design PR #113. Continue on `fix/device-services-ux` / PR #202; finish documentation PR #204 after the verified runtime merge. Live main is `ef57a6c9` from already-merged #203; its release exists. The companion Cart Studio repository has no open PRs.

Rechecked exact head `b8abd89d`: all six workflows failed. Five device/launcher/regression workflows stop at compilation because the Inbox cleanup removed `scan_library` while retaining both callers. The actual encrypted SMB server job passed. Website build fails on two stale `SETTINGS.md#network-game-uploads` links after the File sharing heading changed. Next: restore post-stop library/header scanning, repair stale guide links/instructions, finish independent UX review, then run fresh exact-source CI before merging. A fresh workspace is available; earlier disk-full notes are historical. Physical handheld/desktop/live-account checks remain unverified.

## File sharing, cartridge menu, clock and RetroAchievements — 2026-11-04

Development notes for [PR #202](https://github.com/jgraving/slot.sp/pull/202) on `fix/device-services-ux`. Reconciled with main `ef57a6c9` after PR #203 landed independently. Preserve the full SDCard share, current Samba safeguards, update handling, and Start-button cartridge menu from main. The draft’s older conflicting implementations are superseded by the merged-main tree plus the follow-ups below.

- File sharing has a masked password and native A-button Show/Hide hints. Set password opens the keyboard and applies the change after encrypted storage succeeds. Removed legacy Inbox code and tests. CI exercises a custom password over SMB3.
- Developer Wi-Fi report shortcut is L1 + R1 on the Network page. The existing result pill confirms export to Diagnostics. No permanent row or public shortcut hint.
- Cart Studio is in the Start-button cartridge menu. Gameplay has no duplicate entry and the picker has no over-cart Studio hint.
- Time zone is an offset-only editor with dimmed calendar and clock fields. Offset updates recompute local display values and preserve UTC. Back restores the previous settings context.
- RetroAchievements retries a failed current-game session independently of background library sync. The RA menu shows a persistent Current game status instead of a transient game-status pill. Added a regression with a failing session and successful library/artwork requests.
- Earlier local validation was blocked by a full sandbox filesystem. Final CI results are recorded above.

## Network and settings usability follow-up — 2026-11-04

Full-card coordination review: the old upload-only server could safely begin asynchronous shutdown as a game/editor started, but direct Games/Saves/System access changes that boundary. Games, Cart Studio and OTA installation now require sharing to be off and its worker fully stopped. A short notice directs the user to turn it off; pending shutdown is not treated as complete. Added a regression covering blocked game/editor/update operations and readiness only after shutdown. This prevents starting local file use while an authenticated client is still writing the same card. Existing disconnect/link/power teardown and full-card read/write access are retained. Default brightness remains Select+Up/Down; Start+Up is only a deliberate custom-binding test fixture.

CI follow-up: GitHub reports an Actions/Pages incident (https://www.githubstatus.com/incidents/xr3myzj7ms51), with queued jobs and delayed state updates. Source `0e3a8d6` reached the bindings suite and failed its obsolete expectation that Start immediately opens the chip picker. Updated that physical-input test to assert the requested cartridge menu, select Emulator, and verify the existing picker opens; strengthened the Start+Up chord assertion so it must not open either menu. Updated the on-device shortcut reference and its label test. No assertion was disabled. Older source `b2dd9eb` already passed the real encrypted SMB wire-transfer step, including full-card folders and outside-card isolation; final-source server/package and all regressions still must pass before merge.

Recovery checkpoint: [PR #203](https://github.com/jgraving/slot.sp/pull/203), branch `fix/network-settings-usability`. Local application/update suites pass (184), launcher unit suite passes (166, known sandbox socket case awaits CI), RA passes (32), UI quick-menu inventory corrected and its 29 tests plus four typography tests pass. Light 640 GLES Start/Studio and Network captures completed; inspected full-card page, disabled manual time and automatic-time notice. Final polish gives the full SMB URL a dedicated row and dismisses the manual-clock notice on the next controller action. Added light/dark native captures for update spinner/version rows and a long IPv4 sharing address.

The local build disk filled during the remaining native capture compilation. Sandbox launch then also failed for lack of space; elevated removal of generated incremental cache was rejected by environment policy. No source was deleted. All code/recovery checkpoints are published; use GitHub CI for remaining device/package, real encrypted SMB wire transfer, expanded regression and native render verification. Do not infer hardware success. Six workflows for source `26169f1` were queued at this checkpoint; this revision requires a fresh run. Final-source CI, merge and release are pending. The source mode fix for the DHCP callback and full-card share remain intentional; Python test file modes remain unchanged.

Validation checkpoint: 166 launcher unit tests and 184 targeted application/update/clock/sharing tests pass locally, with the sandbox-blocked Unix datagram test left enabled for CI. All 32 RA tests pass (two existing live-network tests remain ignored). UI core/clock tests pass; one explicit opening-row inventory needs its two new cartridge entries. Native Start → Cart Studio capture is correct; network/clock and update captures are in progress. SMB credential/config, Wi-Fi privacy/credential and slotlink shell checks pass. Wi-Fi callback was mode 0644 in source: intentionally making it executable and enforcing that during packaging; this is a real portability bug, not a confirmed diagnosis of the user device. PR #203 is draft pending final tests/CI.

Implementation checkpoint: full-card share/Finder URL and post-stop library rescan; Network file sharing label/disclosure; update spinner, date+source version and installed/BaseOS check rows; BaseOS discovery no longer depends on successfully finding its installer on PATH (installation still requires the external verdict); manual clock guards/dimming and offline automatic-time disable; Start cartridge menu; kernel IPv4 detection; RA artwork reconnect wake and transport error categories; extended sanitized report. Device build compiles, shell config checks pass. Initial local tests exposed obsolete import-UI assertions and the known sandbox-blocked Unix socket test; import tests are being revised for explicitly requested direct access, and socket coverage stays enabled in CI. One local incremental-link failure disappeared when rebuilding with CARGO_INCREMENTAL=0. New regressions, native captures, complete tests and remote CI are pending. Mac/handheld failure cause is not yet proven.

Active branch: `fix/network-settings-usability`, starting from main `1d457937`. User requests clear network file sharing/disclosure and working Mac Finder access, update spinner/version identification and BaseOS offer investigation, disabled manual date/time under automatic time, reliable background RA and Cart Studio on the highlighted game. Investigate shared connectivity and preserve existing security/update checks. No physical device reproduction available; user connects from Mac/Finder. Next: inspect server/network/background/update behavior, implement focused corrections and regression checks, publish reviewable changes and verify CI before merge. User subsequently explicitly requested read/write access to all content-card folders; implement a full `SDCard` share with no import UI. Preserve password/encryption and outside-card symlink restrictions.

## Merged Wi-Fi diagnostics — 2026-11-04

Completion: [build-2026.11.04-b4bf214](https://github.com/jgraving/slot.sp/releases/tag/build-2026.11.04-b4bf214) is published with its System ZIP, SD2 ZIP, matching sources, checksums and build metadata. Verified release assets are uploaded, the tag points to merge `b4bf214e6ece23d0fba7f85dbcf4e39f951bbf71`, and [post-merge Build and release](https://github.com/jgraving/slot.sp/actions/runs/41848567747) succeeds. [Reply posted and read back](https://github.com/jgraving/slot.sp/issues/182#issuecomment-6633575440): maintainer has no RG34xxSP; asks demifiend9 to install this version, retry, save the offline report, shut down normally and attach the text file. Nothing is automatically uploaded. Await the reporter's device evidence; no confirmed RG34xxSP fix is claimed.


PR [#200](https://github.com/jgraving/slot.sp/pull/200) merged to main as `b4bf214e6ece23d0fba7f85dbcf4e39f951bbf71`, after all six workflows passed on exact source `6053032e41426b4d83e1c3bb788151f866327728`. Verified live base, final PR head, successful merge and identical tested/merged Git trees. No failed assertion was disabled. Continue from main.

- Final CI: [device/package](https://github.com/jgraving/slot.sp/actions/runs/41846384859), [launcher/native](https://github.com/jgraving/slot.sp/actions/runs/41846384898), [website](https://github.com/jgraving/slot.sp/actions/runs/41846384865), [scaling](https://github.com/jgraving/slot.sp/actions/runs/41846384864), [blending](https://github.com/jgraving/slot.sp/actions/runs/41846384836), [shortcuts](https://github.com/jgraving/slot.sp/actions/runs/41846384857): all success. The workspace-blocked Unix socket test passes in CI. Existing native-only fixtures run separately in the launcher workflow.
- Local verification: 40 Network-menu tests, five diagnostic tests, device-feature compilation, credential-boundary/helper/slotlink checks, production Network export walkthrough and light/dark progress/error captures pass. A deterministic large-output/nonzero-exit test reproduced the CI probe race and passed after the fix. Visual review shortened the report-row hint and error text to avoid overlap at 640 pixels.
- On-device action: Settings → Network → Save Wi-Fi report (last row, Up from Wi-Fi toggle). Wait for the saved filename, shut down normally, copy Diagnostics/wifi-NNNN.txt from the content SD card. Reports omit SSIDs, IP/MAC addresses, credentials, tokens and raw output; collection is read-only/offline and uploads nothing. A report can distinguish association, DHCP/interface address, supplicant detection, route, resolver and watcher/client availability. No manual-IP configuration was added.
- Wi-Fi progress remains active while getting an address, with a 30-second address timeout, a specific disconnect warning and late-lease recovery. Updates show the address-wait reason. The RG34xxSP root cause still requires physical-device evidence; the maintainer has no RG34xxSP. No physical handheld reproduction is claimed.
- Test artifact: https://github.com/jgraving/slot.sp/actions/runs/41846384859/artifacts/13330804470 (verified present; expires 2026-11-05 14:26 UTC). The System ZIP is the update, not the corresponding-source ZIP.
- Main [build/release](https://github.com/jgraving/slot.sp/actions/runs/41848567747), [launcher](https://github.com/jgraving/slot.sp/actions/runs/41848567658) and [website](https://github.com/jgraving/slot.sp/actions/runs/41848567612) started after merge. Release and reporter reply are now complete (see above). The repeated post-merge launcher render run was still in progress at final handoff; the identical source already passed the full pre-merge native workflow. User authorized the merge and reply.
- OTA safety #198 and handoff #199 were already merged before this work; their post-merge release and launcher runs succeeded. Their changes are retained in this merged version.


## Wi-Fi diagnostics and address-wait feedback — 2026-11-04

CI correction: source `2906fc93` exposed a scheduling-dependent probe classification failure in the shortcut workflow (job 126628180116). The capped reader closes stdout, so a large-output child can exit with SIGPIPE before the parent checks its status. The probe now classifies captured output size before the exit code. Added a deterministic large-output/nonzero-exit case: it failed before the fix and passes afterward, alongside all five diagnostics tests. No assertion was weakened. Remote final-source CI is being rerun; the existing Unix-socket test passed in CI.

Validation update: 40 Network-menu tests pass locally (one existing native-only fixture stays explicitly ignored in that run); device-feature compilation and slotlink regression checks pass. The production Network walkthrough and the added light/dark Wi-Fi progress captures pass. Visual review caught a crowded report-row value and an overlong timeout banner; shortened them to “To SD card” and “No IP address · retry”. New native status captures are included in launcher CI. Final-source checks, merge and published build are pending.

Implementation checkpoint: added Settings → Network → Save Wi-Fi report, an asynchronous offline export to Diagnostics/wifi-NNNN.txt. Exported facts are allowlisted; SSIDs, IP/MAC addresses, credentials, tokens and raw output never enter the report. Local command probes have deadlines/output caps; unique report creation refuses a symlink destination and reports write failures. Helper diagnostics are read-only and report watcher/DHCP process presence, required commands and resolver configuration. Connection progress now remains active through a 30-second address wait, polls at one second while waiting, reports address timeout/disconnect, and recovers if a lease arrives later. Menus show the address-wait reason when opening Updates. No static IP/manual configuration added.

Initial local validation: 23 Wi-Fi-related tests pass, including delayed lease/timeout/recovery, offline export and privacy/path/probe failures; the existing Unix datagram control-socket test is blocked by workspace PermissionDenied and stays enabled for CI. Real shell-helper fixture and credential-boundary checks pass; full menu/device/native validation is next. No physical handheld tested or RG34xxSP root cause claimed.

User subsequently authorized merging this new version after successful checks, then replying to demifiend9 that the maintainer does not have an RG34xxSP and asking for its report. Also verified OTA #198 and handoff #199 are already merged; post-merge Build and release 41840412698 and launcher 41840413022 both succeeded. Do not re-merge historical OTA PRs.

Starting from main `75f54551f0bbadfd13f638b19bc2b97e25b69c4d` on `fix/wifi-diagnostics`. User requests on-device diagnostics and a reply to demifiend9 in issue #182, plus accurate progress/errors: their newer RG SP takes about ten seconds to get an IP after association. Add an offline, explicitly exported, sanitized report and keep connection progress active through DHCP with bounded timeout and late recovery. Preserve credentials and existing network controls; advanced manual network configuration is outside this change. Next: implement, exercise delayed/missing IP and privacy/export failure cases, publish a PR/test build, then reply with truthful availability and collection instructions. No hardware reproduction or root cause confirmed.

## Merged OTA safety handoff — 2026-11-04

PR [#198](https://github.com/jgraving/slot.sp/pull/198) merged to main as `b8b359fd1be4507888b6d5f114309384e2791ca9`. Exact tested source: `0cb14a5f941fa6c5ca2d93c6639bd6e85493c793` on `fix/ota-install-recovery`. Verified unchanged main base before merge, successful PR merge, main ref, and identical tested/merged Git trees. Continue from main; earlier in-progress instructions below are historical.

- All five final-source workflows passed: [ARM64/server/device/package 41837356090](https://github.com/jgraving/slot.sp/actions/runs/41837356090), [launcher/native/motion 41837356026](https://github.com/jgraving/slot.sp/actions/runs/41837356026), [scaling 41837356010](https://github.com/jgraving/slot.sp/actions/runs/41837356010), [blending 41837356040](https://github.com/jgraving/slot.sp/actions/runs/41837356040), [shortcuts 41837356051](https://github.com/jgraving/slot.sp/actions/runs/41837356051). No final-source failure/retry or disabled assertion.
- Final ARM64 job 126600114217 confirms all 15 updater integration tests, seven real-shell recovery methods, updater/launcher suites, device compilation, credential and UI regressions, both cores, and production packaging pass. Local targeted checks also passed. Fault coverage includes 14 partial-copy/process-kill cases, preparation and sync errors, cleanup interruption, retry/modes, path redirection, existing ready payload preservation, and Config/Games/Saves/custom-file preservation.
- Two added regressions reproduced actual bugs before fixing them: a temporary directory containing a symlink redirected a copy outside System; interrupted post-install cleanup blocked the next boot despite a complete installed tree. Both now pass. Other fixes close installed/staged parent redirection, destination collisions, ignored stale-tree cleanup errors, incorrect expanded-size accounting, and BaseOS temporary-path/ready-file deletion issues. See [OTA-REVIEW.md](OTA-REVIEW.md).
- Test System and corresponding sources: https://github.com/jgraving/slot.sp/actions/runs/41837356090/artifacts/13326688042 (verified present, expires 2026-11-05 13:28:50 UTC). Native review artifacts: https://github.com/jgraving/slot.sp/actions/runs/41837356026/artifacts/13327594771 (verified present, expires 2026-11-05 13:40:06 UTC). Recheck availability after the one-day retention window.
- Main [build/release 41840412698](https://github.com/jgraving/slot.sp/actions/runs/41840412698) and [launcher 41840413022](https://github.com/jgraving/slot.sp/actions/runs/41840413022) started automatically after merge. Their completion/publication is not claimed here. No manual release dispatch or device flash was performed.
- Recovery is retry from retained staging, not whole-tree rollback. The readiness marker is durably retired only after installed files are synced, so partial cleanup cannot masquerade as an incomplete ready update. Physical SD power-cut/FAT behavior, malicious privileged concurrent tampering, and the user's installed external BaseOS firmware writer remain outside verified scope. An older installed updater executes the first upgrade; the new boot checks apply once the corrected installer is installed. GitHub/TLS/digests are the release trust boundary, not independent signatures.


## OTA safety audit — 2026-11-04

Checkpoint 3faff17 is pushed. Post-install cleanup regression reproduced a blocked next boot after a complete installation: interrupted rm -rf left the ready manifest beside partially removed files. The installer now removes and syncs the readiness marker after the installed-tree sync and before deleting recovery contents. All seven shell recovery methods and updater/launcher suites pass, including this previously failing process-kill case. Rust source remains unchanged from the passing 15-test/device/ARM64 checkpoint. Publish the final cleanup fix and verify the resulting exact-head launcher/device workflows before merging.

Final shell review found a temporary-path collision bypass: put is invoked in an OR-list, so set -e does not stop failed mkdir/rm inside it. A .slot-update directory containing a symlink could redirect cp after rm fails. A new fixture reproduced an outside-target overwrite before the fix (expected red test). Temporary-path preflight and explicit mkdir/rm error returns fix it; all six recovery methods and updater/launcher shell suites now pass. The new preparation-failure tests verify no copy and successful retry. Shell-only changes now also trigger ARM64 CI. This requires fresh exact-source CI; prior 570fecb ARM64/package success and 15 Rust/device compilation checks remain recorded evidence for the unchanged Rust source.

Checkpoint 4d993e6 is pushed in PR #198. Follow-up fixes reject redirected BaseOS temporary paths, preserve already-ready payloads, create downloads exclusively, and remove only payloads published by the current attempt. Launcher staging now aborts failed stale-tree cleanup and exclusively creates a fresh staging directory. All 15 Rust updater tests and five shell recovery test methods pass locally; updater/launcher shell suites also pass, including startup refusal after update failure and final-sync recovery. OTA-REVIEW.md records findings, trust boundaries and physical-device limits. Publish final source and verify its CI before merging.

Implemented preflight rejection of staged/installed symlink components and non-file destination collisions before any installation write, plus a sync of the recovery source before replacement. The archive unpacker now verifies actual decompressed lengths against declared lengths. Recovery tests pass: partial-copy errors and process death at each of seven replacement points (14 fault cases), retries, executable modes, custom files and Config/Games/Saves preservation, redirected paths, directory collisions, and sync failure. Existing updater/launcher shell tests pass. All 11 updater Rust integration tests pass (offline, locked), including integrity, truncation and expanded-size regressions. Fault tests are wired into launcher, device and release CI; launcher triggers include shell-only changes. GitHub validation of this combined source is next. No automatic rollback is claimed: the design retains staging and refuses to launch a partial installation until retry succeeds. Physical power-loss/FAT behavior and the external BaseOS firmware writer remain unverified.

User requests a full OTA safety double-check. Starting from main 9f436a4, review release identity/ancestry, checksum and archive boundaries, staged installation, cancellation, disk errors and recovery. Branch fix/ota-install-recovery. Before edits: installer destination parents are not checked for symlinks/directory collisions, allowing writes outside System on a customized card. Add preflight rejection before any mutation and deterministic interrupted-copy/retry coverage. Preserve Config/Games/Saves and existing custom System files. No physical firmware installation or real power-cut test will be performed.

## Merged Studio/sharing/security handoff — 2026-11-04

PR #194 merged into main as `286077e1854cd0c305afcbedfb18533fa73bc7d2` after all six workflows passed on exact source `7d0334ca5f88714e07e6e7ba774f7ad6d6c58e20`. Verified main base was unchanged and the actual merged tree equals that tested source. Continue from main. This supersedes earlier draft-only/in-progress instructions; #113 remains a separate design draft.

- All six successful runs: ARM64/server/device/package 41828785364; launcher/native 41828785370; website 41828785342; scaling 41828785417; blending 41828785340; shortcuts 41828785488. No failed final-source workflow, retry or disabled assertion. The website correctly skipped unrelated 3D/deployment jobs while passing its guide checks.
- Final ARM64 server job 126570911346 restored the refreshed runtime cache, passed six rebuild safety fixtures and actual SMB3 password/encryption/upload isolation/shutdown checks. Device job 126571262901 passes 154 core tests (including the Unix socket test blocked locally), 32 achievement tests, 55 retro library tests, 15 editor/upload tests, 57 Studio state tests, nine updater tests, power/sleep/menu/kid/time/UI suites, both bundled cores and package checks. Two existing live achievement tests remain ignored; the one native library fixture is separately executed and passed by the launcher job.
- Final local evidence: 332 combined integration tests; 88 UI library/menu tests; device-feature compilation; six packaging fixtures; SMB/Wi-Fi boundary and launch/update shell checks pass. Vault compatibility, nonce/header authentication, orphan cleanup/failure gating, RA cancellation and sharing-session regressions pass. No real credentials or live awards used.
- Native final review: 124 Studio/network PNGs decode with correct dimensions. Both exact-source themes/backends pass all original parent/setup assertions and show No new uploads after empty import. An earlier reused local target/output produced stale fixture state and a parent-method assertion; an isolated checkout with temporary tracing verified cleanup and current behavior, then tracing was removed and both uninstrumented walkthroughs passed. No production/test assertion was weakened. Final CI launcher job 126570275019 additionally passes full photo geometry/About reversal/cartridge motion checks, preserving #195.
- Test build plus sources: https://github.com/jgraving/slot.sp/actions/runs/41828785364/artifacts/13321292975 (present/unexpired, expires 2026-11-05 13:31 Europe/Berlin). Native captures: https://github.com/jgraving/slot.sp/actions/runs/41828785370/artifacts/13321509656 (present/unexpired, expires 2026-11-05 13:41 Europe/Berlin). Recheck availability later; retention is one day. Installable System and corresponding source ZIPs are separate inside the test artifact.
- SECURITY-REVIEW.md records concrete fixes, cryptography/secret-handling assessment, current RustSec reachability review and limitations. No direct PIN/parent bypass found in reviewed paths; short-code offline guessing, physical tampering/unlocked memory, flash erasure and known retained dependency advisory context are explicitly described. This is not a complete vulnerability-absence guarantee.
- Main build/release https://github.com/jgraving/slot.sp/actions/runs/41831780938, launcher https://github.com/jgraving/slot.sp/actions/runs/41831781293 and website 41831781192 started automatically after merge. Their completion/publication is not yet claimed; no manual release dispatch or handheld installation performed. Physical discovery/transfers, reboot/lid/power, SD and thermal/live-account/RTC checks remain testing notes.


## Active #194 completion and credential security review — 2026-11-04

### Final combined-source verification in progress

- Runtime security/editor checkpoint published as `b0afb826d716e04a9118166c36288caafdca9d5d`. Its CI-skip marker preserved the in-progress refreshed Samba cache build; the next final-source checkpoint intentionally triggers normal CI.
- Fresh packaged ARM64 Samba build on `b24df42` now passes real encrypted wire transfers, authentication, isolation and teardown, and saved the updated runtime cache. No server settings changed in the later Rust fixes.
- Combined local integration run: **332 tests pass across 13 suites**, no failures or ignored tests. Dedicated credential/RA and Studio suites passed as recorded above. Three crafted ZIP-metadata regressions are included; all nine updater tests pass, proving symlink/oversize rejection and special-permission stripping in the custom unpacker.
- Native Studio and Network walkthroughs pass in light GLES640/dark GL720: all 124 PNGs decode correctly. Reviewed actual label/artwork, palettes, cards, RGB/hex hints and save return. Replaced an unsupported palette glyph with the supported Original chip. Final review caught empty explicit import retaining its busy notice; event results now distinguish explicit import from automatic teardown, showing No new uploads only for an empty requested import. Two focused regressions pass; final network recapture is running.
- Current source still needs its full exact-head GitHub workflows and final device/UI checks. The one locally blocked Unix socket fixture remains enabled in CI. No physical PC/Mac discovery or handheld transfer is claimed. Security review documents limitations and retained ZIP/font dependency advisory context without claiming a complete audit or every dependency patched.

### Credential/session and editor corrections

- Packaging fix published as `b24df42353f2daf27f72e417dba55ea6f96352a5`; its ARM64 Samba rebuild is running. Preserve that build until its refreshed cache is saved; a runtime checkpoint may skip duplicate CI temporarily, followed by a normal final-source commit/check run. No merge until current combined source passes.
- Fixed upload interruption on unrelated credential saves and disconnect revocation while sharing is starting/paused. Initial enablement/rotation still wait for storage; explicit boot opt-in remains. Three lifecycle regressions pass.
- Vault now explicitly clears Argon2 working memory without changing its format/KDF, detects orphan plaintext credential writes (including PID reuse), bounds legacy reads, and gates service availability until encrypted save plus cleanup succeeds. Four vault tests and the combined network suite pass; failed cleanup is fail-closed. PIN/key still never persist. No cryptographic bypass was found in reviewed paths.
- RA cancellation now gates nested/follow-up requests, rejects late login tokens, preserves pending awards and clears retired auth/form copies. Launcher startup closes only its own previous Wi-Fi session before the locked UI. Achievement suite: 32 pass, two pre-existing live tests ignored; launch/Wi-Fi security shell checks pass.
- Studio preserves exact scanned label filenames, retries missing original snapshots before any mutation, restores original color metadata, and reflows generated title labels for changed GB shapes. 15 editor/upload plus 57 Studio state tests pass. Both current native Studio walkthroughs complete; palette glyph review is being finalized.
- Fresh device-feature compilation passes. Broader local library run: 152 pass, one existing Mesa capture ignored, one Unix-domain socket test cannot bind under the execution sandbox (EPERM). This is recorded as blocked, not passing; it remains enabled in GitHub CI. Remaining combined integration tests are being run separately.
- Security assessment and limitations are in SECURITY-REVIEW.md. Six-button codes permit offline guessing from a copied SD card; no claim of complete vulnerability absence or physical-device protection. Dependency review identified old ZIP extraction advisories in APIs this updater does not use; hostile-metadata regressions are being added to verify custom-unpacker protections.

### Review fixes: reproducible Samba runtime packaging

- Combined main checkpoint is published as `914382328b5703a7aec8b6cd28579e717da1bfd8`; normal six PR workflows started. Review found no runtime merge conflict; CURRENT_WORK routing is reconciled.
- Repeated Samba packaging reused stale private libraries/conversion data and could overwrite source provenance with an incomplete manifest. Rebuilds now recreate only a recognized generated runtime output, preserve source/build caches, regenerate dependency provenance and include the preparation helper in corresponding sources. Both ARM64 workflows include the new helper in their cache key and exercise it on cached and fresh builds.
- Six fixture regressions pass, including repeat/interrupted preparation, preserving cached sources, and refusing unrelated, symlinked or overlapping output paths. SMB config security, shell syntax and simulated updater checks pass. Full current ARM64 server/package checks remain CI work.
- Review also identified upload/session, editor-label persistence, plaintext-residue cleanup and RA cancellation issues; separate fixes and regressions are in progress. No existing passing results are being attributed to those unfinished changes.

User clarified the remaining PR is Cart Studio/Samba #194 (OTA #191 is already merged), requests evaluation/completion, and adds review of device-code/PIN encryption protecting Wi-Fi and RetroAchievements credentials. Finish concrete bugs, publish regular checkpoints, validate combined source and merge after passing checks. Earlier draft-only restrictions are superseded.

Before further runtime edits: integrate latest main `18a4e7a8abc880e4aaf3c6007d990f5cecbdace4` into feature head `5425afe078d14a7b2fdcaf059778fc3791e1ec4b`. Only CURRENT_WORK.md conflicts; preserve both histories and add this active routing. Independent reviews cover editor, SMB lifecycle/isolation, private-runtime packaging/OTA, vault cryptography and asynchronous auth/secret exposure. Fresh device/native/security checks will follow fixes. Previous b21dd2f test evidence remains historical until current source is checked. Hardware discovery/transfers and physical SD threat limits remain explicit; no live secrets or device update will be used.

## Merged slot/About handoff — 2026-11-04

- User approved merging to main after tests passed. PR #195 merged as `218bfe06215255001321ef43744e3153b2ab110f`; GitHub PR state and main ref were verified. Continue from main. This supersedes the earlier draft-only/no-merge instructions below.
- Rechecked all five successful workflows on runtime `07227db624154adb27863808d156ec6ba4e64ad6`; the later branch changes were only recovery Markdown and preview artifacts. The existing 326-test fresh validation remains applicable. Main was unchanged from the tested PR base before merge.
- Automatic main build/release https://github.com/jgraving/slot.sp/actions/runs/41803285822 and launcher checks https://github.com/jgraving/slot.sp/actions/runs/41803285869 have started. Post-merge completion/release publication is not yet claimed. No further runtime changes were made.
- All code, review videos and screenshots are on main. #194 remains separate and unmerged. Physical handheld review remains unverified.

## Active recovery: photo-matched slot and About reveal — 2026-11-04

### Final source, previews and validation

- Runtime is published and verified on GitHub as `07227db624154adb27863808d156ec6ba4e64ad6`, PR https://github.com/jgraving/slot.sp/pull/195. Later checkpoint changes only preview artifacts and recovery documentation. No merge or release performed.
- Fresh isolated build confirms **326 targeted tests pass**, zero failures/ignored, and device-feature compilation succeeds. Source and workflow checks are clean. The earlier shared-target stale example issue was eliminated with separate final-review and validation targets; no assertion was disabled.
- Definitive native review: five 360-frame About sequences (light/GLES640, dark/GL720, and seated GBA/GB/GBC using the real frontend) pass exact General-card return checks. Three 300-frame cartridge sequences use actual App timing and production geometry/audio, each reaching the stop and returning with exactly one insert/eject sound. All 2,821 PNGs validate; all five MP4s decode and match 60fps, dimensions and expected durations.
- Committed previews: `previews/about-shell-{light,dark}.mp4`, corresponding settled PNGs, `previews/about-shell-seated-{gba,gb,gbc}.png`, `previews/slot-{gba,gb,gbc}-insert-remove.mp4`, and `previews/slot-photo-comparison.png`. Videos are full-color H.264; cartridge videos include AAC contact audio. The local capture commit `cbbc01e` has the same runtime source as GitHub `07227db`; sticker serial reflects the capture commit.
- Native review includes sharpness, continuous lip/material, complete GBA cap, tall GB/GBC header clipping, entry/exit/reversal and rapid Back → Up → A. Local hint backings appear only while the moving sticker overlaps footer captions. General remains live and reverses smoothly.
- GitHub runtime checks: scaling, blending and shortcuts pass. Device workflow https://github.com/jgraving/slot.sp/actions/runs/41800925975 succeeds, including both bundled cores and System packaging. Test build: https://github.com/jgraving/slot.sp/actions/runs/41800925975/artifacts/13307753004 (verified available at handoff). Launcher/native workflow https://github.com/jgraving/slot.sp/actions/runs/41800925968 also succeeds, including the full GLES/GL walkthrough, photo geometry, all About sequences and cartridge motion. Native artifact: https://github.com/jgraving/slot.sp/actions/runs/41800925968/artifacts/13308790499 (verified available). All five applicable workflows pass on runtime `07227db`, without failures or retries.
- Physical handheld appearance/performance remains for user testing. No hardware deployment, live account action, merge, or release was performed. Resume by checking PR #195 and this section; do not reconstruct the completed work again.


### Final runtime polish

- Published and verified test/export checkpoint `cfee4ba29f1d5037233b8015179b4168f101380b` on PR #195. Scaling, blending and shortcuts succeed on that revision; launcher/native and device build are still running.
- Kept footer actions readable while the black sticker crosses them using small temporary contrast surfaces, drawn over hardware and under hints, removed once the sticker clears. Added a layer-order/size/lifetime regression; app/menu checks now total 228 and the complete targeted set totals 326.
- About now lazily uploads/reuses one lower-body texture and recolors it during exit. This saves 6,912,000 GPU bytes versus the first checkpoint, with no per-frame rasterization/upload. Final added GPU memory versus main is approximately 6.94 MiB after first About; hardware performance remains unmeasured.
- The before/after image uses identical native GLES640 views of original main and revised source, rendered from isolated checkouts. Cartridge dimensions/travel are unchanged.
- Read-only merge-tree against #194 (`5425afe0`) merges runtime/tests/workflows cleanly; both branches' active CURRENT_WORK.md headings conflict and will need documentation reconciliation if combined. This is source compatibility only; no merge requested or performed.
- Final native exports are being rendered from a fresh checkout-specific target after detecting a stale example executable in the earlier shared target. The initially attempted in-game exports did not exercise the new mode and are not validation evidence. Fresh executable markers are verified before the definitive captures.

### Native review and rapid-navigation correction

- Draft PR: https://github.com/jgraving/slot.sp/pull/195. Runtime checkpoint `30de7b95486128c7f7cd19a6a02cf47c2c313cbc` was published and its branch ref verified.
- Native GLES640/GL720 geometry review produced 96 captures (4 finishes × 3 platforms × empty/entering/seated/About). Light/dark About sequences each render 360 frames with rapid reversal and compare the restored General card pixels. GBA/GB/GBC motion reviews each validate 300 frames, exactly one insert/eject contact sound, connector stop and shelf return; full-color MP4s are being finalized.
- Independent inspection caught a surface-size jump on Back → Up → A opening Date & Time before the shell settled. Parent motion now applies to other settings sheets during the remaining shell motion; a regression assertion covers this case. Final captures are being regenerated after that fix.
- Current relevant local tests: 325 pass (slot app/menu/boot/eject/notices/accessibility 227; UI quick-menu/notices/sticker 51; chrome/material 43; About motion 4). Device-feature compilation previously passed and is being rechecked on final source. GitHub workflows run on the pushed revisions; no remote completion claimed yet.
- Texture memory: cached GPU slot/body textures total approximately 15.1 MiB across four finishes, about 13.5 MiB more than main's slot-only textures. This is an estimate from dimensions; handheld memory/performance and physical sharpness remain unmeasured.

### Runtime reconstruction checkpoint

- Photo review supports a narrower 400px shell around the existing 308px opening, smaller corner wells and a broad continuous lip; the existing cartridge stop and travel remain unchanged. New native vectors and 2× texture caches include the lower About shell with continuous paint/grain.
- About now uses a reversible spring to move shell/sticker/cartridge together and recede the live General card. This is a first runtime checkpoint; motion/visual regressions are being exercised before completion.
- Fresh device-feature compilation passes. Shell material tests: 15 pass, including high-density geometry, neutral hardware, seam/grain and the aperture ratio. Runtime setup smoke checks also passed (77 input, 6 layout); these do not replace the pending full task checks.
- Native review exporters and MP4 encoders are being added; final captures and remote CI are still pending. Regular publication is via the connected GitHub API because this workspace's CLI has no GitHub push credential.


- User explicitly authorized implementation and regular pushes to GitHub. Publish checkpoints on `fix/photo-slot-proportions`; no merge or release requested.
- Live main is `ab180bfb474a723ac3d44d387273e7ece9ab9028`. The prior log's `93797c63` checkpoint and photo-slot branch are absent remotely; reconstructing from source and the attached photos/screenshots/worklog. Historical validation claims are not current results.
- Photos are the visual authority. Use independent judgment for opening/body ratio, narrow cheeks, shallow curved lip, screw wells and rear wall. Preserve the already-good cartridge dimensions, seating and insertion/ejection behavior.
- Recreate General → About shell/sticker reveal, sharp enlarged textures and coordinated live General-card shrink/downward motion, reversing smoothly on Back and rapid input. Preserve seated cartridges, transparent interiors, all finishes/themes, header and actions.
- Work split: shell geometry/material, About/menu/compositor, native motion exporters/CI. Next: inspect photo comparisons, implement coherent checkpoints, run relevant Rust/device/native checks and publish review videos plus draft PR. Physical handheld verification remains separate.

Last updated: 2026-11-04 (Europe/Zurich).
Repository: jgraving/slot.sp
Active branch: `feat/device-studio-network-sharing`
Draft PR: https://github.com/jgraving/slot.sp/pull/194
Current work: on-device Cart Studio and encrypted game/wallpaper uploads; see the latest entries below.
Previous merged PRs: https://github.com/jgraving/slot.sp/pull/190 and https://github.com/jgraving/slot.sp/pull/191
Verified OS implementation/release: `356c30749a98d103c249af1f95355047b25c5818`
Status: active feature development; keep #194 draft. Prior merged baseline follows.

## Merged handoff — 2026-11-03

User authorized merging both follow-ups once tests passed, superseding earlier draft-only instructions. Rechecked all six workflows on both tested runtimes; every workflow succeeded. Later changes were recovery Markdown only.

- PR #190 merged into main as 169ad506577db458f690dd613402feea07430f5a.
- PR #191 retargeted from fix/ra-background-progress to main and merged as 356c30749a98d103c249af1f95355047b25c5818, preserving ancestry.
- Verified combined source: 92c2e8a470c30ca3add237780d1b32fc932a8156. CI: 141 core tests, 31 achievement tests, 18 updater/time/SNTP tests, complete launcher/UI regression checks, both bundled cores, production native light/dark walkthroughs, website, scaling, blending and shortcuts all pass.
- Combined test build: https://github.com/jgraving/slot.sp/actions/runs/41731781331/artifacts/13276413591.
- Native review: https://github.com/jgraving/slot.sp/actions/runs/41731781266/artifacts/13277792404.
- CURRENT_WORK.md on main is the recovery entry point. Continue new work from main; do not reopen the merged drafts or repeat implementation. Earlier draft/retarget instructions below are historical and superseded.
- Physical handheld heat/radio/RTC, live RA/deferred-award acceptance and real firmware interruption/recovery remain testing notes. No live secrets were used, no firmware was installed, and no release was published.
- This post-merge documentation PR records the actual results without changing runtime source.

## Current verified follow-up

Combined runtime **92c2e8a470c30ca3add237780d1b32fc932a8156** passes all six workflows: launcher/native 41731781266, ARM64/package 41731781331, website 41731781278, scaling 41731781246, blending 41731781272 and shortcuts 41731781290.

CI device logs confirm **141 core tests**, **31 achievement tests**, **6 updater + 5 network-time + 7 SNTP tests**, complete parent/menu/UI regressions and both bundled GBA cores. Native CI explicitly ran the RA production-compositor fixture and full light/GLES/dark/GL walkthroughs. The local final dark General/code/time/cancellation walkthrough passed with 47 valid PNGs; key light and dark captures were independently reviewed.

- Combined build: https://github.com/jgraving/slot.sp/actions/runs/41731781331/artifacts/13276413591.
- Combined native review: https://github.com/jgraving/slot.sp/actions/runs/41731781266/artifacts/13277792404.
- Both artifacts were present/unexpired at handoff; recheck later.
- General starts Device code, Updates, Automatic time, Time zone. Disconnected update/time rows are grey and A routes to Network with short feedback. Clock/offset edits apply immediately without shifting UTC; Done leaves the editor.
- Includes final #190 RA/cache/controller/floating-pill, empty-code/Done readiness, direct parent cancellation and consistent Device code / Parent code behavior. Shared runtime 285cfa1e586dabffe1720b3cdcd13f1047230d4f also passes all six workflows (138 core tests). No runtime edit followed successful checks.
- This recovery synchronization explicitly includes parent documentation commit f8932a8ddf855f7e378ac267948dabba81682aba, preserving the stacked PR's mergeability. Later commits change recovery Markdown only.
- Keep both drafts. Physical heat/radio/RTC, live RA-account/deferred awards and actual BaseOS/launcher interruption/recovery remain handheld gates. No live secrets or real firmware installation used. CURRENT_WORK.md is the entry point; next step is device testing, not repeating development implementation/checks.

## Previous verified state — followed by active work below

This is the separate OTA/time draft, stacked on PR #190. Current runtime: `810f1a52d0e7913f6abcfe4ab0b4c6d8d46a21cd`. PR #190 is fully verified on 16ac082; the OTA branch includes its final encrypted-only/six-button PIN, RA progress/icon, neutral meters, Wi-Fi and quiet-unlock changes.

- OTA downloads target jgraving/slot.sp releases and validate integrity, embedded identity and forward ancestry. BaseOS is a separate explicit model-matched update with compatibility and power gates. Automatic time is opt-in and preserves UTC offset/manual settings.
- All six workflows pass on the recorded runtime: ARM64/package and both bundled cores, website, scaling, blending, shortcuts, full launcher/UI regressions and production native renders. All 137 core tests and all 18 update/time integration tests pass.
- Combined test build: https://github.com/jgraving/slot.sp/actions/runs/41692152351/artifacts/13257574576 (present/unexpired when checked).
- Local light/GLES and dark/desktop-GL walkthroughs pass; Updates, disconnected feedback and automatic-time off/on pages reviewed. Final selected-row highlight was also visually verified. Interrupted-install simulation preserved executable mode, custom System files, Config, Games and Saves.
- No physical firmware installation was executed. Real BaseOS installation/recovery and live handheld networking/time remain device-test gates; keep this PR draft. Retarget main after #190 merges.
- Recovery entry point: CURRENT_WORK.md on fix/ra-background-progress. Historical checkpoints below retain superseded requests; final behavior is encrypted-only with six-plus new button PINs and separate four-button parent codes.

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


## Separate OTA/time integration

### Separate OTA/time integration starting

Branch feat/fork-ota-time depends on fix/ra-background-progress / PR #190. Source references: upstream slot commit 87788eafba77bc93ba11048d22f1c2f08f792a91 (OTA) and 4f402cb8bc20431facb03c3277640a9ed8e9bdfb (network time). Adaptation plan before edits: use jgraving/slot.sp GitHub stable releases and slot.sp-System.zip asset with digest/source verification; stage a new System tree and swap at next launch with recovery, leaving Config/Games/Saves untouched. BaseOS only for detected BASEOS_TARGET/BASEOS_VERSION with baseos-update compatibility verdict, verified digest/size, explicit install action and low-battery gate. Preserve encrypted supplicant networking; never import upstream wifi.conf. Integrate nonblocking network time using actual connected network and a user toggle, preserving UTC offset and display schedules. No hardware OS update will be run in this session. Runtime toolchain installation is being attempted to validate locally. Not implemented yet.

### OTA/time local implementation checkpoint

- Adapted upstream update worker and SNTP scheduling without upstream Wi-Fi config or settings storage. Launcher release selection is pinned to jgraving/slot.sp and validates GitHub digest/size plus build-info product/repository/architecture/source. Archive validation restricts extraction to System and rejects traversal, symlinks, oversized entries, duplicate names and control characters.
- Network gains Updates and Automatic time. Installs are explicit; BaseOS requires correct target/version, checksum, baseos-update WILL APPLY verdict and adequate battery/charging. Downloads use cancellable workers; no real update has been installed.
- System staging is applied at next launch with atomic per-file replacements, retained complete staging for retry, and no writes outside System. Unlike upstream's cleanup, unknown System files are retained to preserve fork/custom additions.
- Added release/archive/target/cancellation tests and shell tests for incomplete/traversal packages and config/game/save preservation. Shell update and existing launch tests pass locally. Initial host cargo check passes; Rust runtime was recovered locally after installer limitations. Full test/device checks pending.
- Automatic time runs in a worker only when the unlocked network is actually connected, retries failure after a minute, polls daily after success and preserves UTC offset and schedule settings. Existing manual clock UI is retained.

### OTA/time tests ready for CI

- Local host check and device-feature check pass. All 17 release/archive/cancellation/network-time/SNTP tests pass. Shell staged-update and existing launch tests pass.
- Added menu regression coverage for connected-only checks, low-battery BaseOS gating, time-toggle readiness, and progress/completion texture refresh.
- Synced the latest RA fixes/tests from PR #190 into this dependent branch; do not regress its stale-notice or migration-failure coverage. Updated upstream-style auto time to preserve this fork's UTC offset and manual clock menu.
- No real network update or OS installation has run. Physical interruption/recovery and BaseOS install testing remain mandatory before merging this draft.

- Added GitHub ancestry checking before offering a launcher release: stable must be ahead of the running build. Newer/diverged PR builds cannot be downgraded merely because their hash differs from latest stable. No automatic install/restart is performed.

### Compatibility default

- Automatic time defaults off, matching upstream's opt-in behavior and preserving existing manual clock setups. Its switch can enable background correction after Wi-Fi connects. Local checks passed before this default adjustment; the readiness test now covers the off→on→off sequence.

Local OTA source/default checks pass; full CI is running. This branch includes PR #190 and remains a separate draft for OTA/BaseOS device testing.

### Native walkthrough timing follow-up

- PR #190: all six workflows pass on 16ac082; ARM64 System artifact 13257020164 and native-review artifact 13255334115 are present. Light GLES/dark GL Network and PIN captures reviewed locally; neutral volume/kid-time examples reviewed.
- Local OTA native walkthrough completed the correct separate-parent-code UI but its immediate on-disk assertion raced the existing asynchronous settings writer. Added a bounded wait for that write, preserving the assertion instead of removing it. Runtime behavior is unchanged by this fixture correction.
- Expanded the OTA native walkthrough to capture Updates, the disconnected-check pill and Automatic time off/on states. No actual update request is sent by the disconnected fixture. Fresh CI will include these additional pages.

### Native UI review corrections

- Updates' initial helper/status value overlapped its action label. Removed the helper and limited inline state to short Checking/Downloading/Available/Retry labels; detailed failures remain pills. A staged update now offers an actual Restart to install action through the existing safe restart flow.
- Correction to earlier native diagnosis: parent-method marker changes are synchronous, not handled by the asynchronous settings writer. A traced clean walkthrough confirmed correct removal. The generated mock card is now reset before each example run so rerunning into an old output folder cannot inherit old PIN/parent settings. Keep the final marker assertion (bounded wait) and verify the clean walkthrough again. No production kid-menu change was made.

- Clean native OTA light/GLES walkthrough now passes end to end, including the parent-method marker assertion. Updated restart-action regression passes locally. Final change is UI/status and test-fixture cleanup; rerun required CI on this revision.

- Final visual review also found the new Updates page was absent from the shared custom-list focus range. Included its page key so the normal selected-row highlight and motion render there. This is a one-range UI correction; no update-install logic changed. The interrupted-install simulation also passed, preserving executable mode, custom System files, Config, Games and Saves.

### Final packaging checkpoint

Current runtime 810f1a52d0e7913f6abcfe4ab0b4c6d8d46a21cd has a successful ARM64 package/bundled-core run 41692152351, System artifact 13257574576. The last pending workflow is launcher native rendering (41692152466). No runtime changes should follow unless a new failure is found. Update completion evidence and CURRENT_WORK.md when that run finishes.

### Final verified handoff

- Runtime: 810f1a52d0e7913f6abcfe4ab0b4c6d8d46a21cd. All six exact-head workflows completed/success: launcher 41692152466; ARM64/package 41692152351; website 41692152398; scaling 41692152447; blending 41692152404; shortcuts 41692152430.
- Device job 126113196756 logs confirm 137 core library tests plus 6 update, 5 network-time and 7 SNTP tests pass. Staged-update and launch shell tests pass. Both bundled cores validate.
- Combined test build: https://github.com/jgraving/slot.sp/actions/runs/41692152351/artifacts/13257574576.
- Native review: https://github.com/jgraving/slot.sp/actions/runs/41692152466/artifacts/13259275746.
- Production light/GLES and dark/GL walkthroughs completed locally; Updates, disconnected feedback, automatic-time off/on and the final selected-row highlight were reviewed. Clean generated cards prevent repeat-run state contamination. No production parent-menu change was made.
- Local interrupted-install simulation successfully resumed with executable mode, custom System files, Config, Games and Saves preserved.
- This final synchronization changes recovery documentation only and includes the latest PR #190 documentation parent to resolve its worklog conflict. No runtime change follows successful checks. Keep draft for physical BaseOS/RTC/network/recovery testing; do not claim hardware firmware validation.

### Active follow-up: PIN entry, parent cancellation and General network tools (2026-11-03)

Record before implementation:
- Cancelling device-PIN authentication for parental controls with Menu must return to the prior menu immediately, without leaving the blank Unlock with device PIN card requiring another B.
- RA progress must float above menus/card clipping, like the other notification/meter pills.
- Move Updates and Automatic time out of Network and place them directly after Security PIN at the top of General. Grey them out without an active Wi-Fi connection; A opens Network and shows a short Wi-Fi-required pill identifying the selected feature.
- PIN entry starts with zero dots, adds dots per press. During setup/confirmation, Done/Start is dim and changes state/colour until six minimum device buttons (four for a separate parent code) are entered. Start before the setup minimum shakes the existing entry without clearing it and shows a short minimum-length pill. Unlock must preserve silent variable-length verification and legacy four/five-button upgrade; do not disclose the stored PIN length.
- Shared parent/PIN/RA changes belong in #190 and must be carried into #191; General placement/connectivity changes belong in #191. Keep both drafts, run targeted regressions and native captures, then existing CI; no merge requested.

### Automatic time / time-zone correction

Before edits: user reports automatic time is wrong for their location and points out it still needs a time-zone choice. SNTP sets UTC only; existing implementation preserves an arbitrary old fixed offset. Add an explicit Time zone selector alongside Automatic time in General, persist the selected offset and apply it immediately without changing UTC or the automatic-time switch. Use clear UTC offsets and short location examples; do not guess the user's location. Document fixed-offset/daylight-saving limits in the guide. Include regression and native review coverage.

### Wi-Fi stability and immediate time changes

Before edits: user reports repeated Wi-Fi disconnects and RA restarting/blinking; investigate radio readiness and service lifecycle before choosing a fix. Also clarifies the existing UTC offset control is discoverable but requiring Start to save is inconsistent. Make time-zone/offset changes apply and persist immediately; preserve manual date/time editing semantics where needed, and remove misleading Save hints for controls that auto-apply. No logs from the handheld are available, so distinguish source defects/reproductions from unverified radio causes.

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

### General integration ready for final CI

- General now starts Device code, Updates, Automatic time, Time zone. Network contains only Wi-Fi controls and scanned networks. Disconnected network tools are dimmed; A opens Network with feature-specific Wi-Fi-needed feedback and B returns to the original General row. Connected automatic time toggles inline, and Updates returns to General. Disabled Automatic time advertises Select/A instead of a misleading Change hint.
- Time zone opens the existing offset field directly, already in edit mode; changes apply/persist immediately without shifting UTC. Date/time changes from settings also auto-apply; Done simply leaves. Updated setup/update/time guides.
- Carried all shared RA/controller/cache/PIN/cancel/naming changes through remote shared runtime 285cfa1e586dabffe1720b3cdcd13f1047230d4f. The shared native harness uses platform C char so ARM tests can compile it.
- Local General checks: 27 targeted network tests, 58 launcher quick-menu tests and all 61 UI library/quick-menu tests passed. Combined full core/clock/menu rerun is running. Light native General/PIN/connectivity/clock captures reviewed. First walkthrough had incomplete PNG output and a final fixture-marker failure; a fresh traced run completed and every PNG validated. Trace showed synchronous correct marker removal, and temporary trace code was removed. Fresh dark review and exact-head CI remain required.
- No firmware update was installed. No live RA secrets or account tests have been run. Both drafts remain open. Publish the combined revision, then verify six workflows and native outputs on that exact revision before handing over a new test build.

### Final all-green handoff

All six workflows pass on combined source 92c2e8a470c30ca3add237780d1b32fc932a8156. Exact evidence/build/native links are at the top and in CURRENT_WORK.md. Final checkpoint only updates Markdown and carries latest #190 documentation ancestry. Verify docs-only compare and both draft PR mergeability before ending. Remaining work is physical handheld/live-account testing; no follow-up merge/release requested.

### Merge authorized by user — 2026-11-03

Before merging: user explicitly authorizes merging both PRs if tests pass, superseding earlier draft-only instructions. Rechecked all six workflows on shared runtime 285cfa1e586dabffe1720b3cdcd13f1047230d4f and combined runtime 92c2e8a470c30ca3add237780d1b32fc932a8156: all success. Compared current heads f8932a8ddf855f7e378ac267948dabba81682aba / a3d707db65dc69ff009b129f7ebcc03e0e534d4b against those tested runtimes; later differences are only CURRENT_WORK.md and WORKLOG.md. Plan: ready/merge #190 preserving ancestry, retarget #191 to main, ready/merge #191, then record actual merge SHAs in main through a documentation PR. Hardware/live-account caveats remain testing notes, not an approval block; no release or firmware installation requested.

### CI display clarification and requested double-check

The latest documentation heads used CI-skip markers, so GitHub did not display green checks on those heads. The user questioned this, then clarified that passing tests on identical code is acceptable and requested a double-check. Reverified: all six suites pass on source 285cfa1 / 92c2e8a; all subsequent differences, including actual merged main 356c30749a98d103c249af1f95355047b25c5818, are CURRENT_WORK.md / WORKLOG.md only. Do not describe this as untested code or user-provided verification: the agent verified the recorded Actions results. The configured main push workflows are running automatically on 356c307: Launcher quality-of-life 41735091909, Build and release 41735092008, pages 41735092013. No extra release dispatch was issued. Wait for actual results, preserve failures if any, and use normal CI for this pending documentation PR without a skip marker.

### Issue reply requested; recipient verified

User requests an RA issue reply tagging the user after the CI double-check. Verified the sole matching open request: #182, Request: Retroachievements support, authored by @demifiend9; issue/comments re-read before drafting, no competing recipient or existing reply. Authorized reply should link merged support, GBA softcore scope, cached/offline evaluation and queued reconnect uploads, current guide and a verified build. Wait for configured main checks before posting; record actual comment URL afterward. Main ARM64 workflow 41735092008 is pending behind earlier main release build 41735010181 because release-main concurrency is serialized. No CI failure has been reported at this point; no additional runtime changes or manual workflow/release dispatch made.

### Website CI scope requested — 2026-11-04 (Europe/Zurich)

Before edits: user asks why OS changes trigger full 3D website checks. pages.yml broadly matches all root *.md (including recovery notes), and every website run unconditionally starts model/video checks. Narrow root triggers to actual guides consumed by prepare-site.py, and gate model/video checks to homepage/model/shared assets/templates/config/test changes. Guide-only edits must retain website build/link/navigation checks without 3D renders; OS-only and recovery-log-only edits must not start website CI. Manual runs and CI-scope changes should exercise all website jobs. Preserve deployment only after successful build and any required model check; explicitly handle an intentionally skipped model dependency. Add meaningful classifier/deployment-condition regression coverage; no device runtime changes. Main verification and the authorized @demifiend9 reply to issue #182 remain outstanding; post only after verified main membership and main CI double-check.

### Website CI scoping implementation ready

Replaced wildcard root Markdown triggers with the 15 published guide inputs from prepare-site.py. Added a change-classification job; guide changes run build/link/navigation only, while homepage/model/media/shared template/style/font/config changes also run model/video checks. OS sources and recovery notes trigger no website workflow. Manual dispatch and CI classifier/workflow changes run everything. Deployment explicitly accepts an intentionally skipped model job after successful website build, but blocks a failed required model or failed build. Six local routing/integration regressions pass and the workflow YAML parses. This website-workflow change will exercise full website CI once; device runtime is unchanged.

### Merged-main double-check and RA reply completed — 2026-11-04 (Europe/Zurich)

Actual main 356c30749a98d103c249af1f95355047b25c5818 passed all configured push workflows: Launcher/native 41735091909, ARM64 build/release and device regressions 41735092008, website 41735092013. This is fresh CI on the real merged commit, in addition to the six passing premerge source suites. The serialized first-merge build also passed. Automatic release: https://github.com/jgraving/slot.sp/releases/tag/build-2026.11.03-356c307. No manual release dispatch was issued.

After rechecking main membership, all three results and the issue's author/comments, posted the authorized reply to #182 tagging @demifiend9: https://github.com/jgraving/slot.sp/issues/182#issuecomment-6617858916. Reply covers merged GBA softcore support, initial internet preparation, cached offline tracking/queued reconnect uploads, current exclusions, guide and the verified main build. The posted body and tag were read back successfully. No issue closure requested or performed.

Website CI routing helpers/tests live under site/scripts so they also fit the existing website-only exclusions in the device release workflow. This avoids causing an OS release build just for the new website CI helper. Six routing/integration regressions pass after relocation and YAML validation passes. PR #192 changes only website CI and recovery notes; it must pass its normal applicable latest-head checks before merging. No skip marker is used on its final implementation commit. CI-scope/workflow changes deliberately exercise all website jobs once; future OS-only/recovery edits trigger no website jobs, and guide-only edits skip 3D checks. Keep remaining physical/live-account caveats as testing notes, not unmerged feature work.

### Final merged CI and issue handoff — 2026-11-04 (Europe/Zurich)

PR #192 merged into main as cff3899aa51cb67f6090ef2775a674fe21785d79. Exact implementation head 883cb577b89bd49537abf09d56bcee45b99b8618 passes website workflow 41737516993 (attempt 2): routing, build/link/navigation and model/video all pass. The first model job 126271182056 timed out at the unchanged boot-video frame observation (check-sp-model.cjs:171); only that job was retried, and passed with unchanged source/assertion. No test was disabled or loosened. Six local routing/integration regressions and YAML validation also pass.

Website triggers now exclude OS and recovery-only edits. Published guide changes keep website checks without 3D; homepage/model/shared rendering dependencies retain 3D/video checks. Manual and CI-routing changes exercise the full website suite. Helpers remain under site/ to avoid unrelated device release builds. Deployment handles an intentionally skipped model while blocking failed required checks.

Actual OS main 356c307 passed its fresh launcher/native, device/build/release and website workflows. Automatic release https://github.com/jgraving/slot.sp/releases/tag/build-2026.11.03-356c307 remains the current OS build; later changes are website CI/recovery only. The authorized @demifiend9 reply is posted and verified: https://github.com/jgraving/slot.sp/issues/182#issuecomment-6617858916. #190 and #191 were merged earlier; no feature work remains unmerged. Hardware/live-account follow-ups remain testing notes.

This final journal update changes only recovery Markdown. Under the requested CI scoping it should not launch website/device jobs, and uses no CI-skip marker. Compare against the tested CI head before resuming and continue from main.

### On-device Cart Studio and network sharing — 2026-11-04

Before implementation: user requests selective upstream on-device Cart Studio integration plus Samba/network discovery for game uploads. Start from main cff3899 (including merged website CI scope #192). Branch: feat/device-studio-network-sharing. Preserve encrypted Wi-Fi/device code, RA, fork renderer/style/audio/kid policy and OTA source. No merge authorized for these new features.

Plan: adapt upstream controller editor and artwork pipeline to fork cards, preserve offline appearance editing, then add independently controlled SMB sharing/discovery. Sharing defaults off and does not survive reboot unless explicitly opted in; show short warning notification pills. Stop sharing on disconnect/network changes and restrict persistent enablement to the explicitly chosen network. Share only game-upload content, never the SD root, credentials, settings, logs or system files. Verify daemon availability/package integration against BaseOS; do not claim support from a settings switch alone. Add lifecycle/path/security and editor persistence regressions, validate device build and native light/dark UI, publish a test PR/build and durable recovery checkpoints. Physical Windows/macOS discovery and handheld transfer remain device checks until actually exercised.

Implementation checkpoint: upstream Studio state machine, palette navigation, artwork matching/PNG generation and shell-choice persistence are selectively adapted; our existing renderer/default labels remain unchanged. New slot.sp editor uses fork cards, fonts and button hints; Select from carousel or Gameplay > Cart Studio opens it, and child/in-game access is blocked. Label fetching, previews and saves run on workers. First no-default-features host compile passes. Local Rust 1.96 installed under scratch; linker uses system bfd because this execution sandbox lacks /proc/self/exe. No CI/device success claimed yet.

Latest user clarification supersedes the temporary default discussion: sharing must turn OFF on reboot by default, while keeping an opt-in to stay on across reboot on the chosen network. Password protection is required in either case. BaseOS source confirms Avahi hostname resolution but no Samba server; package current signed-source Samba plus discovery rather than exposing a nonfunctional toggle. Shared inbox staging will protect existing games and prevent partial uploads from appearing in the library. Git pushes are unavailable through shell auth in this environment; durable checkpoints use the connected GitHub Git-data API instead.

Network implementation checkpoint: password-only SMB3 with mandatory encryption/signing, a generated independent password encrypted in the existing vault, explicit show/rotate controls, opt-in persistence bound to the selected SSID, default reboot-off, and short warning pills. Sharing/discovery are supervised as one process group, stopped on Wi-Fi disconnect, launcher startup/crash, gameplay/link/kid-mode/sleep/shutdown. SMB exports only Inbox/GB, GBC and GBA with symlink following disabled; user-requested stop/import atomically publishes allowed files without overwriting games. Disconnects stop without importing incomplete transfers. New files refresh the existing racks without resetting preferences/favorites/recents.

Samba is absent from BaseOS. Added a separate ARM64 Debian trixie build job for pinned maintained Samba 4.25.1 and wsdd2, including private loader/libraries so the launcher retains its older glibc baseline. mDNS service advert is restricted to the chosen IP; wsdd2 is restricted to wlan0. Corresponding sources are a separate build/release ZIP to avoid inflating OTA installation; runtime notices/manifest remain in System. Packaging refuses an over-limit OTA ZIP. Native real-SMB integration test covers password rejection, mandatory encryption, upload isolation, symlink/traversal denial and teardown. Local config-generator security checks pass; full live-SMB result is pending CI.

Host compile and device-feature compile on x86 pass. First broader local library run: 137 passed, one intentionally ignored native fixture, three existing Wi-Fi tests need updated indices for the new sharing row, one Unix socket bind test is blocked by this sandbox and remains enabled in CI. No success claimed for ARM64-only code until its actual job runs. Next: fix menu regression expectations, test editor persistence/lifecycle and sharing defaults, native theme review, publish checkpoint and draft PR, inspect CI results and repair all failures.

### Review and CI corrections

Draft PR: https://github.com/jgraving/slot.sp/pull/194. Preserved newly advanced main's documentation through ab180bf before the first published checkpoint; no fork runtime changes were lost.

Initial published checkpoint 80b639f CI exposed a missing Samba build prerequisite (embedded Kerberos expected flex); switched to the installed system MIT Kerberos and added parser build prerequisites. Several existing menu tests assumed the immediate row after Achievements was Fast forward; update those tests to navigate explicitly past the new Cart Studio row. The expected in-game hidden rows now include Cart Studio. These failures are recorded rather than described as passing.

Native light editor walkthrough completes and captures all eight screens. Review caught that combining cartridge pixels with a UI texture recolors some art in light mode: separated raw artwork/swatches from semantic UI textures, retaining the fork's actual cartridge colors. Also replaced generic text keycaps with the existing production bitmap button legends. Preserve the user's Select quick controls: editor access is Settings > Gameplay > Cart Studio only; the earlier Select-route checkpoint note is superseded. Custom-color help uses real footer button icons. Saving on sleep/shutdown is asynchronous and power readiness waits for it.

Security review changed the SMB login mapping to unprivileged nobody (client name slot) and only uses root for the isolated FAT inbox file operations; it does not authenticate a root administrative SMB session. Mandatory SMB3 encryption/signing, no guest, no followed links, explicit network address and /run-only passdb remain. Runtime dependency packages are built in their own supported distro container, with a weekly refreshed cache. No TLS/SMB libraries are injected into the launcher or emulators. Actual live-SMB and ARM64 results remain pending the corrected CI run.

Local follow-up validation: 29 network tests pass (one explicit Mesa-only fixture remains ignored); 56 upstream Studio state/animation tests pass, plus 5 new editor-save/import/security integration tests. Existing quick-menu, in-game and kid suites pass after explicit row navigation corrections. Production dark GL/720 and light GLES/640 Studio walkthroughs complete through save, with eight PNGs each; captures were visually reviewed and button/art-layer corrections are included. Repeat native light review after those final visual corrections, then inspect corrected CI and real server tests. Do not infer physical network-discovery or handheld transfer success from the simulator.

### Corrected host/native review; server download retry

The complete local network walkthrough now passes, including sharing off by default, opt-in reboot warning, on/hidden password, explicit reveal, stop/import and the existing vault/RA/parent flows. Native editor light and dark captures were reviewed with production bitmap hints and untouched artwork colors. Editor lists now use the shared spring selector rather than jumping between raster highlights.

CI on 05e0305: Samba job failed before configuration because download.samba.org refused its TCP connection (curl 7), not a server/test result. Added bounded IPv4 retries for connection errors and an official samba.org mirror; both retain the exact pinned SHA-256. The three small regression workflows reached the slot-ui menu assertion, which still expected nine Gameplay rows and omitted Cart Studio from its openable rows; corrected both explicit expectations. Actual launcher/core menu tests preceding that assertion passed. Source-package and real SMB results remain pending; do not report an available device build yet.

Production Linux sharing code now also type-checks in the local device-feature build instead of being hidden behind an ARM-only cfg. Local launcher script, Wi-Fi credential boundary and SMB config isolation checks pass. New editor/import tests are required in the main/native and ARM64 workflow test lists, not just an optional check. The warning is shortened to “Keep off on public Wi-Fi.” Keep PR #194 draft until exact-head checks and native server/package tests are complete.

### Shutdown-helper portability correction

A direct shell check found Debian/BusyBox-style sh rejects the double-dash argument in `kill -TERM -- -PGID`. Removed that nonportable argument from the production stop, trap and crash watchdog calls. The native SMB test now invokes the real stop helper and separately simulates a dead launcher parent, requiring the listener/process group to close in both cases; it no longer relies solely on Python killing the test process group. Rust-owned stop already uses killpg via libc. Local SMB config/script syntax checks pass. This is a development finding, not a previously released bug. Test-build retention is one day, matching the existing PR artifact preference.

### Minimal Samba runtime and share discovery

Source fetch retries succeeded on 2287a9c, then configure revealed that enabling every optional VFS also selected a snapshot/D-Bus dependency. Restricted the build to the default file VFS and standalone authentication modules instead of adding unrelated filesystem services. Source review also confirmed modern Samba share enumeration needs its named-pipe RPC worker: package/supervise only samba-dcerpcd + rpcd_classic in named-pipe-only mode, through the private loader, and test authenticated share enumeration as well as direct uploads. Include matching private gconv data so Unicode/codepage conversion does not depend on the older BaseOS glibc installation. Restore RPC wrapper executable mode after Actions downloads. Main release packaging validates all server/helper ARM64 binaries and loader; builds are stamped to avoid silently reusing an older configuration. Native server result remains pending corrected CI.

Samba configure now succeeds with minimal modules. The next build exposed Samba's documented limitation on nonshared helper linkage (duplicate RPC objects when forcing rpcd_classic nonshared). Retain the tested nonshared mode for smbd only; other helpers use the already bundled private shared libraries/loader. Added the optional ASN.1 generator prerequisite. This does not change authentication/encryption restrictions. Native sharing-page review also caught the new page key omitted from the shared focus range; included it so Sharing has the same moving selector as the other Network pages. Both sharing UI/default tests pass locally after that correction.

### Built runtime; packaging path and final editor review

105e946 built and installed Samba/wsdd2 and collected dependency sources successfully, then failed copying the build recipe because configure had changed the working directory. Resolve the recipe path at startup for both its stamp and source-bundle copy. The live upload/shutdown test remains pending; this is no longer a compiler failure.

Native artwork review found previews needed the fork's alpha-aware label cleanup and board layers for clear/glitter shells. Reused the existing conversion for previews/downloads, adjusted title-label dimensions to edited GB shapes, and added shape/transparency regressions. All 7 editor/import tests, 59 UI library and 36 cartridge-render regressions pass locally. Password reveal now resets when closing/reopening Sharing; both sharing UI/default tests pass. Native network walkthrough exposed an asynchronous parent-method file write observed before completion; wait for the actual write with a bounded deadline, consistent with its later existing persistence check. Re-run final light/dark walkthroughs and real server/device CI before handoff.

### Resumed upload completion — 2026-11-04

User asks to continue the on-device Cart Studio/network upload work, explicitly including wallpaper uploads from PC/Mac. Verified live draft PR #194 at 105e946; latest ARM64 Samba and launcher workflows are still running, scaling has passed. Recovered a clean checkout of the existing branch. Existing sharing exports only a game Inbox; wallpaper support is missing.

Next: add a confined Inbox/Wallpapers staging folder, bounded full PNG validation and non-overwriting import into Wallpapers, update import feedback and PC/Mac instructions, and add integration/security regressions. Preserve default reboot-off, encrypted password/vault, network binding and lifecycle stops, Select quick controls and existing editor behavior. Inspect live real-server/build failures and repair them. Keep the PR draft; no merge/release requested. Physical desktop discovery and handheld transfers remain unverified.

### Wallpaper and packaging checkpoint

- Added Inbox/Wallpapers to the existing encrypted Games share; installed Wallpapers remains private. Explicit stop/import stages a bounded copy, fully decodes static PNG data and IEND, then publishes without replacing existing files or dangling links. Limits: 16 MiB encoded, 4096 per side and 8,388,608 pixels. Invalid/partial/unsupported/duplicate files remain in Inbox. Imported backgrounds are immediately available in the existing Display picker without changing the current selection.
- Fixed sharing-page invalidation on an empty stop/import and included wallpapers in the completion count. Added actual packaged-SMB wallpaper transfer coverage and source-directory link checks for all four inbox folders. Guides describe PC/Mac discovery, direct address fallback, folder mapping, import and size limits.
- 105e946 server job 126300943095 built Samba and downloaded dependency sources, then failed at final cp of tools/build-samba.sh after cd into the source tree. Fixed by resolving the script's absolute path at entry and using it for stamping/source packaging. Actual real-server integration and device package were skipped on that failed run; do not claim they passed.
- Local Rust 1.96 compile succeeds. All 8 editor/upload integration tests and 56 Studio state tests pass. New UI wallpaper-import/empty-stop regression passes. Python SMB config/password/path checks, shell syntax and git diff whitespace checks pass. Other warnings are existing dead-code warnings.
- Next: push checkpoint, inspect fresh exact-head CI (especially real ARM64 SMB and packaging), repair any failures, review native captures and update draft #194. No device discovery/transfer result claimed.

Concurrent branch update 45ef8ae arrived before the wallpaper push. Reapplied the wallpaper changes with a three-way merge onto it, retaining all label cleanup/transparent-shell/shape/password-hide/native-harness fixes and both worklog entries. The same Samba source-path correction is already included there; no duplicate build-script change is needed. Revalidate the combined source before publishing.
Combined source on 45ef8ae passes all 10 editor/upload integration tests and 56 Studio state tests locally; configuration restrictions still pass. The original 8-test result above predates the two retained rendering regressions. New wallpaper UI test was also compiled/passed before combination; full network suite follows in CI.

### Durable combined-source validation

Published c6516fe6cc77879f348178b8e1c50bd3d188f96c on the existing draft PR #194, based on 45ef8ae. Working tree matches that committed source. Local combined validation: 10 editor/upload integration tests, 56 Studio state tests and 30 network tests pass; one explicitly ignored Mesa fixture remains for CI. Linux device-feature compilation passes. Both editor walkthroughs finish with 10 valid PNGs each and both full network walkthroughs finish with 52 valid PNGs each (light GLES/640, dark GL/720). Selected editor, transparent/glitter, hex, sharing default/reboot warning/password-hidden/stopped captures were independently inspected. These are simulated-device renders, not physical discovery or handheld networking evidence.

On c6516fe, scaling 41748165618, blending 41748165628 and shortcuts 41748165639 have succeeded. Launcher/native 41748165656, website 41748165659 and ARM64 real-server/package 41748165729 are still running. The ARM64 job is building the private runtime; its actual SMB test and device package remain pending. This recovery checkpoint changes Markdown only and must not replace the tested-source SHA with an unverified runtime claim.

Final light/GLES640 and dark/GL720 network walkthroughs pass after the bounded persistence wait; sharing selector, hidden password and warning pills were visually reviewed. Source audit found deep suspend/restart readiness also needs to wait for editor commits and complete sharing teardown (poweroff already did). Added that readiness gate and a regression requiring a saved shell before the suspend callback. All 7 editor/import, 21 power and 10 sleep tests pass locally. Added network runtime copying to the existing developer adb-deploy path, and made PR test packages use release binary/source/OTA-size validation. Publish this lifecycle/package checkpoint on checkpoint/studio-sharing-lifecycle while the feature build runs, then apply it to the feature branch once the compiled runtime is cached.

Concurrent wallpaper-upload checkpoint c6516fe arrived during validation and cancelled 45ef8ae while saving its freshly compiled Samba cache. Retained that checkpoint in full and reapplied the lifecycle/package/developer-deploy fixes on top. Wait for the new running build cache before publishing to avoid cancelling compilation again.

Combined lifecycle validation on c6516fe passes 10 editor/upload, 21 power, 10 sleep and 30 network tests, and device-feature compilation. Existing 56 Studio, 58 quick-menu, 49 in-game and 43 kid tests pass. Live public art index (7,356 entries) and WebP examples were retrieved over HTTPS and parsed/converted by production Rust functions with a synthetic CRC fixture; no ROM was uploaded. Direct Rust HTTP is refused in this execution sandbox, so this does not claim actual on-device TLS testing.

Native server run 41748165729 successfully compiled AND cached Samba, then share enumeration timed out after 15s in smbclient; upload/security steps after enumeration were not reached. Inspect RPC/service diagnostics next, keeping encryption/password restrictions intact. Lifecycle/package checkpoint was published independently as 161f532 on checkpoint/studio-sharing-lifecycle while compilation ran; now apply the same changes atop d202bf6, preserving its recovery notes and wallpaper work. No device package or real-server success claimed.

### Real-server diagnosis: required registry helper and character conversion

The fixture-only debug logs in f6c930a's ARM64 run 41749065109 show password/NTLMv2 authentication succeeds. rpcd_classic then fails initialization because its service-control endpoint calls the absent winreg named pipe. Add the maintained rpcd_winreg binary and private-loader wrapper to the existing named-pipe-only supervisor; validate both worker binaries/wrappers in packages. No guest/root authentication or encryption restriction is relaxed. Logs also show CP850 unavailable: Debian trixie's optional libc-gconv-modules-extra package is needed before bundling matching private conversion data. Add an accented filename transfer assertion. Include the dependency-environment recipe alongside the source-build script in the corresponding-source ZIP. New build cache key is expected because runtime contents changed. Re-run native actual-SMB tests and device package before claiming support.

The first helper correction a5ba53a stopped during apt prerequisites: libc-gconv-modules-extra belongs to newer Debian suites and is not in trixie. Verified Debian's trixie libc6 file manifest already includes IBM850.so. Removed the mistaken extra package and assert the bundled module exists. The fixture's tempfile root was 0700, so the authenticated unprivileged session could not traverse to its otherwise-correct runtime gconv modules; make only that disposable SD root traversable (0755), retaining private auth/passdb paths. Real SD roots are traversable. Keep the accented upload assertion to verify actual runtime conversion. Registry-helper addition remains the independently evidenced fix for the enumeration failure.


### Upload interruption follow-up

Latest f6c930a retains all wallpaper work and adds the concurrent editor suspend/restart gates, power/sleep coverage and release-package validation. Its diagnostic job 126309313521 confirms SMB3.11/NTLMv2 authentication succeeds, then rpcd_classic fails initializing svcctl because the winreg pipe/helper is unavailable. This is a dependency/startup failure, not an authentication failure. Keep existing restrictions and require actual encrypted enumeration/upload success.

While that server issue is being diagnosed, improve upload interruption recovery: a fixed .slot-upload-PID staging filename can survive a crash and block later imports if the PID is reused. Use independently created random staging paths for games and wallpapers; keep pre-existing staging files untouched. Add regression coverage for subsequent successful import in the presence of an old staging file. No relaxed PNG validation or overwrite behavior.

Local interruption regression passes: 11 editor/upload tests, 21 power tests and 10 sleep tests all pass on the combined f6c930a lifecycle source. A concurrent a5ba53a server fix now bundles rpcd_winreg and complete conversion modules, with accented filename coverage. Preserve it and stage the upload-recovery change separately while that runtime compiles, then integrate after its cache is saved.

This integration preserves the server fixes from ca899861 and the upload-recovery checkpoint 7cd771f. The old PID-only staging file remains untouched; subsequent imports create their own exclusive random staging files and remove them after success or validation failure. Fresh combined CI remains required.

Registry-helper build ca89986 now passes real authenticated share enumeration/listing and rejects a wrong password. Its next assertion incorrectly assumed smbclient --client-protection=off refuses server-required encryption: Samba still encrypts when the server requires it. Replace that assumption with a transparent loopback relay that parses SMB framing and rejects any plaintext Create/Read/Write/QueryDirectory request while transferring a real fixture with the preference off. Require encrypted transform frames and matching uploaded bytes. Separately reject SMB2-only clients. Keep existing password, traversal/link, Unicode and shutdown/watchdog tests. This is a stronger observation of actual privacy behavior, not a relaxed server setting.

Preserved concurrent upload-recovery commit 4864717 in full before publishing the wire-level SMB test correction. Its random exclusive staging files and interruption regression are retained.

### Final verified upload/Studio handoff — 2026-11-04

All six workflows pass on source b21dd2f9baf1830fadf28509a56aacebe66d48aa: ARM64 real-server/device/package 41750769740, launcher/native 41750769859, website 41750769786, scaling 41750769831, blending 41750769836 and shortcuts 41750769789. No runtime edits followed this verification.

- Actual ARM64 server job 126315020329 passes share enumeration, wrong-password/SMB2 rejection, observed encrypted SMB3 transform frames with no plaintext file operations, game and wallpaper uploads, accented filenames, path/link isolation, production stop helper and dead-parent watchdog cleanup.
- Device job 126315306921 passes the full device-feature check; 144 core tests (one explicitly ignored native fixture), 11 editor/upload integrations, 56 Studio state tests, 21 power, 10 sleep, menu/kid/time/update suites, 31 achievement tests (two explicitly ignored live tests), 55 retro library tests, and both bundled core checks. Packaging validates the private runtime/workers, corresponding sources, executable modes, archive layout/CRC and OTA size limit.
- Launcher/native job 126315020572 passes full regressions and all production GLES/GL walkthroughs. Local source review covered editor cards, alpha-aware artwork, clear/glitter shells, RGB/hex, real footer legends, sharing defaults/password hiding/warnings, and both themes. The final-source light network walkthrough also finishes successfully after all lifecycle changes.
- Test System + sources: https://github.com/jgraving/slot.sp/actions/runs/41750769740/artifacts/13286121041 (present/unexpired when rechecked; expires 2026-11-05 01:05 UTC, subject to cleanup).
- Native review: https://github.com/jgraving/slot.sp/actions/runs/41750769859/artifacts/13286511236 (present/unexpired; one-day retention).
- Existing draft PR #194 contains all wallpaper, interruption-recovery, editor artwork, lifecycle, Samba/RPC/conversion and wire-test fixes. Keep draft; no merge/release requested. Physical PC/Mac discovery and handheld transfer/reboot/lid behavior remain explicit device checks.
- Final recovery update changes only CURRENT_WORK.md and WORKLOG.md. Use [skip ci] for this Markdown-only checkpoint because GitHub PR path filtering uses cumulative changed paths and would rerun/cancel the same already-passing feature suites. Compare current HEAD to the verified source before resuming; do not claim an unverified runtime change.

Follow-up: Select replaces X in the emulator picker; wait for its existing shell/chip closing animation before opening Studio, then carry the closed cartridge left into place. Right opens only the Studio rows with single disclosure chevrons; Finish/Shape still adjust and Save still requires A. Added regression coverage for clean close, save-and-stay, unsaved cancel/discard, anchored submenu poses and clipped option travel. First CI checkpoint compiled successfully but caught the older straight-path midpoint assertion after introducing spring overshoot; corrected the assertion to inspect the carry phase and added continuity/settle coverage. No native/device result is claimed yet.

Save-and-stay review found the worker may coalesce/drop a pending preview when prioritizing Save. On save success while staying open, and on failure, enqueue a fresh preview so the editor cannot remain stale or permanently not-ready. Save/retry regressions now wait for preview readiness as well as the write result.

On source a37f020, scaling and blending pass. Shortcut suite passes 10/11 Studio lifecycle tests and catches a test setup error: ready() had already initialized the entrance clock at zero before the test asserted its initial pose at 1000ms. Initialize the entrance clock before waiting for preview readiness, matching runtime ordering. No production motion timing is changed by this correction. Remaining CI/native checks still pending.

Power/suspend follow-up: if explicit Save is already writing when finish() arrives, retain the close-after-save request instead of leaving the editor open. Reuse the existing write and add an in-flight-save close regression; no duplicate write is started.

Native review found a duplicate SVG fill attribute in the new Wi-Fi atlas; corrected it and added a real four-layer rasterization regression. The first ARM64/package pass did not exercise native asset construction; native rendering remains a merge gate. The RA pulse now has draw-level lifecycle/position coverage. Change Device code uses a vector right-facing chevron instead of Set; locked and unset status remain informative.

Right D-pad now follows single disclosure chevrons in settings lists and the network File sharing / Change Device code rows. Tile grids retain directional navigation, adjustable rows retain value changes, binding controls retain capture behavior, and plain actions are not aliased to A. Added network disclosure input coverage.

Renamed the Display tile/heading to Display & appearance. Moved Recents and Sort Favorites out of General into that card, preserving stable enum/settings IDs. Updated parent/heading/order and Recents navigation regressions plus guide paths.

CI caught a Date & Time regression fixture that walked a hard-coded seven rows in General after Recents/Favorites moved. Updated it to resolve DateTime’s current row position. The new Wi-Fi raster, RA pulse and disclosure tests passed in the same run; full fresh-source CI still required.

Fresh CI passed all 178 launcher unit tests, then found an older navigation regression requiring Right to do nothing on Date & Time/About. Updated that test to the requested disclosure behavior: Left stays on the row, Right opens it, Back returns, and opening/cancelling does not write settings. Existing binding-row Left/Right behavior is retained.

Combined native review now also drives physical Start → X (no Studio) → Select from the GBA emulator picker, captures the shell closure before Studio entry, and verifies the explicit Save row keeps the editor open with a clean baseline before B closes it. Capture waits include save completion as well as preview readiness.

All six workflows passed on ce6db8a, and native Wi-Fi joining/connected frames and settings root were visually inspected. Screenshot review caught the longer Display & appearance page heading being truncated by the legacy 260px heading limit, despite the tile fitting correctly. Give only this heading a 380px one-line allowance, preserving other headings and status corners; add actual raster-width/ink coverage and re-run exact-source CI before merge.

Native source 6c722f2 passed all six workflows. Independently inspected light GBA/GB, dark GBC, label-sheet, transfer, entrance and save-confirmation PNGs from artifact 13592478125. The floating layout, captions, chevrons and save confirmation are visible. Entry review caught global heading/footer being translated with the rising card; preserve their anchored chrome positions while moving only the option body. Final combined-source rendering remains required for this correction and the new picker/Save captures.

All six combined-source workflows passed on 19866a4. Final native captures confirmed anchored chrome and Save-and-stay, but picker-to-Studio frames exposed an existing same-stem cache bug: after saving, picker parts still showed the previous label/finish. Add an appearance revision to builder requests/results and frontend cache keys; invalidate readiness on Save (and shell-logo refresh) and reject late old-revision results. Require fresh CI and inspect the same transition again before merging #216.


### Studio layout cleanup — 2026-11-08

User asks for more cartridge/card spacing, a comfortably inset Save row, single-line option names without Choose/hex/value subtitles, paired adjustment chevrons on the left of Finish, and a lowercase slot.sp shell mark without a rectangular backing.

Active branch fix/studio-layout-cleanup starts at main 97fb96d (merged #216). Reuse the anchored layers/save semantics and simplify only layout. Cartridge/title now occupy a narrower left region with a 32px gutter to the card at both screen widths. Root rows share one 44/52px pitch and focus geometry, removing Save's hard-coded edge position and stacked subtitles. Finish/Shape put both adjustment arrows on the left; other disclosures stay right.

The old placeholder generator interpolated across a broad horizontal rectangle, flattening mould shading into a badge. Rebuilt five offline shell assets from the pinned, locally obtained original facsimiles, reconstructing the lettering area along the mould's vertical shading with smoothed boundaries before applying only lowercase glyphs. Originals remain temporary inputs and are not bundled. Updated generator requires an explicit source directory and preserves the shell outline/alpha. Locally verified all five output dimensions and alpha channels match their source; enlarged cleaned shoulder crops reviewed. Current runtime has not been compiled locally (Rust toolchain is not installed in this workspace).

Next: run exact-source CI and inspect native GBA/GB/GBC light/dark root, Save, wordmark and picker/entry captures. Do not claim success or merge before those results. Physical handheld checks remain pending.

All five workflows passed on c302b9f2 (scaling 42425264269, blending 42425264287, shortcuts 42425264279, ARM64/package 42425264271, native 42425264295). Downloaded artifact 13595472326 and verified SHA-256. Native GBA light/dark and GB/GBC roots, Save focus/stay-open, picker return and label submenu show the requested spacing, single-line names and unbacked wordmarks. Review caught a remaining caret alignment issue: arrow ink centres were 11px below text ink centres (measured on the native 640px frame). Shift only root-row caret quads up 11px; no layout or input semantics change. Re-run exact-head CI and inspect final captures before merge.

### RetroAchievements artwork scope and notifications — 2026-11-08

Branch fix/ra-unused-artwork starts at live main 41a87741. Found demifiend9's exact report in issue #182 comment 6718974752. Production Queue startup/rescan enumerates all cached game JSON and prefetch queues all core badges. Each success waits three seconds; 4,000 badges impose >3.3 hours before network costs. The frontend ignores Notice.badge entirely. User steered fix toward an Achievement artwork switch and actual, well-fitted notification badges. Plan: default off, dynamically cancel/wake artwork work without restarting tracking; only current-game badge preparation; no library artwork scan or prefetch; preserve rules/unlocks and cached files. Decode badges off UI thread; aspect-preserving, smoothly sampled pill artwork with bounded text. Add worker/toggle/layout regressions and render both themes/widths. Toolchain not preinstalled; installing standalone Rust locally because rustup cannot resolve /proc/self/exe in this sandbox. No device verification yet.

Implemented optional default-off artwork with Current game / Favorites & recents (+ current) / Entire library scope. Scope updates use shared atomic state without replacing Service/evaluator. Removed eager discovery of all cached games and artwork-only library refresh requirements. Old cached assets are retained. A synthetic 80-game/4,000-badge regression verifies off=0, favorites/recents=100, explicit library=4,000; dynamic toggles preserve runtime unlocks. Badge HTTP uses independent serial one-second pacing with exponential failure backoff and Retry-After seconds/HTTP dates; cooldown survives reconnect/scope reset. 38 achievement tests pass, two live tests ignored. Three image fit/filter/missing-image tests pass; preference persistence passes after correcting its missing Config-directory fixture. Native production-compositor walkthrough passes both themes at 640/720. Visual inspection caught white text on the light remapped pill; corrected to theme ink, tightened short-pill width, and re-running captures before publication. Physical handheld behavior remains pending. User explicitly authorized replying to issue #182 once fixed.

Corrected final native images were inspected: light640 long pill has dark high-contrast text, full 48px badge, two title lines and two description lines with padding; light720 short pill sizes to its copy. Dark captures and both settings pages are readable and controls fit. Full network suite passes 47 tests (three native review tests ignored there; dedicated notice walkthrough ran separately), settings-notice suite passes four tests, achievement suite passes 38 (two live tests ignored), image layout/filter suite passes three. Device-feature compile is in progress. Local checkpoint 08c69522 exists; shell push cannot read GitHub HTTPS credentials, so publish through the connected GitHub tools and verify the full Git tree hash. No merge or device result is claimed.

Published PR #219 on 0e9bc6fd (tree 5e0798ad matches local). Replied to demifiend9 on issue #182 comment 6719457733, explicitly saying not released/handheld pending. Device-feature compilation passed locally. Moved the three image regressions into slot-ui's library tests so existing CI includes them, with 3/3 passing; existing settings-notice drawing suite passes 8/8. First remote scaling, blending and shortcut workflows pass. Device pipeline's Samba job passes real-server checks, but dependency setup fails before Rust checks because taskfile.dev's installer cannot resolve latest. Website Rust/WASM build succeeds then Binaryen download fails. Both are external download failures, not runtime/test failures. Replace mutable Task metadata lookup with a pinned v3.54.0 official archive + per-archive checksum verification and curl retries in network/release builds. Local attempts to fetch release checksums also return 504, so do not claim successful live installer validation. Shell syntax checked; fresh CI required. No runtime changes since visually reviewed source; only test relocation, build setup and recovery docs.

### Current-game achievement browser and merge authorization — 2026-11-08

User requests demifiend9's issue #182 suggestion: browse cached achievements for the current game, then reply with a ready test build. User explicitly authorizes merging both #218 (Studio controls) and #219 (RA artwork) once tests pass. Live checks: #218 fails label-reset integration in both launcher/device jobs; repair underway in sibling worktree. #219's device setup still gets HTTP 504 from GitHub release download URLs; public GitHub release-asset API succeeds for the same Task checksum file, so use a pinned API fallback with published SHA-256 verification. Browser design: reachable from in-game RA settings, offline cached rules plus server/local unlock union, scrollable locked/earned list, full selected description/points and bounded cached badge decoding off UI thread. No bulk downloads caused by browsing. Verify account isolation, missing cache, local queued unlocks, long text, both themes and widths. Extend #219, preserve #218 fixes separately, then CI/test-build/issue reply/conditional merges.

Browser implementation: read-only account-scoped cache API joins server IDs with durable local unlocks, filters core achievements and follows ROM bytes across renames. UI has three 40px full-image badge rows, locked/earned state, points, progress, All/Locked/Earned filters, shoulder paging and a detail page with scrollable complete descriptions. Cache hashing, definitions and only visible badge decoding happen on background workers; no HTTP is triggered. Missing account/cache/unsupported game/corrupt ledger are explicit. 40 achievement tests pass (two live ignored), model filter/paging/empty/detail tests and network routing are in progress. Native captures at both widths/themes were inspected; added real Back/Filter/Details controller legends and a Scroll hint for long detail pages. Current-game browse disclosure also opens with Right. Task's pinned API fallback and SHA verification now run successfully locally (3.54.0); add --no-same-owner for rootless local extraction. Website build gets packaged Binaryen on PATH because wasm-pack's source explicitly uses PATH before release downloads; optimization remains enabled. Studio PR #218 has four green workflows including ARM64/device/package; native walkthrough still running. No merge or issue-build reply yet.

Final browser source passes 40 achievement tests (two live ignored), 48 network tests (four dedicated native tests ignored in that suite), three browser model tests, the browser disclosure/offline/back routing test, 66 UI library tests and device-feature compilation. Dedicated production compositor browser walkthrough passes all four theme/width combinations, with All/Locked/Earned, paging, full detail scroll and missing cache. Final light640 list, dark720 scrolled detail and missing-cache captures inspected: badge area, title/status/points, full selected detail, real button hints, progress and scrollbar are within the card. Initial local link failure came from reusing the sibling worktree target cache; cleaned slot/slot-ui artifacts and rebuilt successfully. No runtime changes after final review. Publishing #219 browser checkpoint now; #218 ARM64/package build is green and native review remains in progress. No merge yet and issue reply waits for browser test artifact.


### Studio control and alignment revision — 2026-11-08

User reports Finish controls do not work as expected, requests left-aligned Studio menu text and a repaired, cleaner color picker. Live #217 is already merged as 41a8774 (confirmed independently); continue on new branch fix/studio-controls-alignment from live main. Inspection finds root A ignores Finish/Shape, shell color presets overwrite finish, preset A has no confirmation, and every menu label is centered. Palette controls are cryptic O/A/+, RGB has no visual channel feedback. Next: fix input semantics, preserve finish during color selection, align text and controls, improve palette and RGB/hex layouts, add focused regressions/native review, publish for CI. Physical handheld checks remain pending.


Implemented: every root label (including Save) uses the same left edge; Finish/Shape arrow pairs sit immediately left of their labels. A now cycles Finish/Shape as well as directional adjustment, with a short named finish notice and uncluttered A Next hint. Shell color presets, Reset and Auto preserve finish/shape; A confirms a palette selection and returns to its root row. RGB/hex accept A for returning to the previous editor level. Automatic title-label colors retain a generated preview and custom editing starts at the effective label color. Palette uses named Reset/Auto/Custom actions, visible selection rings, bounded row-based scrolling and a scroll indicator. RGB uses left-aligned channel names, numbers, tracks and a color swatch; hex also shows a live swatch. Preserve save-and-stay, unsaved-close confirmation, anchored artwork and picker entry.

Local validation: installed an isolated Rust 1.96 toolchain in this workspace (rustup cannot use /proc here; used official standalone components and the system BFD linker). 62 Studio state tests + 13 Studio control tests pass after a clean launcher rebuild; the palette visibility test passes at both 640/720; 5 legend and 31 quick-menu tests pass. Two artwork Python tests and git diff --check pass. A broader launcher-library run passed 182 tests and ignored 2, but its unrelated wifi::tests::the_control_socket_gets_its_reply_past_the_events_in_between failed at Unix socket creation with PermissionDenied / Operation not permitted. This is not claimed as a full passing library/remote/device run.

Production native walkthroughs passed for light GLES/640 GBA and dark GL/720 GBC, exercising A Finish, both finish changes, root alignment, palette, RGB/hex, save/close and GBA picker return. Reviewed actual root/finish/palette/RGB/hex captures. The last palette scroll correction initially hit stale Cargo output; cleaned the launcher package, reran all 75 Studio tests, verified palette_scroll exists in the rebuilt example, and reran the light native walkthrough successfully. Its scrolled palette now shows complete rows with no wasted clipped band. Rechecked palette visibility after that final correction. No physical handheld or ARM64/remote-CI result is claimed.

Publication blocked: automatic approval review rejected git push -u origin fix/studio-controls-alignment because explicit authorization to publish to jgraving/slot.sp is required. Do not bypass via connector writes. No new PR exists. Implementation, tests and recovery are committed locally; ask to push this branch and open a PR, then run exact-source CI and review uploaded native captures. Do not merge without authorization.


User explicitly approved pushing fix/studio-controls-alignment to jgraving/slot.sp and opening a PR. Shell git push failed because this checkout has no HTTPS credentials. Publish the same verified files through the authenticated GitHub connector; do not merge. Runtime changes remain the locally tested source.


Published as PR #218 on `fix/studio-controls-alignment` at `02257f45c962ba1aea67b06264108415f76335ec`. The connector-created tree `c1368d390f37787f4a6e13694bcfce805540c095` exactly equals local validated HEAD, including all eight changed files. User approval covers publishing and PR creation; do not merge. CI has been requested by the PR; remote and hardware results are not yet claimed. This follow-up changes recovery Markdown only.


### PR #218 merge readiness — 2026-11-08
User now authorizes merging #218/#219 if tests pass. Live #218 CI failed device_studio_sharing::studio_replaces_and_removes_the_scanned_label_filename: resolved automatic preview color leaked into persisted title PNGs, preventing removal of an old scan. Keep resolved color for previews, but let saving an automatic Title remove the existing label so the shelf renders its normal automatic title. Preserve other controls and color/finish fixes. Re-run focused Studio and integration tests, then exact-head CI before merging.

Studio state/control/integration suites now pass (62 + 13 + 10). Preserve automatic preview colors without persisting automatic Title PNGs. Added the same pinned Task installer as #219 with an official release-asset API fallback; the ARM64 archive fetched through that API matches the published SHA-256. This unblocks the device build from GitHub download-URL 504s. Fresh exact-source CI required; user authorized merge only after passing checks.

Combined-build integration: merge reviewed #218 source a58d3f77 into #219 so the browser test package contains both requested PRs. Production code merges cleanly; preserve both logs, #219 recovery routing and its verified API-fallback Task installer. Remote publication will use parents 26168f2c and a58d3f77. Neither PR has been merged to main yet. Run combined Studio/browser tests and exact-head CI before conditional merges.

Combined #218/#219 source passes all 85 focused Studio tests (62 state, 13 controls, 10 device-sharing integrations). Full launcher library run passes 187 tests; one existing Unix Wi-Fi socket test is blocked locally by sandbox PermissionDenied, and four native capture tests are separately ignored in that run. The corresponding native/device CI tests run outside this sandbox. Dedicated combined-source browser capture passes all four themes/widths and representative captures were inspected; no runtime edits followed. Preserve this distinction rather than reporting the full local library suite as green. Publish combined source and run exact-head CI/package; only then issue-build reply and authorized merges.

PR #218 merged as 948604c7 after all five exact-head workflows passed on a58d3f77. Downloaded native artifact 13612447566, verified SHA-256 24fcd87182bd87e0699a87c90d5fab947fd11fc475e7dc80821ff7d3d2b26205, and inspected light GBA Finish/glitter controls, dark GBC scrolled palette and dark Save-and-stay capture. Combined #219 head a9970348 (tree ecd4b3e0) is published with a58d3f77 as a second parent, so its device build includes merged Studio fixes. #219 device tests pass and packaging is in progress. Website job failed downloading wasm-pack (HTTP 504); reran only failed jobs in 42467960887. It previously passed this same download and the packaged Binaryen fix on source 26168f2c, so retry transient fetch before changing source again. Issue reply still waits for the combined browser artifact; no #219 merge yet.

Combined a9970348 device check and full runtime tests passed, and the optimized device binary compiled, but dist stopped on exfatprogs source archive download (HTTP 504). No browser/runtime compile failure. The website retry also failed its wasm-pack archive download. Add the same pinned official release-asset API fallback to these remaining downloads: wasm-pack asset 420850440, digest c09f971e…, locally verified as 0.15.0; exfatprogs asset 513924582, retaining the existing 1.4.3 source SHA-256 4b5052cb…, locally fetched and verified, Task recipe dry-run parsed. No runtime source changes. Publish this build-setup checkpoint and rerun exact-head package/website checks. #218 remains merged. Issue reply still waits for an actual browser package; do not send the older Studio-only artifact.


### Achievement browser feedback — 2026-11-08

User requests addressing demifiend9's new issue #182 comment 6721420603. He confirms the browser functions, but reports the game heading touches the card top, excess lower description spacing, asks for a screen-filling list, detail artwork and top-level pause-menu Achievements. Verified live main 6e722115 includes merged #218/#219. New branch fix/achievement-browser-feedback starts there. Plan: dedicated expanded sheet with consistent header/body/footer padding, five visible rows instead of three plus redundant selected description; full badge in details with cached-only asynchronous loading during previous/next; direct GBA pause entry with Back returning to its origin. Preserve settings route, filters, scroll, rate limiting and offline behavior. Verify production captures at both widths/themes and routing/badge regressions, publish updated test build, then reply on #182. Physical handheld tests remain pending.

User steering: pause-menu Achievements must appear only while the RetroAchievements toggle is enabled. Gate on the saved preference (not Wi-Fi/sign-in/service status), also require supported GBA, and keep the settings route available for setup. Add off/on/off plus offline visibility coverage.

Implemented a dedicated expanded sheet (16px screen inset, padded game header, five 54px rows, anchored footer below the card) using the existing page spring, with decorative cartridge chrome suppressed only while browsing. Removed redundant list-description preview in favor of five visible achievements; A keeps full scrollable copy. Detail view now contains the complete cached badge, downfilters large sources and never enlarges small images. Background loading follows previous/next into entries not previously visible; keeps bounded cache and no HTTP. Pause entry has a trophy icon, respects the RA preference and GBA support, and Back returns to the originating pause row/settings row.

Local device-feature compilation passes. All new model/routing tests pass including offline toggle visibility, disable persistence, GB hiding, return origin and asynchronous detail artwork. Full launcher library has 189 passing tests, four separately exercised native tests ignored, one pre-existing Unix socket test fails at socket creation with sandbox EPERM (same limitation as prior work); do not claim a fully green local suite. In-game/quick-menu/session integration tests pass; UI library/expressive/quick-menu checks pass after updating expected row lists for the new entry. Native production review is in progress for 640/720 light/dark with actual cached sample badge, pause entry, five-row list, detail, scrolled detail, previously unseen detail and missing artwork/cache. Physical handheld verification remains pending.

Published initial PR #220 at 76df1e72 (tree 912438a3 matches local). First native inspection confirms expanded geometry and trophy tile, but reveals held Right in the review harness repeatedly advances the browser during frame settling, invalidating its named captures. Release every injected App key, assert list/filter/detail selection per capture and capture return/disabled pause states. Align the browser's Back keycap order with other menus. Re-run final native capture and publish this small follow-up before relying on CI artwork screenshots.


### RA Download for selector — 2026-11-08

User requests paired chevrons and D-pad Left/Right selection for Download for, with A doing nothing, and explicitly authorizes merge after passing checks. Branch fix/ra-download-selector starts at verified main e2db62bd (merged #220). Existing scope row is marked non-adjustable and A cycles it. Mark it adjustable to use the existing paired chevrons and Left/Right footer, ignore A before persistence/worker updates, preserve artwork-toggle A behavior and scopes/backoff. Check longest Favorites & recents value against the label/carets at 640px; fit only this selector if necessary. Update existing persistence test for both directions and A no-op; capture existing production RA walkthrough with all three scopes. Publish, wait for CI, inspect captures and merge with the tested head.

Implemented the scope row's adjustable flag and A no-op before any save/service update. Shared row/footer rendering now supplies paired value chevrons and Left/Right Change. Fit this selector's selected value to remaining space after its label, both chevrons and a 16px gap; other rows retain their layout. Existing persistence regression now verifies every scope in both directions including wrap, A preserves scope/config bytes/revision, and the artwork A toggle still works. All 49 network tests pass (four native tests run separately). Clean-built native RA review and UI tests are running; existing native scenes now include Current game, Favorites & recents and Entire library without adding capture steps. No physical handheld result is claimed.

Local native review passes and confirms all three values fit with complete chevrons; 68 UI library and 31 UI-menu tests pass. Published #221 at 811b2657 (tree e5e33631 equals local). Visual inspection found the shared 400px progress pill covers part of the longer Left/Right Change footer at 640px. Keep the established progress placement and give only this selector a compact D-pad pair at the right plus B Back at the left, avoiding an A hint and progress/control overlap. Re-render before relying on final screenshots and rerun exact-head CI.


### Crash recovery and Sound pairing disclosure — 2026-11-08

Recovered live #221 at 3386b4d5: five workflows passed; launcher native review still running. #220/#219/#218 already merged. User additionally requests a submenu chevron on Sound's Pair device row. Continue on fix/sound-pair-disclosure from #221 so the test build includes the pending selector correction. Add the same single disclosure glyph and value-edge placement used by settings, reserve label space, and allow Right to open pairing consistently with other disclosures. A and Bluetooth enable-on-entry stay intact. No local Rust toolchain is present and direct GitHub clone is unavailable in this workspace; use authenticated connector publication and exact-source CI for verification. Physical handheld checks remain pending. Finish #221 merge after all checks pass, then verify this follow-up.


### Native Sound disclosure coverage — 2026-11-08

Verified original #222 815f91f1 passes launcher test step including routing regression; native rendering/build still running. Existing automatic accessibility captures do not expose the Bluetooth-only Pair row. Add selected/unselected Pair audio device fixtures to the existing production control_preview compositor walkthrough, automatically rendered at GLES/640 and GL/720. This changes only review coverage, not runtime behavior; git diff --check passes. Local Rust unavailable. Publish this checkpoint and require fresh exact-head six-workflow verification plus inspection of the new native images before reporting visual readiness. #221 remains 3386b4d5, five workflows pass, launcher motion review pending. Merge only #221 after six successes; #222 stays unmerged. Physical handheld checks pending.


### Final selector merge and Sound readiness — 2026-11-08

#221 merged as a7306c161074a77e8691632c8c2bdd86e54afaa1 using expected head 3386b4d589f52549cca5e2bfbf4dc9069f4828e4 after all six workflows passed. Verified merge tree ca52846b9b4be2bafd08cf6f310f27095ed622b2 equals tested head tree. Inspected downloaded native RA artifact 13622451960 (SHA-256 32472812924f484e8e187fe2e92c42c2ffc183c1fed87dd331d22cdf7e781216): 640/light Favorites & recents and Entire library, 720/dark Current game. Paired chevrons/values fit; progress clears B and D-pad footer. No automatic merge or check bypass used.

#222 is ready, remains OPEN and UNMERGED at 6ff991a37516ad8fbf1c8b2ad820946a30dd4ead. Runtime matches original 815f91f1; follow-up adds only native capture coverage and recovery notes. All six exact-head workflows passed:
- Scaling regression checks: https://github.com/jgraving/slot.sp/actions/runs/42493464933
- Blending regression checks: https://github.com/jgraving/slot.sp/actions/runs/42493464961
- Shortcut guide regression checks: https://github.com/jgraving/slot.sp/actions/runs/42493465001
- pages: https://github.com/jgraving/slot.sp/actions/runs/42493464992
- Network and achievements test build: https://github.com/jgraving/slot.sp/actions/runs/42493465098
- Launcher quality of life checks: https://github.com/jgraving/slot.sp/actions/runs/42493465036

Launcher suite includes pairing_disclosure_opens_with_a_or_right_only and Sound style tests. Device workflow passed checks, optimized build, both bundled cores and packaging. Test System + corresponding sources: https://github.com/jgraving/slot.sp/actions/runs/42493465098/artifacts/13622298061 (expires 2026-11-09 15:17 UTC; System ZIP is update, source archive separate). Downloaded native artifact 13623450822, SHA-256 6e16a24eab1cf664b289244b7d0b1b6c29870b8aa034aae203cf742a4ba532cc. Independently inspected controls-gles and controls-gl sound-pair.png and sound-pair-selected.png: visible single value-edge chevron in both focus states, clear label spacing and footer. These are production face/compositor fixtures, not a physical device or live Bluetooth test. Physical handheld checks remain pending. #222 merge requires separate authorization. Final status saved on this separate checkpoint branch so the verified source head is unchanged and CI is not restarted. Follow-up can now be disabled.
