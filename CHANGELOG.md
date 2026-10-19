# PhaseUP — Version History

PhaseUP is in early development. This history preserves milestones and links to the exact source commits. **These are historical source snapshots, not yet published GitHub Releases or prebuilt Windows/Android binaries.** Some early builds have known bugs.

## v0.3.1 — Window memory and debug tools
**Source snapshot:** [bc66cb4](https://github.com/EtherealProject3D/PhaseUP/tree/bc66cb4eb692714e9ed1e577b913fa30e4e0f177)

- Save and restore desktop window dimensions after resizing.
- Toggle an F3 diagnostic overlay with FPS, dimensions, canvas scale, mouse position and game state.
- Window position is not yet persisted.
- This build was committed but has not yet been independently compiled or user-tested.

## v0.3.0 — Responsive UI
**Source snapshot:** [6424d78](https://github.com/EtherealProject3D/PhaseUP/tree/6424d78bd50aa19ded69bd6a29084e5cd60c459a)

- Increased the default window size to 1600 × 1000.
- Added aspect-preserving canvas scaling and matching mouse coordinates.
- Added a prominent turn-status indicator.
- Fixed an accidental upside-down rendering regression in the initial scaling commit.
- User reported successfully playing a hand after the camera fix.

## v0.2.1 — Playable table interactions
**Source snapshot:** [4d66495](https://github.com/EtherealProject3D/PhaseUP/tree/4d66495827e08dc4224fbf8b18212778bd9b8da7)

- Showed laid phases face-up on the table and displayed a version label.
- Added a floating card preview while dragging and staging-area feedback.
- Highlighted valid hit groups and supported hitting directly by dragging from the hand.
- Fixed round completion when the final card was played as a hit.
- User confirmed a hand could be completed.

## Earlier prototypes
See the [repository commit history](https://github.com/EtherealProject3D/PhaseUP/commits/main/) for earlier work on the game rules, CPU players, staging, sorting and the first UI. Earlier version labels have not been independently verified, so they are not assigned retroactive release numbers here.

## Building a historical snapshot
Choose a source snapshot link above, then use GitHub's **Code → Download ZIP**, or check out the listed commit with Git. Build using Rust/Cargo:

```powershell
cargo run --release
```

For planned features, see [ROADMAP.md](ROADMAP.md). For tracked tasks, see [GitHub Issues](https://github.com/EtherealProject3D/PhaseUP/issues).

## Publishing formal GitHub Releases
For each milestone, create a Git tag pointing to the exact snapshot commit (for example `v0.2.1`, `v0.3.0`, `v0.3.1`), then publish a GitHub Release from that tag. Mark these early builds as **pre-releases**. Do not tag current `main` for older versions.
