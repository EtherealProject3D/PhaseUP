//! Menu definitions and persisted overrides shared by both renderers.
// Each binary uses a different half of this editing/rendering API.
#![allow(dead_code)]
use serde_json::{json, Value};
pub(crate) const PAGES: &[(&str, &str)] = &[
    ("home", "Home — new game"),
    ("home_resume", "Home — game in progress"),
    ("settings", "Settings"),
    ("settings_resume", "Settings — game in progress"),
    ("resolution_choices", "Resolution choices"),
    ("confirm_new", "New Game confirmation"),
    ("how_to_play", "How to Play"),
    ("rules", "Game Rules"),
    ("developer_tools", "Developer Tools"),
    ("ui_workshop", "UI Workshop"),
];
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Bounds {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl Bounds {
    pub(crate) fn json(self) -> Value {
        json!({"x":self.x,"y":self.y,"width":self.w,"height":self.h})
    }
    pub(crate) fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }
    fn read(v: &Value) -> Option<Self> {
        let r = Self {
            x: v["x"].as_f64()? as f32,
            y: v["y"].as_f64()? as f32,
            w: v["width"].as_f64()? as f32,
            h: v["height"].as_f64()? as f32,
        };
        ([r.x, r.y, r.w, r.h].iter().all(|v| v.is_finite()) && r.w > 0. && r.h > 0.).then_some(r)
    }
}
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ElementKind {
    Panel,
    Button,
    Text,
    Content,
}
#[derive(Clone)]
pub(crate) struct Element {
    pub id: String,
    pub kind: ElementKind,
    pub bounds: Bounds,
    pub text: String,
    pub size: f32,
    pub dx: f32,
    pub dy: f32,
}
pub(crate) fn pointer_to_canvas(
    canvas: (f32, f32),
    viewport: (f32, f32),
    pointer: (f32, f32),
) -> (f32, f32) {
    let scale = (viewport.0 / canvas.0).min(viewport.1 / canvas.1);
    (
        canvas.0 / 2. + (pointer.0 - viewport.0 / 2.) / scale,
        canvas.1 / 2. + (pointer.1 - viewport.1 / 2.) / scale,
    )
}
pub(crate) fn path(mobile: bool, aspect: &str) -> String {
    format!(
        "assets/layouts/menus_{}_{aspect}.json",
        if mobile { "mobile" } else { "desktop" }
    )
}
pub(crate) fn defaults(page: &str, w: f32, h: f32) -> Vec<Element> {
    let mut result = vec![];
    let dx = (w - 1920.) / 2.;
    let dy = (h - 1200.) / 2.;
    let mut add =
        |id: &str, kind: ElementKind, x: f32, y: f32, w: f32, h: f32, text: &str, size: f32| {
            result.push(Element {
                id: id.into(),
                kind,
                bounds: Bounds {
                    x: x + dx,
                    y: y + dy,
                    w,
                    h,
                },
                text: text.into(),
                size,
                dx: 0.,
                dy: 0.,
            })
        };
    use ElementKind::*;
    if page == "resolution_choices" {
        let mobile = w / h > 2.;
        let presets: &[(i32, i32)] = if mobile {
            &[(2400, 1080), (2340, 1080)]
        } else {
            &[
                (1280, 800),
                (1440, 900),
                (1600, 1000),
                (1920, 1200),
                (2560, 1600),
                (1280, 720),
                (1600, 900),
                (1920, 1080),
                (2560, 1440),
            ]
        };
        add(
            "panel",
            Panel,
            834.,
            346.,
            252.,
            47. + presets.len() as f32 * 54.,
            "",
            22.,
        );
        add(
            "title",
            Text,
            846.,
            350.,
            228.,
            28.,
            if mobile {
                "MOBILE RESOLUTIONS"
            } else {
                "DESKTOP RESOLUTIONS"
            },
            17.,
        );
        for (i, (rw, rh)) in presets.iter().enumerate() {
            add(
                &format!("resolution_{i}"),
                Button,
                846.,
                385. + i as f32 * 54.,
                228.,
                48.,
                &format!("{rw} x {rh}"),
                17.,
            );
        }
        return result;
    }
    if ["how_to_play", "rules"].contains(&page) {
        add(
            "title",
            Text,
            365.,
            108.,
            1190.,
            62.,
            if page == "rules" {
                "GAME RULES"
            } else {
                "HOW TO PLAY"
            },
            45.,
        );
        add(
            "content",
            Content,
            365.,
            170.,
            1190.,
            860.,
            &help_sections(page == "rules")
                .iter()
                .map(|(_, text)| text.as_str())
                .collect::<Vec<_>>()
                .join("\n\n"),
            23.,
        );
        add(
            "scroll_hint",
            Text,
            415.,
            975.,
            1090.,
            32.,
            "Mouse wheel / Up-Down arrows to scroll",
            17.,
        );
        add("back", Button, 730., 1028., 460., 60., "BACK TO MENU", 22.);
        return result;
    }
    add("panel", Panel, 620., 160., 680., 880., "", 22.);
    let title = match page {
        "home" | "home_resume" => "PHASEUP",
        "settings" | "settings_resume" => "SETTINGS",
        "confirm_new" => "NEW GAME?",
        "developer_tools" => "DEVELOPER TOOLS",
        _ => "UI WORKSHOP",
    };
    let title_top = if page.starts_with("home") {
        221.
    } else if page.starts_with("settings") {
        176.
    } else {
        226.
    };
    add("title", Text, 660., title_top, 600., 70., title, 54.);
    if page.starts_with("home") {
        let resume = page == "home_resume";
        let first = if resume { 355. } else { 385. };
        let step = if resume { 82. } else { 85. };
        let mut row = 0.;
        if resume {
            add("resume", Button, 730., first, 460., 64., "RESUME GAME", 28.);
            row += 1.;
        }
        for (id, text) in [
            ("new_game", "NEW GAME"),
            ("how_to_play", "HOW TO PLAY"),
            ("rules", "GAME RULES"),
            ("settings", "SETTINGS"),
            ("quit", "QUIT"),
            ("difficulty", "CPU DIFFICULTY: NORMAL"),
        ] {
            add(id, Button, 730., first + step * row, 460., 64., text, 28.);
            row += 1.;
        }
        if resume {
            add(
                "difficulty_note",
                Text,
                690.,
                first + step * row - 6.,
                540.,
                30.,
                "Difficulty applies to new games",
                18.,
            );
        }
    } else if page.starts_with("settings") {
        for (id, text, y) in [
            ("window_heading", "WINDOW", 260.),
            ("interface_heading", "INTERFACE", 484.),
            ("appearance_heading", "APPEARANCE", 646.),
        ] {
            add(id, Text, 750., y, 420., 24., text, 22.);
        }
        for (id, text, y) in [
            ("preset", "PRESET: 2400 x 1080 ▼", 292.),
            ("layout", "LAYOUT: MOBILE", 354.),
            ("fullscreen", "TOGGLE FULLSCREEN", 416.),
            ("recent_moves", "RECENT MOVES: ON", 516.),
            ("speed", "GAME SPEED: NORMAL", 578.),
            ("theme", "THEME: MODERN & CLEAN", 678.),
            ("developer_tools", "DEVELOPER TOOLS", 740.),
            ("back", "BACK", 802.),
        ] {
            add(id, Button, 730., y, 460., 54., text, 28.);
        }
        if page == "settings_resume" {
            add("resume", Button, 730., 864., 460., 54., "RESUME GAME", 28.);
        }
    } else if page == "confirm_new" {
        add(
            "warning",
            Text,
            660.,
            320.,
            600.,
            52.,
            "Start over? Current game progress will be lost.",
            22.,
        );
        for (id, text, y) in [
            ("confirm", "YES, NEW GAME", 405.),
            ("cancel", "CANCEL", 505.),
            ("resume", "RESUME GAME", 605.),
        ] {
            add(id, Button, 730., y, 460., 64., text, 28.);
        }
    } else if page == "developer_tools" {
        add(
            "note",
            Text,
            670.,
            310.,
            580.,
            44.,
            "Visual previews do not affect gameplay.",
            22.,
        );
        add(
            "workshop",
            Button,
            730.,
            445.,
            460.,
            64.,
            "UI WORKSHOP",
            28.,
        );
        add(
            "back",
            Button,
            730.,
            545.,
            460.,
            64.,
            "BACK TO SETTINGS",
            28.,
        );
    } else {
        add(
            "back",
            Button,
            730.,
            920.,
            460.,
            64.,
            "BACK TO DEVELOPER TOOLS",
            28.,
        );
    }
    result
}
#[derive(Clone)]
pub(crate) struct Document {
    pub root: Value,
}
impl Document {
    pub(crate) fn new(w: f32, h: f32) -> Self {
        Self {
            root: json!({"schema_version":1,"canvas":{"width":w,"height":h},"screens":{}}),
        }
    }
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        let root: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let doc = Self { root };
        let (w, h) = doc.canvas();
        if !doc.root.is_object()
            || w < 320.
            || h < 240.
            || w > 10000.
            || h > 10000.
            || !w.is_finite()
            || !h.is_finite()
        {
            return Err("Invalid menu canvas".into());
        }
        if !doc.root["screens"].is_null() {
            let screens = doc.root["screens"]
                .as_object()
                .ok_or("Invalid menu screens")?;
            for screen in screens.values() {
                if !screen.is_object() {
                    return Err("Invalid menu screen".into());
                }
                for field in ["elements", "labels"] {
                    if !screen[field].is_null() && !screen[field].is_object() {
                        return Err(format!("Invalid menu {field}"));
                    }
                }
            }
        }
        Ok(doc)
    }
    pub(crate) fn canvas(&self) -> (f32, f32) {
        (
            self.root["canvas"]["width"].as_f64().unwrap_or(1920.) as f32,
            self.root["canvas"]["height"].as_f64().unwrap_or(1200.) as f32,
        )
    }
    pub(crate) fn custom_text(&self, page: &str, id: &str) -> Option<&str> {
        self.root["screens"][page]["labels"][id]["text"]
            .as_str()
            .filter(|s| !s.is_empty())
    }
    pub(crate) fn elements(&self, page: &str) -> Vec<Element> {
        let (w, h) = self.canvas();
        defaults(page, w, h)
            .into_iter()
            .map(|mut e| {
                if let Some(r) = Bounds::read(&self.root["screens"][page]["elements"][&e.id]) {
                    e.bounds = r;
                }
                let s = &self.root["screens"][page]["labels"][&e.id];
                let num = |key: &str, default: f32| {
                    s[key]
                        .as_f64()
                        .filter(|v| v.is_finite())
                        .map(|v| v as f32)
                        .unwrap_or(default)
                };
                e.size = num("size", e.size).clamp(8., 120.);
                e.dx = num("dx", 0.);
                e.dy = num("dy", 0.);
                if let Some(text) = self.custom_text(page, &e.id) {
                    e.text = text.replace("{value}", &e.text);
                }
                e
            })
            .collect()
    }
    pub(crate) fn element(&self, page: &str, id: &str) -> Option<Element> {
        self.elements(page).into_iter().find(|e| e.id == id)
    }
    pub(crate) fn text(&self, page: &str, id: &str, value: &str) -> String {
        self.custom_text(page, id)
            .map(|s| s.replace("{value}", value))
            .unwrap_or_else(|| value.into())
    }
    pub(crate) fn set_bounds(&mut self, page: &str, id: &str, r: Bounds) {
        self.root["screens"][page]["elements"][id] = r.json();
    }
    pub(crate) fn set_style(
        &mut self,
        page: &str,
        id: &str,
        text: &str,
        size: f32,
        dx: f32,
        dy: f32,
    ) {
        self.root["screens"][page]["labels"][id] = json!({"text":text,"size":size,"dx":dx,"dy":dy});
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn moved_menu_buttons_keep_their_pointer_targets_under_letterboxing() {
        for canvas in [(2666.6667, 1200.), (2600., 1200.), (1920., 1200.)] {
            for viewport in [(1600., 900.), (1280., 720.), (900., 700.)] {
                let mut d = Document::new(canvas.0, canvas.1);
                d.set_bounds(
                    "home",
                    "new_game",
                    Bounds {
                        x: canvas.0 / 2. - 240.,
                        y: 410.,
                        w: 480.,
                        h: 72.,
                    },
                );
                let r = d.element("home", "new_game").unwrap().bounds;
                let scale = (viewport.0 / canvas.0).min(viewport.1 / canvas.1);
                let point = (
                    viewport.0 / 2. + (r.x + r.w / 2. - canvas.0 / 2.) * scale,
                    viewport.1 / 2. + (r.y + r.h / 2. - canvas.1 / 2.) * scale,
                );
                let actual = pointer_to_canvas(canvas, viewport, point);
                assert!(r.contains(actual.0, actual.1));
                assert!(!r.contains(actual.0, r.y + r.h + 1.));
            }
        }
    }
    #[test]
    fn malformed_screen_maps_are_rejected_without_panicking() {
        for root in [
            json!({"screens":[]}),
            json!({"screens":{"home":4}}),
            json!({"screens":{"home":{"labels":[]}}}),
        ] {
            assert!(Document::parse(&root.to_string()).is_err());
        }
    }
    #[test]
    fn override_round_trip_preserves_other_screens_and_unknown_fields() {
        let mut d = Document::new(2666.6667, 1200.);
        d.root["future_setting"] = json!({"keep":true});
        let old = d.elements("settings")[0].bounds;
        d.set_bounds(
            "home",
            "new_game",
            Bounds {
                x: 100.,
                y: 200.,
                w: 400.,
                h: 80.,
            },
        );
        d.set_style("home", "new_game", "Start: {value}", 31., 5., -2.);
        let s = serde_json::to_string(&d.root).unwrap();
        let d = Document::parse(&s).unwrap();
        assert_eq!(d.element("home", "new_game").unwrap().bounds.x, 100.);
        assert_eq!(d.text("home", "new_game", "NEW GAME"), "Start: NEW GAME");
        assert_eq!(d.elements("settings")[0].bounds, old);
        assert_eq!(d.root["future_setting"]["keep"], true);
    }
    #[test]
    fn all_pages_have_unique_ids_and_valid_bounds_for_mobile_and_desktop() {
        for w in [1920., 2133.3333, 2600., 2666.6667] {
            for (page, _) in PAGES {
                let d = Document::new(w, 1200.);
                let e = d.elements(page);
                let ids: std::collections::HashSet<_> = e.iter().map(|e| &e.id).collect();
                assert_eq!(ids.len(), e.len());
                assert!(e.iter().all(|e| e.bounds.w > 0.
                    && e.bounds.h > 0.
                    && e.bounds.x >= 0.
                    && e.bounds.x + e.bounds.w <= w + 0.01));
            }
        }
    }
    #[test]
    fn invalid_overrides_fall_back_and_profile_files_are_separate() {
        let mut d = Document::new(1920., 1200.);
        d.root["screens"]["home"]["elements"]["new_game"] =
            json!({"x":1.,"y":2.,"width":-1.,"height":64.});
        assert_eq!(d.element("home", "new_game").unwrap().bounds.w, 460.);
        assert_ne!(path(true, "20_9"), path(true, "19_5_9"));
        assert_ne!(path(false, "16_9"), path(true, "20_9"));
    }
}

pub(crate) fn help_sections(rules: bool) -> Vec<(bool, String)> {
    use crate::phases::{phase_name, PHASES};
    let mut rows: Vec<(bool, String)> = Vec::new();
    if rules {
        rows.push((true, "THE GOAL".into()));
        rows.push((false,"Complete all ten phases in order. Each round, players work on their own current phase. A player who completes a phase advances to the next one at the end of that round.".into()));
        rows.push((true, "THE TEN PHASES".into()));
        for i in 0..PHASES.len() {
            rows.push((false, format!("{}: {}", i + 1, phase_name(i))));
        }
        rows.push((true, "CARD COMBINATIONS".into()));
        rows.push((false,"Set: cards with the same number, regardless of color. Run: consecutive numbers, regardless of color. Color group: cards of the same color, regardless of number.".into()));
        rows.push((true, "WILD CARDS".into()));
        rows.push((false,"Wilds can stand in for a number or color in a phase or hit. Every group must contain at least one ordinary numbered card. Wilds cannot replace Skip cards.".into()));
        rows.push((true, "SKIP CARDS".into()));
        rows.push((false,"A Skip card is discarded to make another player miss a turn. It cannot be used in a phase or hit, and cannot be drawn from the discard pile.".into()));
        rows.push((true, "HITTING".into()));
        rows.push((false,"After laying your own phase, you can add cards to any completed phase if the resulting set, run, or color group is valid.".into()));
        rows.push((true, "ENDING A ROUND".into()));
        rows.push((false,"A round ends when someone gets rid of all their cards. Players who laid their phase advance one phase; others retry the same phase next round.".into()));
        rows.push((true, "SCORING".into()));
        rows.push((false,"Cards remaining in your hand count against you: numbers 1-9 = 5 points; 10-12 = 10 points; Skip = 15 points; Wild = 25 points.".into()));
        rows.push((true, "WINNING".into()));
        rows.push((
            false,
            "The game ends when a player completes phase 10 and the round finishes.".into(),
        ));
    } else {
        rows.push((true, "1. DRAW A CARD".into()));
        rows.push((false,"Start your turn by clicking the draw deck or the top discard card. You cannot take a Skip from the discard pile.".into()));
        rows.push((true, "2. BUILD YOUR PHASE".into()));
        rows.push((false,"Check the phase shown above your hand. Drag cards into the staging groups. A set uses matching numbers, a run uses consecutive numbers, and a color group uses one color.".into()));
        rows.push((true, "3. LAY YOUR PHASE".into()));
        rows.push((false,"When your staged cards satisfy the phase, click LAY PHASE. Your cards move to your table area. You can only lay your phase once per round.".into()));
        rows.push((true, "4. HIT OTHER PHASES".into()));
        rows.push((false,"Once your phase is laid, drag a card onto your phase or another player's completed phase to extend a valid group. For example, add a 7 to a run ending in 6.".into()));
        rows.push((true, "5. DISCARD TO END YOUR TURN".into()));
        rows.push((false,"Click a card in your hand to select it, then discard it. On mobile, tap cards to toggle multiple selections; select exactly one to discard. You can also use a Skip to make an opponent miss a turn.".into()));
        rows.push((true, "6. FINISH THE ROUND".into()));
        rows.push((false,"Be the first to get rid of all your cards, or keep working on your phase until another player goes out. Completed phases advance next round.".into()));
        rows.push((true, "TIP: SORT YOUR HAND".into()));
        rows.push((false,"Use RUNS, COLORS, or SETS below your hand to change how cards are arranged. Sorting does not change which cards you own.".into()));
        rows.push((true, "WINDOW SHORTCUTS".into()));
        rows.push((false,"Right-click the PHASEUP logo during a game, or press Ctrl+0, to reset the game window to its default size. You can also reset the window size from Settings.".into()));
        rows.push((true, "NEED THE DETAILS?".into()));
        rows.push((false,"Open GAME RULES from the main menu for the ten phase requirements, special cards, scoring, and win conditions.".into()));
    }

    rows
}
