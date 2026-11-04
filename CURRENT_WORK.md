# Current handoff — Studio, sharing and security merged

Updated 2026-11-04 (Europe/Berlin). Continue from **main**. Read AGENTS.md, then WORKLOG.md before new work.

[PR #194](https://github.com/jgraving/slot.sp/pull/194) merged as `286077e1854cd0c305afcbedfb18533fa73bc7d2` after all six workflows passed on exact runtime `7d0334ca5f88714e07e6e7ba774f7ad6d6c58e20`. The merged tree is identical to that verified source. The slot/About changes from #195 are preserved. Earlier draft-only instructions in historical logs are superseded; do not reconstruct or reopen completed work. ZIP-ROM design draft #113 remains separate.

## Completed behavior

- Settings → Gameplay → Cart Studio edits labels, shell colors/finishes and GB shape. Appearance works offline; artwork matching uses Wi-Fi. Original labels survive read failures and mixed-case filenames; generated titles resize with the shell. Existing Select controls, artwork transparency, achievements/audio and kid/game/link restrictions remain.
- Settings → Network → Network sharing offers password-authenticated encrypted SMB3 uploads. Only Inbox is shared. Explicit stop/import adds games and validated PNG wallpapers without replacing installed files. Sharing defaults off after reboot; opt-in restoration requires vault unlock and the selected network. Disconnects revoke sharing even when paused; unrelated credential saves no longer interrupt uploads. Empty imports report No new uploads.
- Samba uses a private loader/library tree, corresponding-source package and OTA size checks. Safe rebuilds refresh dependencies and complete provenance instead of retaining stale libraries.
- Credential review fixes orphan plaintext migration cleanup, explicitly clears Argon2 working memory, blocks RA follow-up requests/late tokens after sign-out, and closes previous launcher-owned Wi-Fi sessions before locked startup. Existing encrypted vaults remain compatible.

## Verification and evidence

All six workflows pass on `7d0334c`: [ARM64/server/device/package](https://github.com/jgraving/slot.sp/actions/runs/41828785364), [launcher/native](https://github.com/jgraving/slot.sp/actions/runs/41828785370), [website](https://github.com/jgraving/slot.sp/actions/runs/41828785342), [scaling](https://github.com/jgraving/slot.sp/actions/runs/41828785417), [blending](https://github.com/jgraving/slot.sp/actions/runs/41828785340), [shortcuts](https://github.com/jgraving/slot.sp/actions/runs/41828785488).

- ARM64 evidence includes real encrypted wire transfers, wrong-password/SMB2 rejection, isolation and teardown; 154 core tests, 32 achievement tests, 15 editor/upload tests, 57 Studio tests, nine updater tests, both bundled cores and package validation pass. The local sandbox-blocked Unix-socket test passed in CI. Existing live-account tests remain explicitly ignored; the ignored native fixture runs separately and passes.
- Local combined integration suites: 332 pass. Final device-feature/UI checks and shell security/update tests pass. Exact-source light/GLES640 and dark/GL720 Studio/network walkthroughs pass; 124 screenshots validate. Full CI also passes photo geometry, About reversals and all cartridge insertion/ejection sequences.
- [Test System and matching sources](https://github.com/jgraving/slot.sp/actions/runs/41828785364/artifacts/13321292975) and [native review captures](https://github.com/jgraving/slot.sp/actions/runs/41828785370/artifacts/13321509656) were present/unexpired at handoff. One-day retention; recheck before sharing later. The System ZIP is the update; the separate sources ZIP is source material.

## Security scope and remaining checks

Read [SECURITY-REVIEW.md](https://github.com/jgraving/slot.sp/blob/main/SECURITY-REVIEW.md) for findings, protections, advisory reachability and limitations. Six-button codes permit offline guessing from a copied SD card; a long unpredictable code/passphrase is stronger. The review does not prove absence of every flaw, encrypt the full card or protect an already compromised/unlocked device. Old ZIP automatic-extraction advisories are mitigated by the custom unpacker and tested hostile-metadata rejection, not by upgrading that dependency.

Physical PC/Mac discovery, handheld transfers/SD behavior, reboot/lid/power and thermal behavior remain device checks. No real credentials, live awards or physical firmware installation were used. Earlier RA/RTC/OTA physical checks also remain testing notes, not unmerged feature work.

The normal [main build/release](https://github.com/jgraving/slot.sp/actions/runs/41831780938) and [main launcher checks](https://github.com/jgraving/slot.sp/actions/runs/41831781293) started after merge; completion is not yet claimed. [WORKLOG.md](https://github.com/jgraving/slot.sp/blob/main/WORKLOG.md) holds the full recovery history and evidence.
