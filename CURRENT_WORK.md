# Studio controls and alignment — PR #218

Active PR: https://github.com/jgraving/slot.sp/pull/218
Branch: `fix/studio-controls-alignment`, based on main `41a8774` (merged #217). Read WORKLOG.md.

Implementation published at `02257f45c962ba1aea67b06264108415f76335ec` with tree identical to local validated HEAD. Fixes Finish/Shape A cycling, left-aligned menu labels, palette confirmation/finish preservation, scrolling, and RGB/hex presentation. 112 focused local Rust tests and 2 artwork checks pass. Native light/GLES640 and dark/GL720 walkthroughs passed; final scroll adjustment was clean-built and visually checked in light. A broader Unix-socket test was sandbox-blocked.

User approved publishing and opening the PR. Shell HTTPS credentials were unavailable, so the authenticated connector published the same files. Remote CI is pending; inspect exact-source workflows and uploaded native captures before any merge. Physical handheld testing remains pending. Merge is not authorized by the current approval.
