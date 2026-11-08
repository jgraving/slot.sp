# Studio controls and alignment — publication authorized

Branch: fix/studio-controls-alignment, based on main 41a8774 (merged #217). Read WORKLOG.md. Finish/Shape A cycling, left-aligned menu labels, color confirmation/finish preservation, palette scrolling and RGB/hex improvements are implemented and committed locally.

75 focused Studio tests, 1 palette visibility test, 36 legend/menu tests and 2 artwork tests pass. Native light/GLES640 and dark/GL720 walkthroughs passed; the final scroll correction was clean-built and visually verified in the light walkthrough. An unrelated Unix-socket library test is sandbox-blocked; physical handheld and ARM64/CI checks remain pending.

User approved pushing to jgraving/slot.sp and opening a PR. Shell git push lacks HTTPS credentials; use the authenticated GitHub connector to publish the same source. Verify the published tree matches local HEAD, open the PR and run CI. Merge has not been authorized for this revision.
