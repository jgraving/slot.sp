# Active work: device shell logos and model refresh

Branch: `refresh/device-shell-logos`, based on main `60558b65ad5039f789155ede1cce68c19a9b55a8`.

Read [CLEANUP-WORK.md](CLEANUP-WORK.md) for current implementation, validation and remaining work. Cleanup PR #211 and browser editor PR #26 are merged. Maintainer completed the one-time retired artwork cache deletion. No cleanup controls were added.

Current changes: cached direct-upstream device shell logos, lowercase `slot.sp` fallback marks, and removal of the website model's lid and underside brand engravings. Run source CI and regenerate preview media before merge. Physical-device networking remains unverified.

Next separate task: compare upstream on-device Studio updates and adapt them to our UI. Historical completed work remains in WORKLOG.md; current scoped checkpoints are in CLEANUP-WORK.md.
