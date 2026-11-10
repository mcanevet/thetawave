# thetawave — ported real-world Leafwing fixture

## NOTICE — upstream attribution

- **Upstream project**: Theta Wave — <https://github.com/thetawavegame/thetawave>
  (formerly `mdrummies/thetawave`)
- **Commit ported from**: `94fad04fb913bb50245ef33d96abc95ca8cd2355`
  (upstream `main`, 2024-12-20)
- **Upstream license**: MIT — kept verbatim in [`LICENSE.upstream`](LICENSE.upstream)
  (Copyright (c) 2022 Theta Wave)
- **Statement of modifications**: ported to Bevy 0.20 /
  leafwing-input-manager 0.22 (crates.io release; upstream pinned a
  git revision); companion crates bumped to their Bevy-0.20-compatible
  releases; assets reduced to the headless-minimum set — real sound
  files (~13MB) replaced by one tiny silent stand-in WAV with every
  dynamic-asset key repointed at it, ~55 premium art files that
  upstream serves from a private S3 bucket (absent even from a
  pristine upstream checkout) replaced by 1×1 placeholder PNGs,
  `fonts/Lunchds.ttf` — "Lunchtime Doubly So" by codeman38
  (<https://zone38.net/>), a 100%-free dafont.com font (license asks
  for credit; credit hereby given) absent from the
  upstream repo but required by its asset lists — added from an
  equivalent public copy; playback-only/steam/wasm scaffolding and
  the optional `cli` feature (argh) removed; local sqlite
  persistence (thetawave_storage) KEPT as an off-by-default
  `storage` feature, same as upstream — excluded from the playtest
  battery. See "Port notes" below for the full
  list.

## What this fixture is

A REAL third-party game — nobody involved in bevy_playtest wrote it —
vendored into the fixture battery to prove bevy_playtest's
declared-tier input discovery machinery (Leafwing reader,
intent-tap, reconciliation)
against a game with RON-loaded `InputMap`s, multiple `Actionlike`
types, real gameplay actions, and no knowledge of bevy_playtest.

It is deliberately NOT a known-answer oracle: there is no ground-truth
file and no accuracy claim. The battery proves the game boots headless
under takeover and reacts to a real declared action; the demo-level
discovery assertions prove the reader enumerates the game's actual
`InputMap`s. Honest claims only.

## Port notes (Bevy 0.14.2 → 0.20)

Companion-crate bumps: `bevy_rapier2d` 0.37, `bevy_kira_audio` 0.27,
`bevy_asset_loader` 0.28, `leafwing-input-manager` git-rev → 0.22.
(Detailed migration notes are recorded in the port commit itself.)

Upstream in-crate tests are restored where portable. The `cli`
(argh-based flag parsing) feature and its test were removed with
this port; the `storage` crate survives as an optional
off-by-default `storage` feature (upstream parity), so its test
exists here but is excluded from the playtest battery, which never
enables the feature. The argh dependency went with the removed `cli`
feature.

## Running

Standard battery tier (same as every fixture):

```sh
cargo test --manifest-path fixtures/Cargo.toml \
  -p fixture_thetawave --features playtest --test playtest \
  -- --test-threads=1
```

`--test-threads=1` is required: `run_scenario` is exclusive per process.

Debugging a scenario directly (live logs, quick re-runs):

```sh
BEVY_PLAYTEST_SCENARIO=<scenario> cargo run --features playtest
```
