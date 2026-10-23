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

In the game's UI Workshop, open **FULL BOARD PREVIEW** to use the alignment toolbar.
The **TARGET** button cycles through CPU, HUMAN, DRAW, DISCARD, and DRAW COUNT. For DRAW and
DISCARD, **SWITCH TEXT / PILE** selects the text or pile; the arrows move that
selection one logical pixel at a time. DRAW text moves its title; DRAW COUNT independently moves the remaining
card count. Text offsets are relative to the pile, so moving a pile carries its
text with it. Click **SAVE OFFSETS** to persist all adjustments across restarts;
**RESET SELECTED** resets only the selected target and component. Pile click areas
and card animations follow the adjusted pile positions.

**Current scope:** The canvas shows editable layout rectangles, not a live rendered Macroquad game. It does not yet provide artwork rendering, card-test controls, resize handles, smart equal-spacing guides, or multiple aspect-ratio presets. Those are future integration tasks. Existing game elements only move where the game renderer uses layout overrides.

This binary is gated behind the optional `ui-designer` feature so normal game and Android builds do not compile the desktop editor dependencies.
