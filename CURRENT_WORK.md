# RA Download for selector: CI running

PR #221, fix/ra-download-selector, head 3386b4d589f52549cca5e2bfbf4dc9069f4828e4, tree ca52846b9b4be2bafd08cf6f310f27095ed622b2. Recovery branch checkpoint/ra-download-selector-3386b4d5. Read WORKLOG.md.

User explicitly requests paired chevrons and D-pad Left/Right on RA Download for, A must not change it; merge after passing checks. Implemented adjustable row, A no-op without writes/service updates, fitted long value and compact D-pad footer beside download progress. Artwork toggle still supports A. Scope/download pacing unchanged.

49 network tests pass, with both-direction persistence/wrap and A preserving scope/config/revision. Dedicated clean-built native RA review passes; inspected light640 longest selector with progress, complete chevrons and visible footer controls. UI library/menu/legend suite running. Physical device verification pending.

CI: native 42491467754, device/package 42491467776, pages 42491467796, scaling 42491467869, shortcuts 42491467809, blending 42491467837. Next: finish local UI checks, verify all CI, inspect uploaded selector captures, merge #221 using expected head 3386b4d5, and record completion. Do not ask again for merge permission or move feature head for status-only docs. #218/#219/#220 already merged.
