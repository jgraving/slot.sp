# OTA safety review

Reviewed 2026-11-04 on `fix/ota-install-recovery`, PR #198, against main `9f436a4`. This is a code and simulated-failure review, not certification of physical SD-card power-loss safety.

## Findings corrected

| Failure | Correction and evidence |
| --- | --- |
| Existing `System` parent symlinks could redirect installation outside `System`; destination directories could cause a false-success replacement. | Validate the staged and installed path components before the first replacement. Real shell-installer tests reject links and directory collisions without modifying user data or the external target. |
| A conflicting temporary directory could redirect a copy because shell errexit is suppressed inside the error-handled put function. | Preflight temporary paths and explicitly return on mkdir/rm failure. The regression reproduced an external-target overwrite before the fix and passes afterward; injected preparation failures stop before copying. |
| Interrupted cleanup could leave a ready marker alongside partly deleted staging and block the next boot after a successful install. | After the installed tree sync succeeds, remove and sync the readiness marker before deleting recovery files. A process-kill regression reproduces the blocked retry before the fix and passes afterward. |
| A recovery source not fully flushed before replacement weakens retry after interruption. | Sync staging before the first replacement; retain it until the final installed-tree sync succeeds. Tests fail both pre-install and post-copy sync and verify retry. |
| ZIP metadata could understate expanded length and bypass aggregate accounting. | Require actual decompressed length to equal declared length, alongside existing per-file/total bounds, CRC, path and symlink checks. Crafted metadata regression passes. |
| Launcher staging ignored failure to remove an incomplete old tree. | Abort on failed cleanup and require exclusive creation of a fresh staging directory. Reject a redirected staging root. |
| BaseOS temporary downloads could follow a pre-existing symlink, and failed attempts could delete an already-ready payload. | Refuse existing ready paths and non-regular temporary paths; create a fresh temporary file exclusively. Only remove a published payload if this attempt published it. Tests preserve existing payloads and symlink targets. |

## Verified behavior

- Release selection uses the exact official repository/asset HTTPS URLs and requires a SHA-256 asset digest. Launcher build metadata must match repository, product, architecture and release source; GitHub ancestry suppresses older/diverged launcher offers. BaseOS selection requires the exact device model and a newer stable numeric version.
- Truncated downloads, wrong digests, archive traversal, duplicate/reserved/control-character names, symlink entries and oversized entries fail before a ready launcher manifest is published. Extraction stays inside `System.new`; Config/Games/Saves are not destinations.
- The launcher updater replaces individual files atomically, with the launcher binary and startup script last. Complete staging remains until replacements are synced; the readiness marker is then durably retired before cleanup. Failure stops startup: the launcher does not execute a partially installed tree. This is **retry recovery, not whole-tree rollback**.
- Partial-copy failures and installer process death at all seven fixture replacement positions (14 injected cases) retain staging; retry completes with expected file modes and unchanged games, saves, configuration and unrelated System files.
- BaseOS tests cover no space, download failure, cancellation, short data, incorrect digest, rejection, missing status, a verdict for the wrong payload, stale partial retry, ready-file preservation and redirected paths. Failure leaves no newly staged bootable payload in these fixtures.
- The UI requires Wi-Fi and explicit selection. BaseOS starts only with a known battery of at least 20% or charging/full status. Cancellation cannot interrupt committed staging; shutdown/restart waits for the update job to settle.

Local evidence: all **15 Rust updater integration tests**, **seven shell-installer recovery test methods** (including the 14 injected interruption cases), and existing updater/launcher shell suites pass. GitHub launcher/device/release workflows run these regressions. Exact-head CI completion is recorded separately in WORKLOG.md.

## Limits and recovery expectations

- Process death and mocked I/O failure are not physical power cuts. FAT/exFAT metadata, SD controller behavior, media failure and manual card removal still need device testing. A full/read-only/damaged card can prevent retry; free space or repair the card and retain `System.new` until the installer succeeds. Config/Games/Saves should be backed up before hardware update experiments.
- These boot-installer checks protect updates once the corrected installer is installed. An update from an older installation is initially executed by that installation's existing updater.
- Checksums obtained from GitHub authenticate against the trusted release metadata over TLS; they are not independent release signatures. Compromise of the release account/build pipeline or local privileged tampering remains outside this protection. Path checks do not claim resistance to a concurrent privileged process rewriting the filesystem.
- The external BaseOS firmware writer is a separate trust boundary. Inspection of [upstream baseos-update at faa64a85](https://github.com/pvaibhav/BaseOS/blob/faa64a85f057fe4cdbe10c795406924a4d57d42e/overlay/usr/sbin/baseos-update) shows writing the inactive slot, read-back hashing, a GPT flip and trial-boot recovery. It explicitly has no signing key. This inspection does not establish which writer version is installed on the user's device, validate every boot/GPT helper, or prove interruption safety at its commit point. No firmware was flashed in this review.
