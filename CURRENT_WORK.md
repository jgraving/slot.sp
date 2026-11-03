# Current development handoff

Updated 2026-11-03. Read this first, then each branch's WORKLOG.md and live PR checks. Historical worklog sections contain superseded requests.

## Current follow-up: final CI in progress

PR #186 merged as 6d499951289899fbf2314ecee04d0f69a03a2b28. Follow-ups remain drafts:

| PR | Branch | Current runtime | Verification |
| --- | --- | --- | --- |
| [#190: RA / Device code / Parent code](https://github.com/jgraving/slot.sp/pull/190) | fix/ra-background-progress | 285cfa1e586dabffe1720b3cdcd13f1047230d4f | ARM64/package, website, scaling, blending and shortcuts pass; launcher/native run 41730841530 pending |
| [#191: General update/time tools](https://github.com/jgraving/slot.sp/pull/191) | feat/fork-ota-time | 92c2e8a470c30ca3add237780d1b32fc932a8156 | Scaling/blending/shortcuts pass; device 41731781331, launcher 41731781266 and website 41731781278 pending |

The runtime source is frozen unless a check fails. Later recovery commits are documentation only. Do not repeat implementation.

## Implemented behavior

- Use **Device code** and **Parent code** consistently. Credentials are always encrypted; new controller device codes need six presses, separate parent codes stay four. Legacy short device codes unlock for upgrade. Typed passphrases remain supported.
- Code cards start with zero dots. Device setup/confirmation dims Done until six presses; early Done shakes without clearing and gives the minimum in a short pill. Silent variable-length unlock remains automatic; no stored-length hint is exposed.
- Menu cancellation of a parental device-code prompt returns immediately to the prior adult menu or child settings, without a blank unlock card.
- RA keeps game, artwork and upload progress separate. Cached games count after restart; initial reading states use words instead of 0/0. The controller remains visible for queued/retrying work. The neutral progress pill floats outside the card and includes retry countdowns. Up to date requires all queued work to finish.
- Cached artwork discovery/full PNG checks run in bounded worker batches. Shared unchanged badges are not repeatedly inflated; requests are paced, artwork retries back off, and ordinary download failures preserve account auth. Actual expired/invalid credentials still request reauthentication.
- General starts Device code, Updates, Automatic time, Time zone. Disconnected update/time rows are grey; A opens Network with a feature-specific Wi-Fi-needed pill and B returns to the original row. Time zone opens the offset directly; clock/offset edits apply immediately and preserve UTC.
- OTA targets jgraving/slot.sp releases with integrity/identity/ancestry checks and explicit restart. BaseOS remains separate, target/compatibility/power gated. Automatic time is opt-in. No upstream plaintext Wi-Fi setup is imported.
- Volume, brightness, warmth and kid-time meters remain neutral/adaptive; charging stays green. Wider Wi-Fi arcs and quiet normal unlocks are retained.

## Evidence and next steps

Local: 140 combined core tests pass with one ignored native fixture and the sandbox-blocked Unix socket test excluded; clock 19/19, launcher quick-menu 58/58, UI library/quick-menu 61/61, achievements 31/31. The Unix socket test remains enabled in CI. Shared ARM64 CI passed, including both bundled cores.

Production light/GLES and dark/GL captures were reviewed for floating RA phases/retry/completion, disabled General tools, immediate UTC offset, zero/partial/ready code entry and direct parent cancellation/reopen. Final dark walkthrough passed with 47 valid PNGs. A prior local run had incomplete PNG output and a final fixture-marker failure; a clean traced rerun passed, confirmed synchronous removal at the expected root, and tracing was removed. Keep native assertions.

- Current shared System artifact: [13276730948](https://github.com/jgraving/slot.sp/actions/runs/41730841353/artifacts/13276730948), present/unexpired when checked. Prefer a new combined artifact once #191 passes.
- [RA worklog](https://github.com/jgraving/slot.sp/blob/fix/ra-background-progress/WORKLOG.md).
- [OTA/time worklog](https://github.com/jgraving/slot.sp/blob/feat/fork-ota-time/WORKLOG.md).

Next: verify all six workflows on both exact runtime SHAs, inspect completed device logs and artifact IDs, update PR validation/build links, then checkpoint final evidence in this index and both logs. Recheck artifact availability before sharing. Carry the latest parent documentation into #191 to keep the stacked PR mergeable.

No live account secrets were used. Optional manual RA smoke workflow uses RA_TEST_USERNAME and RA_TEST_PASSWORD only for login, read-only lookup and public artwork, without awards or game sessions. It is not part of ordinary PR checks.

Hardware heat/radio/RTC behavior, live deferred-award acceptance and real BaseOS installation/interruption/recovery still need handheld testing. No physical firmware installation ran. Do not claim these are verified or merge/release without a new request.

Next-chat prompt: Continue jgraving/slot.sp PRs #190 and #191. Read CURRENT_WORK.md on fix/ra-background-progress, then the branch worklogs and live checks.
