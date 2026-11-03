# Current development handoff

Updated 2026-11-03 (Europe/Zurich). Read this entry point first; historical worklogs contain superseded requests.

## Ready for device testing

PR #186 is merged as 6d499951289899fbf2314ecee04d0f69a03a2b28. Follow-ups remain separate drafts:

| PR | Branch | Verified runtime | Status |
| --- | --- | --- | --- |
| [#190: RA/UI/security](https://github.com/jgraving/slot.sp/pull/190) | fix/ra-background-progress | 16ac0826febc8cc800c0de6d8f24fdee64a1c7b3 | All six workflows pass; 135 core tests; native renders and ARM64 package |
| [#191: OTA/time](https://github.com/jgraving/slot.sp/pull/191) | feat/fork-ota-time | 810f1a52d0e7913f6abcfe4ab0b4c6d8d46a21cd | All six workflows pass; 137 core tests plus 18 update/time tests; native renders and ARM64 package |

- [Combined test build, including both PRs](https://github.com/jgraving/slot.sp/actions/runs/41692152351/artifacts/13257574576).
- [UI-only test build](https://github.com/jgraving/slot.sp/actions/runs/41686624422/artifacts/13257020164).
- [Combined native captures](https://github.com/jgraving/slot.sp/actions/runs/41692152466/artifacts/13259275746).
- [PR #190 worklog](https://github.com/jgraving/slot.sp/blob/fix/ra-background-progress/WORKLOG.md).
- [PR #191 worklog](https://github.com/jgraving/slot.sp/blob/feat/fork-ota-time/WORKLOG.md).

Artifacts were present/unexpired at handoff. Recheck before sharing later. Later handoff commits modify documentation only.

## Final requested behavior

- RA prepares the GBA library and uploads unlocks while slot is awake, on the shelf or during gameplay. The old five-minute rescan could reset an unfinished pass and its denominator; that is fixed. Settings show separate game-data/artwork counts, pending uploads, failures and retries. A controller indicates active syncing.
- Credentials are always encrypted; no Unencrypted or Session only selector. Wi-Fi/RA require a PIN and unlocked vault. New button PINs require six or more presses; older short PINs decrypt for upgrade before network access. Old plaintext migrates only after successful encrypted save. Separate parent codes stay four buttons.
- Volume, brightness, warmth and kid-time capsules are neutral with adaptive ink. Charging stays green. Wi-Fi arcs are wider and flatter.
- The random Credentials unlocked message was stale UI feedback, not evidence that Wi-Fi disconnected. Page entry clears it; normal unlock succeeds quietly.
- OTA targets jgraving/slot.sp releases, verifies integrity, embedded identity and forward ancestry, and stages System files for explicit restart. Config, games and saves are preserved. BaseOS is separate and model/compatibility/power gated.
- Automatic time is opt-in, works after actual Wi-Fi connection, and preserves the chosen UTC offset and existing manual clock UI.

## Remaining gates

No physical firmware installation was performed. Test actual BaseOS installation/recovery and launcher update interruption on the handheld before merging #191. Confirm live Wi-Fi/RTC behavior and deferred RA award acceptance; do not promise fully offline awards. Local interrupted-install simulation preserved executable mode, custom System files, Config, Games and Saves.

#191 is stacked on #190; retarget main after #190 merges. Both drafts are mergeable after documentation synchronization. No follow-up merge/release was requested. Keep the branch worklogs current and push checkpoints. Never log actual secrets.

Next-chat prompt: Continue jgraving/slot.sp PRs #190 and #191. Read CURRENT_WORK.md on fix/ra-background-progress, then the linked worklogs and live checks.
