# ZIP ROM support — design for review

Status: proposal only. No runtime support is implemented by this PR. Do not merge or deploy an implementation without explicit review approval.

## Goal and initial scope

Let slot.sp and Cart Studio read a game stored in an ordinary ZIP inside the existing `Games/GBA`, `Games/GB` or `Games/GBC` folders. Keep raw ROMs working as they do today. Decompress in the frontend; do not modify emulator source, patches, binaries or versions.

Proposed first version:

- Exactly one eligible ROM per archive, using the same extension/platform rules as a raw game in that folder. Permit unrelated readme files and ignore directory entries and macOS metadata consistently.
- Support stored and Deflate members. Reject encryption, unsupported methods and multi-volume archives clearly. Decide ZIP64 handling explicitly before choosing a reader.
- Reject ambiguous archives rather than silently choosing their first ROM. Do not add nested archive support, a multi-game picker, 7z/RAR, or archive creation/repacking.
- Read only: do not modify archives or extract temporary files to the SD card. Studio keeps all processing in the browser and continues exporting labels, not games.

## What needs to change

Today the device scanner accepts raw extensions, its header helpers open raw file offsets, and the libretro wrapper reads the selected file into a ROM buffer. Studio filters the same extensions, reads a header slice, and streams the file for CRC identification. Its existing ZIP writer exports labels; it is not an archive reader.

Adding `.zip` to the extension filters alone would feed compressed data to those paths and is not sufficient.

Propose one bounded Rust ROM/archive reader with native and WASM adapters, rather than separate ZIP selection rules in JavaScript and on the device. Keep archive selection, header extraction, decoded size checks and CRC validation consistent. Choose the dependency only after checking WASM compatibility, supported compression methods, licensing, binary size and memory behaviour. No dependency is selected here.

The emulator-facing wrapper must receive the decoded ROM bytes and an appropriate virtual ROM filename with the original ROM extension. Verify both bundled cores' path and sidecar expectations. If either core requires a real extracted file, stop and review the design instead of silently adding a writable extraction cache.

## Compatibility decisions before implementation

| Area | Proposed rule / question to resolve |
| --- | --- |
| Game identity | Use the outer archive basename as the existing cart stem. `Tetris.gb` becoming `Tetris.zip` should retain `Tetris` as its identity. Do not derive saves from a different inner filename. |
| Saves and states | Preserve existing save, state, resume and RTC locations and core selection. Verify raw-to-ZIP and ZIP-to-raw round trips with identical ROM bytes, including cold boot, resume, save-state loading and link reload. |
| Labels and parental controls | Preserve label names, shell settings, Favorites, Recents and kid approvals. An archive with several games must not expose an alternative game under one approved cart. |
| Duplicate files | Never create two carts with the same platform/stem or silently change which ROM runs. Decide whether to prefer the existing raw ROM with a notice or reject the conflict; apply the same rule in Studio. |
| Artwork | Calculate identity from decompressed ROM contents. A ZIP directory CRC may be a hint, not a substitute for validating the actual decoded data. Different compression levels must identify the same game. |
| Naming | Test differing outer/inner names, case, Unicode and members inside subdirectories. Preserve current folder-based GB/GBC classification. Renaming the outer file still changes its existing filename-based identity; do not promise automatic save migration. |
| Sidecars | Audit BIOS, patch, cheat, RTC and core filename handling. Do not silently lose behaviour that works for a raw ROM; document unsupported combinations explicitly. |

## Resource and failure requirements

- Bound archive size, directory size, member count, filename length and actual decompressed bytes, not merely the sizes claimed by the archive. Choose limits that accommodate supported ROMs and documented larger homebrew/hacks.
- Validate local/central-directory consistency, truncated streams and CRC errors. Reject unsafe member names, symlink members and malformed offsets; never use a member path as an output filesystem path.
- Avoid keeping full compressed and decompressed copies at once. Stream decoding where possible, and release buffers promptly. Measure peak memory on device and in the browser.
- Do not fully inflate every game at startup or on each carousel move. Measure header/cache behaviour on a large collection and invalidate cached information when the archive changes.
- Allow Studio to cancel or replace an in-progress read. Keep failures local to the affected cart with understandable messages.
- A failed load must not overwrite saves, resume data, labels or approvals, and must not be reported as SD-card corruption. Existing raw-file behaviour must remain unchanged.

## Review and test gates

1. Agree on archive selection, identity, duplicate precedence, limits, ZIP64 and sidecar policy.
2. Test the shared reader with synthetic raw/ZIP pairs: stored/Deflate, different compression levels, differing names, Unicode, subdirectories, harmless extras, multiple ROMs, wrong-platform contents, encrypted/corrupt/truncated archives, false size claims, excessive entries and path/symlink attacks.
3. Establish native/WASM parity for selected member, header bytes, decoded bytes, CRC, platform and rejection reason. No network access is needed for these fixtures.
4. Demonstrate unchanged save/RTC/state/resume paths and label/approval associations across compression and decompression. Verify failure leaves existing data byte-for-byte intact.
5. Test actual loading through both bundled cores, including core selection, resume, screenshots and link reload. Confirm emulator materials are unchanged.
6. Test Studio folder access and read-only folder/ZIP-label export modes: recognition, existing labels, custom labels, cancellation, duplicate handling and write/export failure. Games must never be written or uploaded.
7. Measure boot scan, first launch, repeated launch, browser responsiveness and peak RAM on representative hardware and larger supported games. Do not claim an FPS or memory benefit without measurements.
8. Review the device and Studio changes separately. Coordinate the Studio renderer/shared-reader pin and website integration so neither side advertises unsupported archives. Keep release notes explicit about the supported ZIP subset.

## Suggested delivery

Start with the shared reader and fixture tests, then separate device and Studio draft PRs. Keep the installed OS and published Studio unchanged while these are reviewed. This document is a review checklist, not approval to ship ZIP support.
