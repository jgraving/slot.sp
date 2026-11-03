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
