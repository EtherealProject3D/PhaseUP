# Saved designer layouts in gameplay

Gameplay reads the document canvas as its logical coordinate space. Camera and
pointer conversion use those same dimensions, so a saved coordinate is applied
once, regardless of the fitted window size or letterboxing.

The runtime adapter translates designer panel names, single/double phase groups,
staging groups, piles, sorting and action buttons. Compact designs share the
phase area between staging and laid cards. A separate staging panel is supported
when present. CPU hand visibility selects the corresponding expanded/collapsed
snapshot; missing entries fall back to `elements`. Preview CPU card counts do
not replace real gameplay counts.

Rows use the saved card height, spacing and offsets. Human hand card offsets,
labels and label text overrides are read from the document. Animation endpoints
use the same row geometry as the cards and pointer targets. Legacy desktop
documents retain their existing renderer defaults. Android embeds both mobile
aspect documents as well as the legacy fallback.

No layout JSON is rewritten. The gameplay Workshop save/reset operations reject
designer documents, directing edits back to the standalone designer.

The designer's Properties panel includes **Card numbers → Number text size**
(8–96 logical pixels). This applies to card values across the selected layout,
including corner values on laid cards. Save with Ctrl+S and reload the game with
F5. **Reset number size** restores the existing hand/laid defaults. The setting
is stored as the optional `card_number_size` field; older files keep their current
appearance. Saved card/panel geometry is unaffected.

The `draw_count (cards remaining)` element is available in the editor's element
list. It has independent position, bounds, text size and text offsets; blank text
uses the live game's remaining deck count. The editor shows a sample count that
can be adjusted in its preview controls. Moving the draw panel also moves this
child label. Older documents receive an in-memory default until saved.

Mobile resolution choices are 2400×1080 (20:9) and 2340×1080 (19.5:9), one preset
per supported mobile aspect. Logical layout coordinates still come from the
saved canvas and scale to the actual fitted window/device viewport.

## Validation

Run automated tests and a release build:

```powershell
cargo test --features developer-tools
cargo build --release --features developer-tools
```

Capture the real gameplay renderer without changing window preferences or saved
layouts:

```powershell
cargo run --release --features developer-tools -- --layout-smoke output/mobile-integration
```

This exits after producing twenty-four PNGs: both mobile aspects, expanded/collapsed
CPU hands, single/double requirements, and building/ready/laid human phases. It does not
start the regular game or save application settings. Captures are local review
artifacts and are not committed.

Regression tests cover the actual saved documents, snapshot fallback, hand
containment at different counts, button mapping, non-1200 canvas heights,
individual card offsets, and staging/laying/discarding through the real game.

Known pre-existing designer test failure:
`resizing_one_card_resizes_its_hand_and_preserves_the_chosen_anchor` fails on the
unchanged baseline as well as this branch. It is unrelated to number rendering.
The new number-size persistence/default tests and the remaining designer checks
pass. To run those checks, use `cargo test --features ui-designer --bin
phaseup-designer -- --skip resizing_one_card_resizes_its_hand_and_preserves_the_chosen_anchor`.

The saved 19.5:9 document contains overlapping geometry: `phase_action` intersects
the hand, and the sorting controls at y=1140 with height=100 exceed its 1200-high
canvas. Gameplay preserves these authored positions. The 20:9 file uses a
separate right action column and left sorting column. Correcting the 19.5:9
composition requires a later designer edit; this change deliberately preserves
both files byte for byte. The 20:9 expanded snapshot also places CPU hands and
laid groups at y=0; selecting that snapshot reproduces its overlap. Normal mobile
play uses the saved collapsed state. Windows rendering/builds are checked here; Android
device interaction is not exercised by these captures.

## Normal pull and launch

```powershell
Set-Location C:\PhaseUP
git switch fix/mobile-size-sync-trigger
git pull --ff-only origin fix/mobile-size-sync-trigger
cargo run --release --features developer-tools
```

Select Mobile in Settings and choose a 20:9 or 19.5:9 preset. Normal launch does
not run tests.
