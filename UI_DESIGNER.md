# PhaseUP UI Designer (Windows)

Official 0.4.0 game builds use Neon Forge and hide Developer Tools and diagnostic
shortcuts. To run the game with the UI Workshop, diagnostics, and theme switching:

```powershell
cargo run --release --features developer-tools --bin phaseup
```

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

In the game's UI Workshop, open **FULL BOARD PREVIEW** for the unified layout
controls. They are available immediately; there is no separate edit-mode toggle
or per-target alignment toolbar. Existing saved offsets still load normally.

**Current scope:** The canvas shows editable layout rectangles, not a live rendered Macroquad game. It does not yet provide artwork rendering, card-test controls, resize handles, smart equal-spacing guides, or multiple aspect-ratio presets. Those are future integration tasks. Existing game elements only move where the game renderer uses layout overrides.

This binary is gated behind the optional `ui-designer` feature so normal game and Android builds do not compile the desktop editor dependencies.

## Board layout nudging

In UI Workshop > Full Board Preview, the layout tools are always available. Click a
visible element or use Previous/Next (Tab/Shift+Tab) to select it. Panels, header
art, visible text, buttons and their labels, card groups and individual card slots
are listed from the actual rendered board. Drag to move; drag the bottom-right
handle to resize. The staging artwork and drop region now honor saved height.

Use LEFT/RIGHT/UP/DOWN or the keyboard arrow keys to nudge the selection. STEP
cycles 1, 5 and 10 logical pixels; Shift multiplies the keyboard step by ten. MODE
switches between moving and resizing (left/right change width, up/down height).
F8 hides the tool panel to reach covered elements; keyboard nudging still works.
Changes save automatically; Ctrl+S saves explicitly, and Ctrl+Z/UNDO reverts.
RESET SELECTED restores the renderer default for the current element. Invisible
or state-dependent elements appear when their preview state draws them. Button
labels are separate selections. Card-slot overrides describe positions, not card
identities, so a new deal does not assign a layout to a particular card value.

The same toolbar also switches the human preview between one and two phase
requirements and toggles artwork/wireframe (F7). F8 hides or shows the toolbar
without turning off selection or keyboard nudging. Back to Tools/Escape exits
layout manipulation, so normal gameplay remains unaffected.

Drag the **WORKSHOP TOOLS - DRAG HERE** title bar to move the whole tool window,
including Save and Undo. It stays inside the logical viewport, keeps its current
position while switching preview states, and remains the same screen size while
the board is zoomed or panned. F8 still hides/shows it.

Ctrl+click adds an element to the selection, or removes it when already selected.
Selected elements all show yellow outlines. Drag any selected member to move the
whole selection, or use the same nudge controls. A normal click on a new element
replaces the selection; a normal click on an existing member preserves the group
for dragging. Click empty space to clear. Previous/Next and Tab select one item.
Group moves/resizes/resets are one Undo action and save together. In Resize mode,
the same width/height delta is applied to each selected item; relative positions
are retained.

Moving a staging panel or its `stage_N_cards` area also carries individually
positioned staging cards, preserving their existing adjustments. Multi-selecting
both a parent and its children moves each card once, and Undo restores the group.
Resizing the box does not scale individually adjusted cards. Staged rows center
horizontally within their card area as its width or card count changes, retaining
individual adjustments relative to the centered row.

Single- and double-requirement human layouts are independent. Staging elements
use `stage_single_1...` or `stage_double_1...` / `stage_double_2...` names, with
separate labels, card rows and slot positions. Human laid-phase frame positions
are also separate. The single CLEAR button and its text do not share overrides
with CLEAR ALL / CLEAR 1 / CLEAR 2. Legacy staging overrides import only into the
double layout and never overwrite explicitly saved mode-specific positions.

Normal staging labels keep `staging_single_label_1` and
`staging_double_label_1` / `_2`. READY labels have independent
`staging_single_ready_label_1` and `staging_double_ready_label_1` / `_2`
positions. Adjust each label while its state is visible; changing another
slot's readiness does not renumber the selected label. New labels default to
horizontal centering in their panel.

In Full Board Preview, use the **CARDS** tab on the draggable tools window to
choose the player, phase, target (hand, staging slot, or laid group), color,
number, Wild, or Skip. **ADD CARD** adds the chosen card; **REMOVE CARD** removes
one matching that exact value/color. **CLEAR TARGET** clears that preview area.
**FILL VALID STAGED PHASE** instantly fills the human's current requirements so
READY labels appear; remove a matching card to return to the incomplete state.
Switch to **LAYOUT** to adjust the visible labels. Ctrl+S / Save Layout saves
positions; the preview cards themselves do not change or save the live game.
