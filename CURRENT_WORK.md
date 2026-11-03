# Current development handoff

Updated 2026-11-03 (Europe/Zurich). Read this entry point before historical WORKLOG.md notes. It is committed on fix/ra-background-progress; main requires changes to go through a pull request.

- PR #186 is merged as 6d499951289899fbf2314ecee04d0f69a03a2b28.
- [PR #190](https://github.com/jgraving/slot.sp/pull/190): RA progress/status, encrypted-only credentials, minimum six-button PIN, neutral meters, flatter Wi-Fi icons, and quiet/stale-free unlock feedback.
  - Branch: `fix/ra-background-progress`.
  - [Full worklog](https://github.com/jgraving/slot.sp/blob/fix/ra-background-progress/WORKLOG.md).
  - Tested runtime: `16ac0826febc8cc800c0de6d8f24fdee64a1c7b3`; all six workflows pass, including 135 core tests and ARM64 packaging.
  - [Test System ZIP artifact](https://github.com/jgraving/slot.sp/actions/runs/41686624422/artifacts/13257020164). Recheck availability before sharing later.
- [PR #191](https://github.com/jgraving/slot.sp/pull/191): separate dependent draft for fork-targeted OTA and opt-in automatic time.
  - Branch: `feat/fork-ota-time`; currently based on #190.
  - [Full worklog](https://github.com/jgraving/slot.sp/blob/feat/fork-ota-time/WORKLOG.md).
  - Current runtime: `361a9aa82567f019761f81fbaafa460c5bf31372`; full CI is running. Check exact-head Actions before claiming readiness.
  - Local host/device checks, archive/release/SNTP/scheduling/menu regressions and staged-update shell tests passed. Native light walkthrough passed. An interrupted-install simulation preserved executable mode, custom System files, Config, Games and Saves.
  - No hardware firmware update was executed. Physical BaseOS install/recovery tests remain a merge gate.

Final user direction supersedes old storage-selector work: credentials are always encrypted; there are no Unencrypted or Session only choices. Wi-Fi/RA require an encrypted unlocked PIN. New button PINs need six or more presses; old shorter PINs unlock only for upgrade. Separate parent codes remain four buttons. Adjustment/kid-time pills are neutral; charging remains green. The stale Credentials unlocked notice on page entry was a UI notice issue, not evidence that Wi-Fi had disconnected.

Keep each branch's WORKLOG.md current and push checkpoints. PRs #190/#191 remain drafts for device review; no follow-up merge was requested. Do not repeat completed work. Never store actual credentials or PINs in logs.

Next-chat prompt: Continue jgraving/slot.sp PRs #190 and #191. Read CURRENT_WORK.md on fix/ra-background-progress, then the linked branch worklogs and live checks.
