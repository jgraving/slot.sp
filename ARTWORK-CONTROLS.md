# Legacy artwork storage

The **Legacy artwork storage** Actions workflow audits or clears only retired `studio-art-vN-` cache keys and completed `github-pages` upload bundles from the Pages workflow. It leaves source branches, release files, ordinary review artifacts, the live website and handheld files intact.

## Use

1. Open **Actions → Legacy artwork storage → Run workflow** on `main`.
2. Leave **Clear** unchecked for an audit. Review the workflow summary.
3. To remove the matching generated storage, run again with **Clear** checked.

The cleanup job runs manually on `main` after its boundary tests pass. It uses repository Actions write permission; it has no branch-write or website-deployment permission. Active Pages runs are excluded and checked again immediately before deleting an upload bundle. The website's normal build publishes an empty artwork catalog and does not restore these retired caches.

## Implementation progress

Branch: `maintenance/legacy-artwork`. Adds the workflow, bounded selection/deletion helper and regression tests. Scope and validation are recorded here; the main website/preview cleanup is tracked separately in CLEANUP-WORK.md after it merges.
