# Credential and network-sharing security review

Reviewed 2026-11-04 for [PR #194](https://github.com/jgraving/slot.sp/pull/194), including its integration with the current main branch. This is a source, regression and packaged-server review, not a guarantee that every flaw has been found or a physical-device penetration test. Only synthetic credentials are used in tests.

## Credential protection

Wi-Fi passwords, the RetroAchievements login password/token and the independently generated Samba password are stored inside the same authenticated encrypted vault. A device code or passphrase derives the key with Argon2id v1.3 (19 MiB, two passes, one lane), matching the [OWASP minimum Argon2id configuration](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html). This is a minimum work factor, not a claim that short codes are strong.

Each vault has a random 16-byte salt; each save uses a fresh random 12-byte nonce. ChaCha20-Poly1305 authenticates the ciphertext and public header. The PIN and derived key are not persisted. Reads are bounded and saves are atomic. Wrong secrets and modified ciphertext/header are rejected. This review preserves the existing format and key derivation, so existing vaults remain readable.

Wi-Fi credentials pass through the supplicant control socket, not process arguments or a saved supplicant configuration. RA uses HTTPS with redirects disabled; request fields and retired token copies owned by the app are cleared. No direct PIN/parent authentication bypass, credential-bearing URL, plaintext RA cache or secret logging was found in the reviewed paths. The separate parent code is a parental-control mechanism and is not a vault encryption key.

## Findings corrected

| Finding | Correction and regression coverage |
| --- | --- |
| Orphaned legacy plaintext credential writes could survive migration, including PID reuse. | Scan credential-specific residues independently of the generic temp sweep. Require authenticated encrypted save and successful cleanup before networking; never import stale orphan data over authenticated secrets. Cover new/existing vaults and cleanup failures. |
| Argon2 working memory was released without explicit clearing. | Own and zeroize the working blocks as well as derived key/input buffers; verify unchanged key derivation and existing-vault compatibility. |
| RA could continue follow-up requests after sign-out during a blocked request. | Gate API/artwork requests, discard late responses/tokens, clear retired token/request copies. Deterministic stalled-login/lookup tests preserve pending awards and prevent follow-up work. An already transmitted request cannot be retracted. |
| A launcher restart could leave its previous owned Wi-Fi session connected behind the locked UI. | Stop that session before update/launcher startup; preserve externally managed Wi-Fi. Exercise the production helper with mocked system commands. |
| Background credential saves could interrupt an active SMB upload. | Preserve an unchanged active/starting sharing session; first enablement and password rotation still wait for encrypted storage. |
| A dropped connection while sharing was paused could silently restore the server on reconnect. | Revoke enablement on Wi-Fi/address loss regardless of server state; retain explicit reboot opt-in before the first connection. |
| Rebuilding Samba could retain old private dependencies and incomplete provenance. | Recreate only recognized generated runtime output, preserve source/build caches, and regenerate dependencies/provenance; refuse unsafe paths. |

## Sharing boundary

Sharing is off by default after reboot. Opt-in restoration requires vault unlock and the selected saved network. Samba requires password-authenticated encrypted SMB3, exposes only the upload Inbox, disables following symlinks, and shuts down for disconnects, launcher loss, gameplay/editor/kid/link sessions and power transitions. Password rotation closes existing sessions. Import does not overwrite installed games or wallpapers; wallpapers are fully decoded with encoded-size, dimension and pixel limits.

The server uses the current Samba 4.25.1 release and a private loader/library tree, without replacing launcher system libraries. Official sources: [release notes](https://www.samba.org/samba/history/samba-4.25.1.html), [security releases](https://www.samba.org/samba/history/security.html). The release includes the latest listed fix; the affected optional Ceph module is disabled in this build. Dependency cache refresh and corresponding sources remain part of packaging.

## Dependency advisory scope

Compared Cargo.lock with the RustSec advisory database at `965bf9ad083b58e6340d65829d85217b18be9290` (2026-11-03), examining version ranges and reachable APIs. This was a source-level advisory review; cargo-audit was not installed or run.

The retained zip 0.6.6 dependency matches [RUSTSEC-2026-0367](https://github.com/RustSec/advisory-db/blob/965bf9ad083b58e6340d65829d85217b18be9290/crates/zip/RUSTSEC-2026-0367.md), [0368](https://github.com/RustSec/advisory-db/blob/965bf9ad083b58e6340d65829d85217b18be9290/crates/zip/RUSTSEC-2026-0368.md), [0369](https://github.com/RustSec/advisory-db/blob/965bf9ad083b58e6340d65829d85217b18be9290/crates/zip/RUSTSEC-2026-0369.md) and [0429](https://github.com/RustSec/advisory-db/blob/965bf9ad083b58e6340d65829d85217b18be9290/crates/zip/RUSTSEC-2026-0429.md). These concern automatic extraction APIs; the updater does not call them. Its custom unpacker rejects symlinks and invalid paths, limits entry/total/actual read sizes, and masks file permissions to 0777. Targeted hostile-archive tests document those protections. A future migration to a maintained ZIP version remains worthwhile; this review does not label the dependency itself patched.

`ttf-parser` 0.25.1 also has a [maintenance-only advisory](https://github.com/RustSec/advisory-db/blob/965bf9ad083b58e6340d65829d85217b18be9290/crates/ttf-parser/RUSTSEC-2026-0192.md); the app uses embedded fonts. No applicable known crypto/TLS advisory was found for the locked versions in this database snapshot. A source-level lookup does not establish absence of undisclosed vulnerabilities.

## Limits that remain

- Six random presses from 12 buttons provide only 2,985,984 combinations (about 21.5 bits); predictable patterns are weaker. Anyone with a copy of the SD card can guess offline without the UI's limits. Use a long, unpredictable code or passphrase for meaningful resistance; no cracking-time estimate was measured.
- Games, saves, the operating system and the rest of the card are not encrypted. Modified software, a compromised running device or an already unlocked session can access credentials. There is no secure-boot/secure-element claim.
- Deleting old plaintext from flash is not forensic erasure. Third-party TLS/HTTP/OS components can retain transient secret copies beyond the buffers this code explicitly wipes.
- A saved SSID is not cryptographic router identity. Use a trusted network; discovery and router behavior still require Windows/macOS and physical-handheld testing.
- Physical transfers, suspend/power behavior and thermal performance remain unverified on the handheld. No real RA credentials or live awards were used for this review.

## Verified completion

PR #194 merged into main as `286077e1854cd0c305afcbedfb18533fa73bc7d2`. All six workflows passed on exact source `7d0334ca5f88714e07e6e7ba774f7ad6d6c58e20`, including real ARM64 SMB authentication/encrypted-transfer/isolation tests, device/package tests and native light/dark review. The locally blocked Unix-socket test passed in CI. Hostile ZIP metadata, vault compatibility/tampering, migration cleanup failures, RA cancellation and sharing-session regressions pass. No test assertion was disabled to obtain this result.

Detailed revisions, results, exceptions and artifact links are recorded in [WORKLOG.md](WORKLOG.md). The hardware and threat-model limits above remain unchanged.
