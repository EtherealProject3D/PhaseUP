# PhaseUP UI Designer (Windows)

This is the new standalone layout editing application. It runs separately from the Macroquad game and does not change the game's default build.

From the repository root:

```powershell
cargo run --release --features ui-designer --bin phaseup-designer
```

The designer opens a resizable desktop window with a hierarchy at left, board canvas in the center, and properties/snapping at right.

- Ctrl+click elements to multi-select, or drag an empty area to box-select.
- Drag a selection to move it together.
- Mouse wheel zooms toward the pointer; middle mouse drag pans; F fits the board.
- Snapping can be toggled for grid, board edges/centers and other elements.
- Ctrl+Z / Ctrl+Y undo and redo; Ctrl+S saves.
- The editor loads and saves `assets/layouts/game_16_10.json` from the repository working directory.
- The game's existing UI Workshop overlay remains available for now.

**Current scope:** The canvas shows editable layout rectangles, not a live rendered Macroquad game. It does not yet provide artwork rendering, card-test controls, resize handles, smart equal-spacing guides, or multiple aspect-ratio presets. Those are future integration tasks. Existing game elements only move where the game renderer uses layout overrides.

This binary is gated behind the optional `ui-designer` feature so normal game and Android builds do not compile the desktop editor dependencies.
