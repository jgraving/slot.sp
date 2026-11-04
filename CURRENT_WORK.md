# Device services and settings follow-up

Active branch `fix/device-services-ux`, based on main `1d45793`. See the newest entry in WORKLOG.md for the RG34XXSP Wi-Fi report, implementation and verification status. Physical DHCP and Finder checks remain pending.

---

# Wi-Fi diagnostics complete

Continue from `main`. PR #200 merged as `b4bf214e6ece23d0fba7f85dbcf4e39f951bbf71`; its tree equals source `6053032e41426b4d83e1c3bb788151f866327728`, which passed all six workflows. The post-merge ARM64 release build also passed. [Published build](https://github.com/jgraving/slot.sp/releases/tag/build-2026.11.04-b4bf214) includes Settings → Network → Save Wi-Fi report, progress through IP assignment, address-timeout/disconnection warnings and late recovery.

[Replied to demifiend9](https://github.com/jgraving/slot.sp/issues/182#issuecomment-6633575440) with the exact release, offline report instructions and the maintainer's lack of an RG34xxSP. Next evidence needed is the reporter's Diagnostics/wifi-NNNN.txt; the device-specific root cause is not confirmed. Do not reconstruct this work or promise a hardware fix. See the newest WORKLOG.md entry. Earlier active branch instructions are superseded.

---

# OTA safety review complete

Continue from `main`. PR #198 (`fix/ota-install-recovery`) merged as `b8b359fd1be4507888b6d5f114309384e2791ca9` after all five workflows passed on source `0cb14a5f941fa6c5ca2d93c6639bd6e85493c793`; the merged tree matches the tested source. See the newest WORKLOG.md entry and OTA-REVIEW.md for findings, recovery behavior and hardware limits. This supersedes earlier active-branch instructions. No handheld installation was performed.

---

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
