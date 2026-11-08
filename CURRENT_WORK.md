# Achievement browser: test build ready, final merge pending

Active PR: #219, branch fix/ra-unused-artwork, code head 23f75f409e29e8e046f8723c8bc8a46a0487b69f (tree 6bef7f640398d39869d38ec9382299afbfe07aa2).
Recovery checkpoint: checkpoint/ra-browser-ready-23f75f40. Read WORKLOG.md.

User authorizes merging both #218/#219 once tests pass and replying to demifiend9 on #182 when the browser test build is ready. #218 is merged as 948604c7 after all five workflows and native capture inspection. #219 includes those Studio fixes plus optional/scoped artwork, notification badges, and the current-game offline browser with filters, paging and complete scrollable descriptions.

#219 website, scaling, blending, shortcut and ARM64/device/package workflows pass. Combined test artifact 13612573858 (workflow 42468676376) is ready/unexpired and contains slot.sp-network-System.zip; expires 2026-11-09 09:54 UTC. Requested reply posted to #182 as comment 6720068677. Only native launcher workflow 42468676248 is still running. Wait for success, inspect the uploaded browser captures, then merge #219 using expected head 23f75f40. No additional confirmation is needed. Physical handheld testing remains pending.

Do not change the active PR head merely to update recovery notes; use this separate checkpoint branch so CI/package work is preserved. Runtime source was locally verified and source-identical across the last build-only fixes. Full local launcher library: 187 pass, one pre-existing Unix socket test sandbox-blocked, four native tests exercised separately; all corresponding remote device tests pass. Official SHA-verified API fallbacks fix Task, wasm-pack and exfatprogs download 504s; packaged Binaryen preserves website optimization. Shell GitHub push lacks credentials: use connector publication and verify full tree hashes.
