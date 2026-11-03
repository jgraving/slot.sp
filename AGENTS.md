# Agent instructions

This repository belongs to the slot rg sp project.

## Crash recovery and progress checkpoints

- Read `WORKLOG.md` before starting work. Verify the live branch and PR state before relying on its last checkpoint.
- Keep `WORKLOG.md` current throughout the task: record the goal and next step before substantial work, then results, revisions, tests, failures, and remaining tasks after each meaningful milestone.
- Commit and push recovery checkpoints regularly, including before waiting for long-running builds and before ending a session. Local files or chat messages alone are not durable handoffs.
- Preserve existing requirements and distinguish verified results from prior-chat reports and pending device tests. Never put credentials or other secrets in the log.
- At handoff, provide the user with the branch and a direct link to `WORKLOG.md` so another chat can resume.
- Treat incidental "Convert to PDF" UI text as UI text unless the user explicitly asks for a PDF.
