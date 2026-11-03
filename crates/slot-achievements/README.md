# slot-achievements

Optional GBA casual/softcore tracking for slot.sp. See [the test guide](../../RETROACHIEVEMENTS.md).
The service receives credentials in memory through `Service::configured`; it never
reads production account configuration files or writes an auth token to its cache.
Game metadata and the account-separated unlock ledger contain no login credentials.

Port provenance is in PORT.md. The bundled rcheevos evaluator has its MIT license
under vendor/rcheevos and in the release's licenses folder. The port preserves the
fork's bounded RAM queue, separate evaluator and HTTP workers, persistent ledger and
fake-server tests. Rewind display frames are gated by slot.sp's emulator loop.
