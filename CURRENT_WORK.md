# Active review — Cart Studio, Samba and credential security

2026-11-04 (Europe/Berlin): user clarified PR #194 is the requested remaining work, and additionally requested a security audit of device-code/PIN encryption for Wi-Fi and RetroAchievements. Continue on `feat/device-studio-network-sharing`, https://github.com/jgraving/slot.sp/pull/194. Evaluate and finish fixes, push checkpoints, run fresh combined CI and merge when verified. This supersedes earlier draft-only instructions below.

Latest main `18a4e7a8abc880e4aaf3c6007d990f5cecbdace4` is integrated in published checkpoint `9143823`, preserving the merged slot/About work. Packaging rebuild fix is published as `b24df42`; its fresh ARM64 Samba cache build is still running. Runtime fixes cover editor label persistence/shape, uninterrupted uploads/disconnect revocation, orphan plaintext cleanup, key-memory clearing, RA cancellation and old Wi-Fi session teardown. See SECURITY-REVIEW.md for scope and limits. Focused tests and device-feature compile pass; full combined/native/CI verification is ongoing. The known sandbox Unix-socket bind test remains enabled in CI. Publish the final source without a CI-skip marker after the Samba cache is saved, verify all checks, then merge #194. Historical green checks below do not validate this newly combined source. No physical handheld testing or real credentials are being used.

Worklog: https://github.com/jgraving/slot.sp/blob/feat/device-studio-network-sharing/WORKLOG.md

---

# Device Studio and network uploads — ready for device testing

Updated 2026-11-04 (Europe/Berlin). Read AGENTS.md and WORKLOG.md first.

- Branch: `feat/device-studio-network-sharing`
- Draft PR: https://github.com/jgraving/slot.sp/pull/194 — keep draft; no merge/release requested.
- Verified source: `b21dd2f9baf1830fadf28509a56aacebe66d48aa`. All six workflows pass on this source; later recovery Markdown does not change runtime.
- Test System + corresponding sources: https://github.com/jgraving/slot.sp/actions/runs/41750769740/artifacts/13286121041
- Native captures: https://github.com/jgraving/slot.sp/actions/runs/41750769859/artifacts/13286511236
- Both artifacts were present/unexpired at handoff; retention is one day, so recheck before sharing later.

## Implemented behavior

- Open Settings > Gameplay > Cart Studio from the selected cartridge. Preserve Select quick controls and fork cards/rendering/button hints. Colors, finish, shape and title labels work offline; Wi-Fi enables catalog artwork matching. Kid mode/gameplay/link restrictions remain.
- Settings > Network > Network sharing advertises encrypted SMB3 to PC/Mac. Sign in as `slot` using the generated sharing password protected by the existing Device code vault. Sharing turns off on reboot by default; explicit persistence waits for vault unlock and the same chosen network.
- The Games share exposes only Inbox/GB, GBC, GBA and Wallpapers. Finish copying, then turn sharing off or select Import uploads. Installed files are never overwritten. Games refresh the carousel; PNGs appear immediately in Display > Wallpaper without changing the selected background.
- Wallpapers require full static PNG validation: at most 16 MiB encoded, 4096 pixels per side and 8,388,608 pixels total. Unsupported, corrupt, partial, hidden, linked or duplicate files remain in Inbox.
- Random exclusive staging paths allow subsequent imports after interrupted writes. Disconnects stop without importing unfinished transfers. Sharing stops for gameplay, Cart Studio, link play, kid mode, sleep/shutdown and launcher crashes. Suspend/restart waits for editor commits and sharing teardown.
- Samba, discovery and both required RPC workers use a private ARM64 loader/runtime. Corresponding sources include both build recipes. Packaging validates binaries, executable modes, source provenance, archive layout/integrity and the OTA size limit.

## Verified and remaining

All six exact-source workflows: ARM64/server/package 41750769740; launcher/native 41750769859; website 41750769786; scaling 41750769831; blending 41750769836; shortcuts 41750769789.

Actual ARM64 server integration passes authentication/password rejection, share enumeration, observed encrypted wire frames during file transfer, SMB2 rejection, game/PNG wallpaper uploads, accented names, path/link isolation and production stop/crash-watchdog cleanup. Device CI passes 144 core tests, 11 editor/upload tests, 56 Studio tests, power/sleep/menu/kid suites, achievement/retro tests and both real bundled cores. Native light/GLES and dark/GL walkthroughs pass; selected local production renders reviewed. A final-source local network walkthrough also passes.

Remaining physical checks: discovery in Windows File Explorer and macOS Finder on the user's router, authenticated handheld transfers, game/wallpaper import on the SD filesystem, Wi-Fi disconnect/reboot persistence, and physical lid/power behavior. These are hardware checks; do not repeat or misdescribe the passing packaged-server test as pending. No physical device, live Wi-Fi credentials, merge or release was used/performed here.

Worklog: https://github.com/jgraving/slot.sp/blob/feat/device-studio-network-sharing/WORKLOG.md

## Previous merged baseline

## Merged slot/About baseline

# Current handoff — slot/About merged

2026-11-04: continue from **main**. [PR #195](https://github.com/jgraving/slot.sp/pull/195) merged as `218bfe06215255001321ef43744e3153b2ab110f` after the user's approval and verification of all five passing PR workflows. The source is the validated runtime `07227db` plus previews and recovery notes; 326 targeted tests pass.

Photo-matched proportions, sharp reversible About reveal, live General-card motion and seated-cartridge preservation are complete. All review videos/screenshots are on main. Read [WORKLOG.md](https://github.com/jgraving/slot.sp/blob/main/WORKLOG.md) for evidence and recovery details. Normal main build/release and launcher checks are running; no post-merge success is claimed yet. Keep #194 separate. Do not repeat reconstruction.

---

# Current development handoff

Updated 2026-11-04. Continue from **main**. Read AGENTS.md and WORKLOG.md before new work.

## Merged and double-checked

The user authorized merging after passing tests. Both PRs are merged:
- [#190: RA and code UI](https://github.com/jgraving/slot.sp/pull/190): 169ad506577db458f690dd613402feea07430f5a.
- [#191: General updates/time](https://github.com/jgraving/slot.sp/pull/191): 356c30749a98d103c249af1f95355047b25c5818.

All six workflows passed on each tested runtime: #190 at 285cfa1e586dabffe1720b3cdcd13f1047230d4f and combined #191 at 92c2e8a470c30ca3add237780d1b32fc932a8156. Changes after those revisions are recovery Markdown only. Combined CI confirms 141 core tests, 31 achievement tests, 18 updater/time/SNTP tests, full launcher/UI regressions, ARM64 packaging/both bundled cores, native light/GLES and dark/GL renders, website, scaling, blending and shortcuts.

- [Combined test build](https://github.com/jgraving/slot.sp/actions/runs/41731781331/artifacts/13276413591).
- [Native captures](https://github.com/jgraving/slot.sp/actions/runs/41731781266/artifacts/13277792404).
- [Detailed worklog on main](https://github.com/jgraving/slot.sp/blob/main/WORKLOG.md).

Artifacts were present/unexpired at handoff; recheck before sharing later. The automatic main build release is [build-2026.11.03-356c307](https://github.com/jgraving/slot.sp/releases/tag/build-2026.11.03-356c307).

## Current behavior

- **Device code** and **Parent code** throughout UI/guides. Credentials always encrypted. Six-plus new controller device codes; separate parent codes stay four. Legacy short codes authenticate for upgrade; passphrases supported.
- Masked entry starts empty. Device setup Done stays dim until six presses; early Done shakes without clearing and shows a short minimum pill. Unlock remains silent and automatic at the correct variable-length code. Menu cancels parent device-code entry directly to the prior menu.
- RA separates game/artwork/upload progress, retains cached-game totals, shows initial reading states instead of 0/0, keeps the controller while work is queued/retrying, and floats the neutral progress/countdown pill outside the card. Up to date requires queued work to finish.
- Bounded worker cache discovery/full PNG validation, unchanged-badge deduplication, paced requests and retry backoff reduce background load. Ordinary failures preserve authentication; expired/invalid credentials request sign-in.
- General begins Device code, Updates, Automatic time, Time zone. Disconnected update/time rows are grey; A opens Network with a short feature-specific pill and B returns. Time zone opens UTC offset directly; clock/offset changes auto-apply and preserve UTC. Done leaves the editor.
- OTA targets jgraving/slot.sp releases with integrity, identity and ancestry checks and explicit restart. BaseOS remains separate and target/compatibility/power gated. Automatic time is opt-in; local offset/daylight-saving changes remain manual.
- Neutral adaptive volume/brightness/warmth/kid-time meters, green charging, wider Wi-Fi arcs and quiet normal credential unlocks are retained.

## Remaining device checks

Confirm real heat/radio/RTC behavior, RA background progress when closing settings, live deferred-award acceptance, both code methods/cancellation, offset persistence and explicit OTA/firmware staging/interruption/recovery. These are remaining hardware tests, not unmerged-development tasks. No real firmware installation or live RA credentials were used in this session.

Optional manually dispatched **RetroAchievements live smoke test** reads GitHub Actions secrets RA_TEST_USERNAME and RA_TEST_PASSWORD for login, read-only lookup and public artwork only. It performs no award or game-session requests; ordinary PR checks use mocks.

Next-chat prompt: Continue jgraving/slot.sp from main. Read CURRENT_WORK.md and WORKLOG.md first. PRs #190 and #191 are merged; preserve completed behavior and log new work before editing.

## Main verification and issue reply

Fresh CI on actual merged main 356c307 passed launcher/native 41735091909, ARM64 build/device/release 41735092008 and website 41735092013. The source revisions had also passed all six premerge workflows; later differences were recovery Markdown. GitHub's previous missing-green display was caused by those later documentation heads, not a failed code suite.

[Replied to RA request #182](https://github.com/jgraving/slot.sp/issues/182#issuecomment-6617858916), tagging its verified author @demifiend9, after main checks passed. No issue closure was requested.

Website CI scoping in [#192](https://github.com/jgraving/slot.sp/pull/192): OS/recovery-only changes do not start website jobs; published guides retain build/link/navigation checks without 3D; model/homepage/shared assets/templates/config changes run 3D/video checks too. Manual dispatch and CI-routing changes exercise everything. Deployment allows an intentionally unnecessary model job to skip while blocking build/required-model failures. Six local routing regressions and YAML validation pass. PR #192 passed workflow 41737516993 on exact head 883cb577b89bd49537abf09d56bcee45b99b8618 and merged as cff3899aa51cb67f6090ef2775a674fe21785d79. One unchanged boot-video observation timed out initially; its single-job retry passed without relaxing the assertion. No new device runtime changes are part of this CI/docs follow-up.
