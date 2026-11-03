# RetroAchievements implementation proposal

Draft for [issue #182](https://github.com/jgraving/slot.sp/issues/182).
This document proposes an implementation; no achievement tracking is enabled by this PR.

## Recommendation

Port the offline backend from [therealwizywig/slot-extended](https://github.com/therealwizywig/slot-extended/tree/247e6cd829cd72ecb51080a780f923bdc6d06d79/crates/slot-achievements)
at commit `247e6cd829cd72ecb51080a780f923bdc6d06d79`, adapting it to slot.sp's emulator loop rather than merging the fork.
Start with GBA on mGBA and gpSP, casual/softcore only. Preserve resume, Polaroids,
rewind, fast-forward, kid mode and the current UI. GB/GBC need their own validated
memory mappings and should follow separately.

The inspected fork contains a real rcheevos evaluator, cached definitions, an
account-separated persistent unlock ledger, background library preparation, badge
caching and deferred uploads. Its README explicitly says live-account acceptance
and on-device performance still require verification. Treat its tests as useful
evidence, not proof that the port is ready to release.

## Does it require Wi-Fi?

| Operation | Internet needed? |
| --- | --- |
| Existing slot.sp games and saves | No |
| Initial account login and game-definition download | Yes |
| Evaluate a previously prepared game's cached achievements locally | No |
| Restart offline and keep pending local unlocks | Supported by the proposed persistent ledger |
| Show unlocks on the RetroAchievements website | Yes, and the server must accept each upload |
| Newly added or changed ROM with no matching cached definitions | Yes, before tracking can begin |

An internet route is the actual requirement; Wi-Fi is the normal handheld route.
The achievement backend must not control the radio. A home-network connection is
a separate integration task: slot.sp's existing `link_radio.rs` manages peer link
sessions through `System/slotlink.sh`, not general home-network configuration.
Do not assume that a handheld supporting wireless link already has internet access.

The [official FAQ](https://docs.retroachievements.org/general/faq.html#can-i-earn-achievements-offline)
currently says fully offline sessions are unsupported: connect at session start,
and reconnect before closing the emulator if a connection is lost. The fork goes
beyond that by storing definitions and unlocks across restarts. Therefore describe
this as experimental local offline tracking with deferred sync until live testing
establishes what the service accepts. Never label a pending upload as server-confirmed.

## Forks inspected

| Fork | Relevant code | Assessment |
| --- | --- | --- |
| [slot-extended](https://github.com/therealwizywig/slot-extended/tree/247e6cd829cd72ecb51080a780f923bdc6d06d79/crates/slot-achievements) | `storage.rs`, `network.rs`, `runtime.rs`, `core.rs`, `library.rs`, tests | Best match for offline starts, persistence and later sync |
| [nickclyde/slot-plus](https://github.com/nickclyde/slot-plus/tree/e2aefa04ba327e8e7d25d62b0cdd9084866438d0/crates/slot-cheevos) | rcheevos client, HTTP worker, memory adapter, offline tests | Useful second reference; inspected client keeps runtime state in memory, so do not infer restart persistence from retry support |
| [alejandrolm96/slot-retroachievements](https://github.com/alejandrolm96/slot-retroachievements/tree/main/crates/slot-cheevos) | GBA memory mapping and HTTP boundary | Useful memory/transport foundation; the inspected crate alone is not a complete offline service |

## Concrete changes

### 1. Expose a safe memory snapshot in slot-retro

Add libretro memory-descriptor ABI support in `crates/slot-retro/src/ffi.rs`
and retain maps reported through the environment callback in `libretro.rs`.
The existing get-memory functions alone do not establish the full GBA mapping.

Add an optional snapshot method to `RetroCore`, defaulting to unsupported.
Only the thread owning the core reads its pointers. Copy into caller-owned RAM;
never send core pointers to a worker. Keep the rcheevos GBA layout:
32 KiB internal RAM, 256 KiB external RAM, and up to 64 KiB save RAM.
Validate each region's readable extent, including descriptor offset, selection and
disconnect bits. Short or absent regions must remain invalid rather than appearing
as a valid zero-filled buffer. Reacquire mappings if a core changes them.

Test both real cores and synthetic maps. A core without valid memory continues
playing with achievements unavailable.

### 2. Port the isolated backend

Import the chosen fork's `crates/slot-achievements`, limiting the first version's
accepted console IDs and library scanner to GBA. Review and adapt its broader
`RetroCore` wrapper: the fork's trait includes methods slot.sp does not have.
Do not expand slot.sp's trait with unrelated multi-system functionality merely
to make copied code compile.

Keep the backend split into evaluator/storage, HTTPS, library preparation and badge
loading. Gameplay submits to a bounded queue with `try_send`, never a blocking send.
At 352 KiB per full snapshot, copying at 60 fps is approximately 20.6 MiB/s;
fast-forward multiplies this cost. Measure on H700 before choosing queue size and
whether evaluation should remain on a worker.

Preserve the upstream fork's attribution and GPL obligations. It vendors the MIT
rcheevos 12.2.1 evaluator at `40d916de00fe757bab40fb4db41a7912193a48e3`;
carry its license into both source and packaged releases. Review the vendored
revision and build inputs rather than downloading mutable C sources during builds.

### 3. Integrate committed frames explicitly

Hook tracking in `crates/slot/src/emu.rs` after each committed forward frame,
including frames skipped for presentation during fast-forward.

**Do not just wrap every `RetroCore::run_frame` call.** The current rewind branch
unserializes an earlier state and then runs a frame to display it. A generic wrapper
would evaluate that frame too. The integration must distinguish forward play from
rewind rendering and future speculative frames.

A proposed interface, to be implemented and tested:

```rust,ignore
// Called by the emulator owner after a committed forward frame.
tracker.submit(core, game_generation, timeline, sequence);

// Called before resuming evaluation after any state load or rewind.
tracker.reset_timeline();

// Called when a game ends; the service remains alive on the shelf.
tracker.unload(game_generation);
```

Use game generations to discard late results after ejection or cart changes.
Sequence gaps and timeline changes reset hit/delta state before evaluating the
next snapshot. Suppress tracking for the two-player mGBA cable path initially.
Test gpSP peer link separately: cached evaluation may continue, while HTTP waits
for a route without changing the link radio.

### 4. Persist first, notify second

Use an account-separated directory under `Saves/RetroAchievements`, keyed by a
normalized account identifier. Cache game definitions by ROM hash and schema version.

The ledger records achievement ID, ROM hash, original earned timestamp and sync
state. Atomically persist an unlock before showing its notification. Retain
confirmed entries to prevent duplicate notifications after offline restarts.
Only mark an upload confirmed after a successful, validated server response.
Rejected awards stay visible as pending/attention; never silently drop them.

Preserve corrupt files and report a concise error. A failed write must not be
presented as a durable unlock. If the HTTP worker can still upload an unpersisted
event, define and test that outcome explicitly.

On graceful power-off, drain accepted snapshots and finish local persistence
asynchronously. Never wait for network availability. Keep the UI responsive and
surface local storage failures; define a bounded shutdown policy. Sudden power loss
can lose snapshots not yet persisted, even with atomic file writes.

### 5. Configure internet and account separately

Start disabled. Missing config leaves the normal launch path unchanged.
For the first implementation use an opt-in account file, with a token stored
after successful HTTPS login. Avoid logging TOML source lines or HTTP bodies that
contain credentials. Remove a supplied password only after the token is safely
saved. Treat FAT card access as access to the credentials.

Add home Wi-Fi as a separate implementation/PR using a radio owner shared with link
play, not a competing process that reconfigures the interface. Verify DHCP, DNS,
certificate roots, device time and return from link mode on the actual OS image.
If time is invalid, keep pending data and report that HTTPS cannot authenticate;
never disable certificate verification.

Prepare ROM hashes and game metadata in the background with the active game taking
priority. Offline launch uses cache immediately, without a connection timeout.
Definitions for unknown games cannot be reconstructed from the ROM alone.

### 6. Add UI after backend validation

Use slot.sp's existing notification and sound conventions. An unlock shows a cached
badge (or fallback), title and points; it never waits on an image download.
Honor volume, mute and kid-mode restrictions. Keep sync/account state separate
from earned notifications, and distinguish local pending from confirmed.

Account changes must not upload another user's ledger. Disabling the feature stops
tracking and networking while preserving pending data for that account.

## Suggested implementation PRs

1. Memory ABI/snapshot support with mGBA and gpSP checks.
2. Optional GBA evaluator, persistent cache/ledger and fake-server tests.
3. Emulator lifecycle integration, committed-frame/timeline handling and shutdown drain.
4. Home-network connection and radio ownership with link regression tests.
5. Unlock UI, config examples, guide and packaging/license changes.

Keep each PR reviewable and leave the feature disabled until the integration checks pass.
This proposal intentionally does not close #182.

## Acceptance checks

- Missing/disabled config: gameplay, saves, menus and radio behavior unchanged.
- Prepare a supported test ROM online; restart offline; trigger a real condition;
  eject, restart and verify one pending durable unlock and no repeated notification.
- Reconnect on the shelf; verify live server acceptance and the account website.
  Repeat for token expiry, duplicate responses, rejected responses and bad clock.
- Switch accounts with pending awards; verify isolation and case handling.
- Simulate disk full, corrupt ledger, crash during replacement and shutdown draining.
- Exercise normal play, fast-forward, rewind, Polaroid load, resume and cart switching.
  No restored or discarded frame should itself create an award.
- Validate mapped RAM with both real mGBA and gpSP builds on supported devices.
- Measure frame pacing, fast-forward throughput, CPU, memory, battery use and dropped
  snapshots with tracking enabled and disabled. Define performance limits from measurements.
- Host/join/exit link play; verify no radio contention and restored home-network access.
- Build host and ARM64 packages, run workspace checks and inspect bundled licenses.

Research performed against slot.sp `fc35c69e49b5a894d86bcad0a045f41b118345e3`.
No runtime changes or executable tests are included in this proposal.
