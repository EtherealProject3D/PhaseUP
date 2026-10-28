# Menu editing

Open the UI Designer and choose **Design: Menus** at the top. Gameplay editing
continues to use **Design: Gameplay** and its existing files.

Choose the same Mobile/Desktop profile and aspect ratio as the game, then choose
a screen: Home, Home with a game in progress, Settings, Settings with a game in
progress, Resolution choices, New Game confirmation, How to Play, or Game Rules.
Developer Tools and the UI Workshop menu shell/back control are also available;
the workshop's gameplay testing widgets remain in the gameplay workspace.

Select an item in **Menu elements** or click it on the canvas. Drag it to move it,
or use **Properties** to change X, Y, width and height. Moving a main panel carries
its controls with it. Resizing the panel changes its frame; controls can be resized
independently. Text controls support custom captions, font size, and X/Y offsets.
Blank text keeps the live game caption; `{value}` inserts its current caption.
For example, a speed button can keep showing the active speed as it changes.
Editing a caption does not change the button's action.

Help pages expose their title, scrolling content area, body size/text, hint and
Back button. Blank help text keeps the shared instructions. Entering custom text
replaces the help copy. Increase the content area's dimensions as needed; the
game rewraps and scrolls the text inside that area.

**Undo/Redo**, **Reset this element**, middle-button pan, Ctrl+wheel zoom, and
**Fit board (F)** are available. **Preview navigation** lets you click between
menu screens without starting a game, quitting, or changing actual settings.

Press **Ctrl+S**, then switch to the desktop game and press **F5**. No rebuild is
needed for later menu edits. Restart/rebuild both apps once to install this
feature. Android packages embed menu files and must be rebuilt to include edits.

Menus save independently in:

* `assets/layouts/menus_mobile_20_9.json`
* `assets/layouts/menus_mobile_19_5_9.json`
* `assets/layouts/menus_desktop_16_9.json`
* `assets/layouts/menus_desktop_16_10.json`

Screens store only overrides. Missing elements use shared defaults; other screens
and unknown metadata survive saving. Home/Settings layouts with and without an
active game have separate overrides, so adding Resume does not shift an edited
layout unexpectedly. Gameplay JSON is not changed by menu editing.

The game uses the same saved bounds for artwork and input, with its menu canvas
used for both camera scaling and pointer conversion. Menu input ignores gameplay
workshop pan/zoom. The resolution dropdown has its own editable screen, and its
outside-click region follows its saved panel. Settings defaults are spaced to
avoid overlapping section labels and buttons.

Validation includes menu save/parse round trips, invalid data fallback, profile
isolation, stable screen/control IDs, parent movement, undo/redo, safe preview
navigation, and pointer mapping under letterboxing. The real game renderer can
produce 22 captures, including custom button geometry/text, without changing
preferences:

```powershell
cargo run --release --features developer-tools -- --menu-smoke output/menu-integration
```

To capture the standalone menu workspace for visual review:

```powershell
cargo run --release --features ui-designer --bin phaseup-designer -- --menu-capture output/menu-integration/editor.png --menu-page settings --menu-element speed
```

Create the output directory first when using the standalone editor capture. Capture
mode uses separate window persistence, saves only the PNG, and exits automatically.
