# PhaseUP UI Designer (Windows)

Official 0.4.x game builds use Neon Forge and hide Developer Tools and diagnostic
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
- The editor saves independent `assets/layouts/prototype_compact_16_10.json` and `assets/layouts/prototype_staging_16_10.json` files. It never writes the polished `game_16_10.json`. Switching layouts saves the outgoing profile. Use Save/Ctrl+S before closing.
- The game's existing UI Workshop overlay remains available for now.

In the game's UI Workshop, open **FULL BOARD PREVIEW** for the unified layout
controls. They are available immediately; there is no separate edit-mode toggle
or per-target alignment toolbar. Existing saved offsets still load normally.

**Plain prototype:** Starts in **Play preview** with a deterministic hand after drawing.
Drag cards into the combined phase area, or back into your hand. Click cards to
raise/select multiple cards. A valid phase enables **PLAY PHASE**; the status
changes from **NOT IN PLAY** to **PHASE IN PLAY**. Played cards cannot return to
the hand; valid hits are allowed. Select one hand card and discard to complete
the test turn. Reset test turn starts again. The sidebar can switch all ten
phases or fill a valid phase instantly.

Drag within the hand to manually position a card (white dot). Sorting preserves
those cards' slots; pressing the same sort again sorts everything. Selecting or
moving cards resets that second-click action. Runs and Sets currently share the
game's numerical sort order.

Uncheck **Play preview** to edit the same rectangles that the preview uses.
Select elements on the canvas or hierarchy; drag, nudge with arrows (Shift = 10),
or edit X/Y/Width/Height in Properties. Enable **Separate staging** to compare
the alternative layout. Each profile has its own undo history while active.

**Scope:** This is an eframe interaction sandbox, using the game's shared card
model and phase validation. It is not yet the Macroquad game renderer or a full
CPU match. The new layouts/settings are not active in the released game.
Opponent panels are placeholders. Full game integration, individual text layout
controls, art and gameplay settings come after layout review. In this preview,
a ready phase blocks discarding until played; the live game's confirmation
behavior is unchanged.

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

Standalone builder: in layout-editing mode, Show selected centerlines (under Snapping) draws gold horizontal/vertical guides through each selected object's center across the board. Blue lines mark the board center. Properties shows the selected object's exact center coordinates. Guides are editor-only and are not saved into the artwork or layout.

## Mobile and desktop prototypes

The builder starts on **Mobile**, importing the earlier prototype layout when no
mobile file exists. Mobile/Desktop and compact/separate staging each save to an
independent `prototype_<device>_<mode>_16_10.json` file. Existing source profiles
and Neon Forge are left intact. This names the target device; it does not yet
change the 1920x1200 logical canvas or implement responsive phone sizing.

In edit mode, select `phase_2_group_1` / `_2` (or `phase_1_group_1` for a single
requirement) directly on the canvas or in Elements. Separate staging has its own
group boxes. Moves and resizes of parent panels carry their children; individual
group adjustments remain editable. Default two-group margins are equal.

Every selected panel/button has Text controls: content, font size and X/Y offsets.
Blank content retains the live label; `{value}` inserts the dynamic label into a
custom caption. These text settings save with the profile; layout Undo currently
covers rectangles, not text edits. Use Reset text to restore the live label.

CPU panels show a hand row and two laid-phase boxes. **Show CPU hand cards** hides
the rows while retaining the count in the CPU heading. Resize/reposition panels
to use the freed space. **Preview CPU laid phases** fills their boxes with sample
sets; these are visual fixtures, not simulated CPU decisions. Hidden rows are not
selectable but their layout is preserved for when they are shown again.

CPU hand visibility now automatically collapses/expands the CPU panels. Hiding removes the band between the heading and laid-phase boxes (102 logical pixels in the default layout), moving those boxes upward without shrinking their cards. Expanded and collapsed CPU arrangements are remembered separately in each profile and persisted with Save. Existing saved hidden-card profiles migrate once. Other board sections retain their positions so you can allocate the reclaimed space yourself. Toggling clears rectangle undo history to keep the two arrangements separate.

Mobile landscape presets: 20:9 (default), 19.5:9, 16:9 and 16:10 are selectable in the toolbar. Desktop remains 16:10. Each aspect saves to its own prototype_<device>_<mode>_<aspect>.json. A new aspect imports a copy of the saved 16:10 profile, proportionally remapping panel positions/widths; it does not overwrite the source. Canvas height remains 1200 logical units and width follows the selected ratio; these are design units, not device pixels. Cards retain their proportions. Fit, centerlines, snapping and property bounds use the chosen canvas. Actual Android integration and device safe areas are not yet part of this builder preview.

Mobile policy: mobile always uses the combined phase area and hidden/collapsed CPU hands. Separate staging and CPU-card visibility controls appear only on Desktop. Existing mobile profiles with visible CPU cards collapse on load; older separate-staging files remain intact but are no longer selectable for Mobile.

Window placement: the standalone editor uses eframe window persistence. Its position, size and maximized state are stored in the current user's application settings and restored on the next launch. Move/resize the updated editor, then close it normally to save its placement. The 1500x900 size is only the first-run fallback. Window preferences are separate from board layout Save.

Aspect choices are now restricted by device: Mobile offers 20:9 and 19.5:9; Desktop offers 16:9 and 16:10. Desktop aspect profiles save independently too. Existing 16:10 files remain available as migration sources.

## Hand card sizing and guide grid

In layout-editing mode (Play preview unchecked), click any card in your hand or a
visible CPU hand. Properties shows shared Card width/height controls; changing
either preserves proportions and changes all cards in that hand. The selected
card also has a yellow corner handle: drag vertically to resize the whole hand.
Choose Center, Left edge, Right edge, Top edge or Bottom edge as the fixed point
for each card. Other hands retain their own sizing. CPU hand cards are available
in Desktop with Show CPU hand cards enabled. Mobile continues to hide CPU hands.
Card-size edits support Undo/Redo while a card is selected. Reset this hand's
card size restores automatic fitting. Save persists hand sizes per layout.

Guide grid has independent Vertical guides and Horizontal guides switches, plus
Columns across and Rows down counts (1-32). Four columns and two rows produce
three interior vertical lines and one interior horizontal line. Guides are
visual only; the existing pixel-grid snapping controls remain separate. Guide
preferences save with each layout.

Resize anchor now applies to every layout object. Select a panel, group, button or card and choose Center, Left edge, Right edge, Top edge or Bottom edge in Properties. Width/Height edits hold that anchor fixed. For multiple selected layout objects, Narrower/Wider and Shorter/Taller apply the same size change to each around its own chosen anchor. Parent changes continue to carry their children; Undo restores the size and position together.
