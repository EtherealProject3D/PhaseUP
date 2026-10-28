//! Runtime adapter for standalone designer documents. Never writes source JSON.
use super::Rect;
use serde_json::Value;
use std::cell::RefCell;

thread_local! { static DOCUMENT: RefCell<Option<Value>> = const { RefCell::new(None) }; }
thread_local! { static PHASE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

pub(crate) fn set_phase(phase: usize) {
    PHASE.with(|v| v.set(phase.min(crate::PHASES.len() - 1)));
}

pub(crate) fn active() -> bool {
    DOCUMENT.with(|d| d.borrow().is_some())
}
pub(crate) fn mobile() -> bool {
    DOCUMENT.with(|d| {
        d.borrow()
            .as_ref()
            .is_some_and(|j| j["profile"].as_str() == Some("mobile"))
    })
}
pub(crate) fn canvas() -> Option<(f32, f32)> {
    DOCUMENT.with(|d| {
        let d = d.borrow();
        let j = d.as_ref()?;
        Some((
            j["canvas"]["width"].as_f64()? as f32,
            j["canvas"]["height"].as_f64()? as f32,
        ))
    })
}
pub(crate) fn show_cpu() -> bool {
    DOCUMENT.with(|d| {
        d.borrow()
            .as_ref()
            .and_then(|j| j["show_cpu_cards"].as_bool())
            .unwrap_or(true)
    })
}
pub(crate) fn separate() -> bool {
    element("staging").is_some()
}
pub(crate) fn card_number_size(default: f32) -> f32 {
    DOCUMENT.with(|d| {
        d.borrow()
            .as_ref()
            .and_then(|j| j["card_number_size"].as_f64())
            .filter(|size| size.is_finite() && (8.0..=96.0).contains(size))
            .map(|size| size as f32)
            .unwrap_or(default)
    })
}

// Parent panels are authored elements too, not just containers for child boxes.
fn panel_rects() -> Vec<Rect> {
    [
        "header", "cpu_1", "cpu_2", "cpu_3", "phase", "staging", "hand",
    ]
    .into_iter()
    .filter_map(element)
    .collect()
}
pub(crate) fn draw_panel_backgrounds() {
    for r in panel_rects() {
        crate::draw_rectangle(r.x, r.y, r.w, r.h, crate::active_theme().panel());
    }
}
pub(crate) fn draw_panel_borders() {
    for r in panel_rects() {
        crate::draw_rectangle_lines(
            r.x,
            r.y,
            r.w,
            r.h,
            1.5,
            crate::Color::new(0.43, 0.43, 0.43, 1.),
        );
    }
}
pub(crate) fn phase_status(laid: bool, ready: bool) -> &'static str {
    if laid {
        "PHASE IN PLAY"
    } else if ready {
        "READY — NOT IN PLAY"
    } else {
        "BUILD YOUR PHASE — NOT IN PLAY"
    }
}
pub(crate) fn draw_phase_status(laid: bool, ready: bool) {
    let key = if separate() && !laid {
        "staging"
    } else {
        "phase"
    };
    label(key, phase_status(laid, ready), 22., crate::WHITEISH);
}
fn read(v: &Value) -> Option<Rect> {
    let r = Rect {
        x: v["x"].as_f64()? as f32,
        y: v["y"].as_f64()? as f32,
        w: v["width"].as_f64()? as f32,
        h: v["height"].as_f64()? as f32,
    };
    ([r.x, r.y, r.w, r.h].iter().all(|v| v.is_finite()) && r.w > 0. && r.h > 0.).then_some(r)
}
pub(crate) fn load(json: Option<&Value>) {
    let doc = json
        .filter(|j| j["elements"]["hand"].is_object() && j["elements"]["phase"].is_object())
        .filter(|j| {
            j["canvas"]["width"]
                .as_f64()
                .is_some_and(|v| v.is_finite() && v > 0.)
                && j["canvas"]["height"]
                    .as_f64()
                    .is_some_and(|v| v.is_finite() && v > 0.)
        })
        .cloned();
    DOCUMENT.with(|d| *d.borrow_mut() = doc);
}
pub(crate) fn element(key: &str) -> Option<Rect> {
    DOCUMENT.with(|d| {
        let d = d.borrow();
        let j = d.as_ref()?;
        let state = if j["show_cpu_cards"].as_bool().unwrap_or(true) {
            "cpu_expanded"
        } else {
            "cpu_collapsed"
        };
        if key.starts_with("cpu_") {
            if let Some(r) = read(&j[state][key]) {
                return Some(r);
            }
        }
        read(&j["elements"][key]).or_else(|| {
            if key != "draw_count" {
                return None;
            }
            let draw = read(&j["elements"]["draw"])?;
            let size = j["labels"]["draw"]["size"].as_f64().unwrap_or(30.) as f32;
            let dx = j["labels"]["draw"]["dx"].as_f64().unwrap_or(0.) as f32;
            let dy = j["labels"]["draw"]["dy"].as_f64().unwrap_or(0.) as f32;
            Some(Rect {
                x: draw.x + dx,
                y: draw.y + draw.h / 2. + size * 0.65 + dy,
                w: draw.w,
                h: 24.,
            })
        })
    })
}
pub(crate) fn mapped(key: &str) -> Option<Rect> {
    if let Some(base) = key.strip_suffix("_1") {
        if ["sort_runs", "sort_colors", "sort_sets"].contains(&base) {
            return element(base);
        }
    }
    let alias = match key {
        "upper_banner" => "header",
        "hand_panel" => "hand",
        "draw_pile" => "draw",
        "discard_pile" => "discard",
        "player_phase_single" | "player_phase_double" => "phase",
        "turn_instruction" | "lay_phase_button" | "lay_phase_button_1" => "phase_action",
        "sort_button_1" => "sort_runs",
        "sort_button_2" => "sort_colors",
        "sort_button_3" => "sort_sets",
        "discard_button_1" => "discard_action",
        _ => key,
    };
    if let Some(r) = element(alias) {
        return Some(r);
    }
    for cpu in 1..=3 {
        if key == format!("cpu_{cpu}_artwork") {
            return element(&format!("cpu_{cpu}"));
        }
        for group in 1..=2 {
            if key == format!("cpu_{cpu}_group_{group}") {
                return element(&format!("cpu_{cpu}_laid_{group}"));
            }
        }
    }
    for count in 1..=2 {
        for group in 1..=count {
            let mode = if count == 1 { "single" } else { "double" };
            if key == format!("human_{mode}_group_{group}") {
                return element(&format!("phase_{count}_group_{group}"));
            }
            let prefix = format!("stage_{mode}_{group}");
            let source = if separate() { "staging" } else { "phase" };
            let source = format!("{source}_{count}_group_{group}");
            if key == prefix {
                return element(&source);
            }
            if key == format!("{prefix}_cards") {
                return element(&source).map(content);
            }
        }
    }
    None
}
pub(crate) fn content(r: Rect) -> Rect {
    Rect {
        x: r.x + 6.,
        y: r.y + 30.,
        w: (r.w - 12.).max(1.),
        h: (r.h - 36.).max(1.),
    }
}
pub(crate) fn row(key: &str, area: Rect, count: usize) -> (Rect, f32) {
    DOCUMENT.with(|d| {
        let d = d.borrow();
        let style = d.as_ref().map(|j| &j["hand_card_styles"][key]);
        let number = |field: &str, default: f32| {
            style
                .and_then(|s| s[field].as_f64())
                .filter(|n| n.is_finite())
                .map(|n| n as f32)
                .unwrap_or(default)
        };
        let h = number("height", area.h.min(136.)).max(1.).min(area.h);
        let w = (h * 0.68).min(area.w);
        let gap = number("gap", 12.).clamp(0., 200.);
        let step = if count > 1 {
            (w + gap).min((area.w - w).max(0.) / (count - 1) as f32)
        } else {
            0.
        };
        let total = if count == 0 {
            0.
        } else {
            w + step * count.saturating_sub(1) as f32
        };
        (
            Rect {
                x: area.x + (area.w - total) / 2. + number("dx", 0.),
                y: area.y + (area.h - h) / 2. + number("dy", 0.),
                w,
                h,
            },
            step,
        )
    })
}
pub(crate) fn hand(count: usize) -> Option<(Rect, f32)> {
    let r = element("hand")?;
    Some(row(
        "hand",
        Rect {
            x: r.x + 20.,
            y: r.y + 42.,
            w: (r.w - 32.).max(1.),
            h: (r.h - 54.).max(1.),
        },
        count,
    ))
}
pub(crate) fn hand_card(count: usize, index: usize) -> Option<Rect> {
    let (mut r, step) = hand(count)?;
    r.x += index as f32 * step;
    DOCUMENT.with(|d| {
        if let Some(j) = d.borrow().as_ref() {
            let style = &j["hand_card_styles"][format!("hand_card_{index}")];
            r.x += style["dx"].as_f64().unwrap_or(0.) as f32;
            r.y += style["dy"].as_f64().unwrap_or(0.) as f32;
        }
    });
    Some(r)
}
#[derive(Debug, PartialEq)]
struct LabelSpec {
    text: String,
    size: f32,
    dx: f32,
    dy: f32,
}

fn label_spec(key: &str, text: &str, default: f32) -> LabelSpec {
    DOCUMENT.with(|d| {
        let d = d.borrow();
        let style = d.as_ref().map(|j| &j["labels"][key]);
        let number = |name: &str, fallback: f32| {
            style
                .and_then(|s| s[name].as_f64())
                .filter(|v| v.is_finite())
                .map(|v| v as f32)
                .unwrap_or(fallback)
        };
        let custom = style.and_then(|s| s["text"].as_str()).unwrap_or("");
        let text = if custom.is_empty() {
            text.to_owned()
        } else {
            PHASE.with(|phase| {
                custom
                    .replace("{value}", text)
                    .replace("{phase_number}", &(phase.get() + 1).to_string())
                    .replace("{phase_name}", crate::phase_name(phase.get()))
            })
        };
        LabelSpec {
            text,
            size: number("size", default).max(8.),
            dx: number("dx", 0.),
            dy: number("dy", 0.),
        }
    })
}

pub(crate) fn label(key: &str, text: &str, default: f32, color: crate::Color) -> bool {
    let Some(r) = element(key) else {
        return false;
    };
    let spec = label_spec(key, text, default);
    let size = spec.size;
    let content = &spec.text;
    let band = if key.contains("group_") || key.contains("_laid_") {
        24.
    } else if ["hand", "phase", "staging"].contains(&key) || key.starts_with("cpu_") {
        32.
    } else {
        r.h
    };
    let available = (r.w - 16.).max(1.);
    let mut lines = vec![String::new()];
    for word in content.split_whitespace() {
        let last = lines.last_mut().unwrap();
        let next = if last.is_empty() {
            word.to_owned()
        } else {
            format!("{last} {word}")
        };
        let wrap = key == "phase_action";
        if wrap && !last.is_empty() && crate::menu_text_width(&next, size as u16) > available {
            lines.push(word.to_owned());
        } else {
            *last = next;
        }
    }
    let top = r.y + (band - size * 1.1 * lines.len() as f32) / 2.;
    for (index, line) in lines.iter().enumerate() {
        let width = crate::menu_text_width(line, size as u16);
        crate::menu_label(
            line,
            r.x + (r.w - width) / 2. + spec.dx,
            top + (index as f32 + 0.85) * size * 1.1 + spec.dy,
            size,
            color,
        );
    }
    true
}

pub(crate) fn laid_card(
    player: usize,
    group: usize,
    groups: usize,
    index: usize,
    count: usize,
) -> Option<Rect> {
    let key = if player == 0 {
        format!("phase_{groups}_group_{}", group + 1)
    } else {
        format!("cpu_{player}_laid_{}", group + 1)
    };
    let (r, step) = row(&key, content(element(&key)?), count);
    Some(Rect {
        x: r.x + index as f32 * step,
        ..r
    })
}
pub(crate) fn cpu_hand(player: usize, index: usize, count: usize) -> Option<Rect> {
    if !show_cpu() {
        let p = element(&format!("cpu_{player}"))?;
        return Some(Rect {
            x: p.x + p.w / 2. - 30.,
            y: p.y,
            w: 60.,
            h: 88.,
        });
    }
    let key = format!("cpu_{player}_hand");
    let (r, step) = row(&key, element(&key)?, count);
    Some(Rect {
        x: r.x + index as f32 * step,
        ..r
    })
}

// Resolve renderer IDs at the label boundary, keeping the document's label
// offsets relative to their owning panels instead of desktop absolute positions.
pub(crate) fn runtime_label(key: &str, text: &str, size: f32, color: crate::Color) -> bool {
    if !active() {
        return false;
    }
    if key.starts_with("draw_count_text") || key.starts_with("draw_remaining_count") {
        return label("draw_count", text, size, color);
    }
    if key.starts_with("version_text")
        || key.starts_with("round_turn_text")
        || key.contains("_not_laid")
        || key.starts_with("human_") && key.contains("title")
    {
        return true;
    }
    let alias = if key.starts_with("turn_")
        || key.starts_with("phase_ready_text")
        || key.starts_with("lay_phase_button")
    {
        Some("phase_action")
    } else if key.starts_with("draw_card_text") {
        Some("draw")
    } else if key.starts_with("draw_title") {
        return true;
    } else if key.starts_with("discard_title") {
        Some("discard")
    } else if key.starts_with("logo_text") {
        return true;
    } else if key.starts_with("sort_runs") {
        Some("sort_runs")
    } else if key.starts_with("sort_colors") {
        Some("sort_colors")
    } else if key.starts_with("sort_sets") {
        Some("sort_sets")
    } else if key.starts_with("discard_button") {
        Some("discard_action")
    } else if key.starts_with("menu_text") {
        Some("menu_button")
    } else {
        None
    };
    if let Some(alias) = alias {
        return label(alias, text, size, color);
    }
    for count in 1..=2 {
        let mode = if count == 1 { "single" } else { "double" };
        for group in 1..=count {
            if key == format!("staging_{mode}_label_{group}")
                || key == format!("staging_{mode}_ready_label_{group}")
            {
                return label(
                    &format!(
                        "{}_{}_group_{group}",
                        if separate() { "staging" } else { "phase" },
                        count
                    ),
                    text,
                    size,
                    color,
                );
            }
            for cpu in 0..=3 {
                for state in ["laid", "unlaid"] {
                    if key == format!("player_{cpu}_{mode}_{state}_requirement_{group}") {
                        return label(
                            &if cpu == 0 {
                                format!("phase_{count}_group_{group}")
                            } else {
                                format!("cpu_{cpu}_laid_{group}")
                            },
                            text,
                            size,
                            color,
                        );
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn draw_count_has_its_own_saved_geometry_and_text_style() {
        let mut j: Value = serde_json::from_str(include_str!(
            "../assets/layouts/prototype_mobile_compact_20_9.json"
        ))
        .unwrap();
        load(Some(&j));
        assert!(element("draw_count").is_some());
        j["elements"]["draw_count"] =
            serde_json::json!({"x":1100.,"y":600.,"width":200.,"height":40.});
        j["labels"]["draw_count"] =
            serde_json::json!({"text":"Cards: {value}","size":24.,"dx":7.,"dy":-3.});
        load(Some(&j));
        let r = element("draw_count").unwrap();
        assert_eq!((r.x, r.y, r.w, r.h), (1100., 600., 200., 40.));
        assert_eq!(
            label_spec("draw_count", "63 left", 17.),
            LabelSpec {
                text: "Cards: 63 left".into(),
                size: 24.,
                dx: 7.,
                dy: -3.
            }
        );
        load(None);
    }
    #[test]
    fn card_number_size_uses_saved_value_and_keeps_legacy_defaults() {
        let mut j: Value = serde_json::from_str(include_str!(
            "../assets/layouts/prototype_mobile_compact_20_9.json"
        ))
        .unwrap();
        // Exercise an older document without changing the user's saved size.
        j.as_object_mut().unwrap().remove("card_number_size");
        load(Some(&j));
        assert_eq!(card_number_size(28.), 28.);
        assert_eq!(card_number_size(32.), 32.);
        j["card_number_size"] = 48.into();
        load(Some(&j));
        assert_eq!(card_number_size(28.), 48.);
        assert_eq!(card_number_size(32.), 48.);
        for invalid in [
            Value::Null,
            serde_json::json!(200),
            serde_json::json!(-1),
            serde_json::json!("48"),
        ] {
            j["card_number_size"] = invalid;
            load(Some(&j));
            assert_eq!(card_number_size(28.), 28.);
        }
        load(None);
    }
    #[test]
    fn designer_parent_panels_and_phase_status_are_preserved() {
        let j: Value = serde_json::from_str(include_str!(
            "../assets/layouts/prototype_mobile_compact_20_9.json"
        ))
        .unwrap();
        load(Some(&j));
        for key in ["cpu_1", "cpu_2", "cpu_3", "phase", "hand"] {
            let r = element(key).unwrap();
            assert!(
                panel_rects()
                    .iter()
                    .any(|p| (p.x, p.y, p.w, p.h) == (r.x, r.y, r.w, r.h)),
                "{key}"
            );
        }
        assert_eq!(phase_status(false, false), "BUILD YOUR PHASE — NOT IN PLAY");
        assert_eq!(phase_status(false, true), "READY — NOT IN PLAY");
        assert_eq!(phase_status(true, false), "PHASE IN PLAY");
        assert_eq!(phase_status(true, true), "PHASE IN PLAY");
        assert_eq!(
            label_spec("phase", phase_status(false, false), 22.).size,
            30.
        );
        load(None);
        assert!(panel_rects().is_empty());
    }
    #[test]
    fn mobile_selection_toggles_independently_with_an_immediate_fixed_raise() {
        use crate::*;
        let j: Value = serde_json::from_str(include_str!(
            "../assets/layouts/prototype_mobile_compact_20_9.json"
        ))
        .unwrap();
        load(Some(&j));
        let mut g = Game::new();
        g.stage = Stage::Play;
        let ids: Vec<_> = g.players[0].hand.iter().take(3).map(|c| c.id).collect();
        for id in &ids {
            g.toggle_hand_selection(*id);
        }
        assert_eq!(g.selected, ids);
        assert_eq!(selected_card_lift(true), 15.);
        assert_eq!(selected_card_lift(false), 0.);
        let before = g.discard.len();
        g.discard_selected();
        assert_eq!(g.discard.len(), before);
        g.toggle_hand_selection(ids[1]);
        assert_eq!(g.selected, vec![ids[0], ids[2]]);
        g.stage_selected(0);
        assert_eq!(g.staged[0], vec![ids[0], ids[2]]);
        assert!(g.selected.is_empty());
        load(None);
    }
    #[test]
    fn desktop_selection_still_replaces_the_previous_card() {
        use crate::*;
        load(None);
        let mut g = Game::new();
        g.toggle_hand_selection(1);
        g.toggle_hand_selection(2);
        assert_eq!(g.selected, vec![2]);
        g.toggle_hand_selection(2);
        assert!(g.selected.is_empty());
    }
    #[test]
    fn saved_text_settings_reach_group_hand_draw_and_action_labels() {
        let mut j: Value = serde_json::from_str(include_str!(
            "../assets/layouts/prototype_mobile_compact_20_9.json"
        ))
        .unwrap();
        for key in [
            "phase_2_group_1",
            "cpu_1_laid_1",
            "hand",
            "draw",
            "phase_action",
        ] {
            j["labels"][key] = serde_json::json!({"text":"{value} / {phase_number}","size":48.,"dx":17.,"dy":-11.});
        }
        load(Some(&j));
        set_phase(1);
        for key in [
            "phase_2_group_1",
            "cpu_1_laid_1",
            "hand",
            "draw",
            "phase_action",
        ] {
            assert_eq!(
                label_spec(key, "SET OF 3", 18.),
                LabelSpec {
                    text: "SET OF 3 / 2".into(),
                    size: 48.,
                    dx: 17.,
                    dy: -11.
                }
            );
        }
        j["labels"]["draw"]["size"] = 8.into();
        load(Some(&j));
        assert_eq!(label_spec("draw", "DRAW", 19.).size, 8.);
        load(None);
    }
    #[test]
    fn saved_mobile_documents_map_without_mutation() {
        for source in [
            include_str!("../assets/layouts/prototype_mobile_compact_20_9.json"),
            include_str!("../assets/layouts/prototype_mobile_compact_19_5_9.json"),
        ] {
            let j: Value = serde_json::from_str(source).unwrap();
            let before = j.clone();
            load(Some(&j));
            assert!(active());
            assert_eq!(
                canvas().unwrap().0,
                j["canvas"]["width"].as_f64().unwrap() as f32
            );
            for cpu in 1..=3 {
                let r = mapped(&format!("cpu_{cpu}_group_1")).unwrap();
                let expected = read(&j["elements"][format!("cpu_{cpu}_laid_1")]).unwrap();
                assert_eq!((r.x, r.y), (expected.x, expected.y));
            }
            for count in [1, 11, 24] {
                let (r, step) = hand(count).unwrap();
                let p = element("hand").unwrap();
                assert!(r.y >= p.y && r.y + r.h <= p.y + p.h);
                assert!(r.x + step * (count - 1) as f32 + r.w <= p.x + p.w);
            }
            assert!(mapped("stage_double_1_cards").is_some());
            assert_eq!(j, before);
        }
        load(None);
    }
    #[test]
    fn cpu_state_and_missing_snapshot_fallback() {
        let mut j: Value = serde_json::from_str(include_str!(
            "../assets/layouts/prototype_mobile_compact_20_9.json"
        ))
        .unwrap();
        for show in [true, false] {
            j["show_cpu_cards"] = show.into();
            load(Some(&j));
            assert_eq!(show_cpu(), show);
            let expected = read(
                &j[if show {
                    "cpu_expanded"
                } else {
                    "cpu_collapsed"
                }]["cpu_1"],
            )
            .unwrap();
            let r = element("cpu_1").unwrap();
            assert_eq!(
                (r.x, r.y, r.w, r.h),
                (expected.x, expected.y, expected.w, expected.h)
            );
        }
        j.as_object_mut().unwrap().remove("cpu_collapsed");
        load(Some(&j));
        assert!(element("cpu_1").is_some());
        load(None);
        assert!(!active());
    }
    #[test]
    fn nonstandard_canvas_and_individual_hand_offsets_are_not_rescaled_twice() {
        let mut j: Value = serde_json::from_str(include_str!(
            "../assets/layouts/prototype_mobile_compact_20_9.json"
        ))
        .unwrap();
        j["canvas"] = serde_json::json!({"width":2400.,"height":1080.});
        load(Some(&j));
        let original = hand_card(11, 5).unwrap();
        j["hand_card_styles"]["hand_card_5"] = serde_json::json!({"dx":13.,"dy":-7.});
        load(Some(&j));
        let adjusted = hand_card(11, 5).unwrap();
        assert_eq!(canvas(), Some((2400., 1080.)));
        assert_eq!(
            (adjusted.x, adjusted.y),
            (original.x + 13., original.y - 7.)
        );
        for key in [
            "sort_runs_1",
            "sort_colors_1",
            "sort_sets_1",
            "discard_button_1",
            "turn_instruction",
        ] {
            assert!(mapped(key).is_some(), "{key}");
        }
        load(None);
    }
    #[test]
    fn actual_game_stages_lays_and_discards_with_designer_geometry() {
        use crate::*;
        let j: Value = serde_json::from_str(include_str!(
            "../assets/layouts/prototype_mobile_compact_20_9.json"
        ))
        .unwrap();
        load(Some(&j));
        for phase in [0, 3] {
            let mut g = Game::new();
            workshop::choose_phase(&mut g, 0, phase);
            g.stage = Stage::Play;
            g.turn = 0;
            workshop::fill_staged_phase(&mut g, 5, 0);
            assert!(g.staged_phase_ready());
            for group in 0..PHASES[phase].len() {
                let r = layout::staging_card(group, 0, PHASES[phase].len(), g.staged[group].len());
                let p = element(&format!(
                    "phase_{}_group_{}",
                    PHASES[phase].len(),
                    group + 1
                ))
                .unwrap();
                assert!(r.x >= p.x && r.y >= p.y && r.y + r.h <= p.y + p.h);
            }
            g.selected = g.staged.iter().flatten().copied().collect();
            g.lay();
            assert!(g.players[0].laid);
            assert!(laid_card(
                0,
                0,
                g.players[0].table.len(),
                0,
                g.players[0].table[0].len()
            )
            .is_some());
            g.selected = vec![g.players[0].hand[0].id];
            let count = g.discard.len();
            g.discard_selected();
            assert_eq!(g.discard.len(), count + 1);
        }
        load(None);
    }
}
