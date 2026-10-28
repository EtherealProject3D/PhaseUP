//! Live menu geometry, typography, and pointer targets use the same document.
use crate::*;
use menu_model::{Bounds, Document};
use std::cell::RefCell;
thread_local! {static DOC:RefCell<Document>=RefCell::new(Document::new(1920.,1200.));static PAGE:RefCell<String>=RefCell::new("home".into());}
pub(super) fn reload() {
    let mode = read_display_layout();
    let (w, h) = read_selected_resolution().unwrap_or(if mode == DisplayLayout::Mobile {
        (2400, 1080)
    } else {
        (1600, 1000)
    });
    let file = menu_model::path(mode == DisplayLayout::Mobile, mode.aspect(w, h));
    #[cfg(not(target_os = "android"))]
    let data = std::fs::read_to_string(file).ok();
    #[cfg(target_os = "android")]
    let data = Some(
        match file.as_str() {
            "assets/layouts/menus_mobile_20_9.json" => {
                include_str!("../assets/layouts/menus_mobile_20_9.json")
            }
            "assets/layouts/menus_mobile_19_5_9.json" => {
                include_str!("../assets/layouts/menus_mobile_19_5_9.json")
            }
            "assets/layouts/menus_desktop_16_9.json" => {
                include_str!("../assets/layouts/menus_desktop_16_9.json")
            }
            _ => include_str!("../assets/layouts/menus_desktop_16_10.json"),
        }
        .to_owned(),
    );
    let doc = data
        .and_then(|s| Document::parse(&s).ok())
        .unwrap_or_else(|| Document::new(1920., 1200.));
    DOC.with(|d| *d.borrow_mut() = doc);
}
pub(super) fn begin(page: &str) {
    PAGE.with(|p| *p.borrow_mut() = page.into());
}
pub(super) fn canvas() -> (f32, f32) {
    DOC.with(|d| d.borrow().canvas())
}
pub(super) fn bounds(id: &str, default: Bounds) -> Bounds {
    PAGE.with(|p| {
        DOC.with(|d| {
            d.borrow()
                .element(&p.borrow(), id)
                .map(|e| e.bounds)
                .unwrap_or(default)
        })
    })
}
pub(super) fn custom_text(id: &str) -> Option<String> {
    PAGE.with(|p| DOC.with(|d| d.borrow().custom_text(&p.borrow(), id).map(str::to_owned)))
}
pub(super) fn font_size(id: &str, default: f32) -> f32 {
    PAGE.with(|p| {
        DOC.with(|d| {
            d.borrow()
                .element(&p.borrow(), id)
                .map(|e| e.size)
                .unwrap_or(default)
        })
    })
}
pub(super) fn has_override(id: &str) -> bool {
    PAGE.with(|p| {
        DOC.with(|d| {
            let d = d.borrow();
            let s = &d.root["screens"][p.borrow().as_str()];
            s["elements"].get(id).is_some() || s["labels"].get(id).is_some()
        })
    })
}
pub(super) fn label(
    id: &str,
    value: &str,
    default: Bounds,
    size: f32,
    color: Color,
    heading: bool,
) {
    let r = bounds(id, default);
    let (text, size, dx, dy) = PAGE.with(|p| {
        DOC.with(|d| {
            let d = d.borrow();
            let page = p.borrow();
            let e = d.element(&page, id);
            let saved = d.root["screens"][page.as_str()]["labels"][id]["size"]
                .as_f64()
                .is_some();
            (
                d.text(&page, id, value),
                if saved {
                    e.as_ref().map(|e| e.size).unwrap_or(size)
                } else {
                    size
                },
                e.as_ref().map_or(0., |e| e.dx),
                e.as_ref().map_or(0., |e| e.dy),
            )
        })
    });
    let metrics = menu_text_dimensions(&text, size as u16);
    let x = r.x + (r.w - metrics.width) / 2. + dx;
    let y = r.y + (r.h - metrics.height) / 2. + metrics.offset_y + dy;
    if heading {
        menu_heading(&text, x, y, size);
    } else {
        menu_label(&text, x, y, size, color);
    }
}
pub(super) fn current_page() -> String {
    PAGE.with(|p| p.borrow().clone())
}
pub(super) fn page_bounds(page: &str, id: &str, default: Bounds) -> Bounds {
    DOC.with(|d| {
        d.borrow()
            .element(page, id)
            .map(|e| e.bounds)
            .unwrap_or(default)
    })
}
pub(super) fn offsets(id: &str) -> (f32, f32) {
    PAGE.with(|p| {
        DOC.with(|d| {
            d.borrow()
                .element(&p.borrow(), id)
                .map(|e| (e.dx, e.dy))
                .unwrap_or((0., 0.))
        })
    })
}
pub(super) fn button(id: &str, text: &str, y: f32) -> bool {
    button_at(
        id,
        text,
        Bounds {
            x: 730.,
            y,
            w: 460.,
            h: 64.,
        },
        28.,
    )
}
pub(super) fn button_at(id: &str, text: &str, default: Bounds, default_size: f32) -> bool {
    let r = bounds(id, default);
    let (mx, my) = game_mouse();
    let hover = r.contains(mx, my);
    themed_button(r.x, r.y, r.w, r.h, hover);
    let base = font_size(id, default_size);
    let size = if menu_text_width(text, base as u16) > r.w - 40. {
        (base - 5.).max(8.)
    } else {
        base
    };
    label(
        id,
        text,
        r,
        size,
        if hover && active_theme() != Theme::Futuristic {
            BG
        } else {
            WHITEISH
        },
        false,
    );
    !board_input_blocked() && hover && is_mouse_button_pressed(MouseButton::Left)
}
#[cfg(feature = "developer-tools")]
pub(super) fn load_for_test(doc: Document) {
    DOC.with(|d| *d.borrow_mut() = doc);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn page_state_selects_the_correct_button_geometry() {
        let mut d = Document::new(2600., 1200.);
        d.set_bounds(
            "home",
            "new_game",
            Bounds {
                x: 100.,
                y: 200.,
                w: 600.,
                h: 90.,
            },
        );
        DOC.with(|v| *v.borrow_mut() = d);
        begin("home");
        assert_eq!(
            bounds(
                "new_game",
                Bounds {
                    x: 0.,
                    y: 0.,
                    w: 1.,
                    h: 1.
                }
            )
            .x,
            100.
        );
        begin("home_resume");
        assert_eq!(
            bounds(
                "new_game",
                Bounds {
                    x: 0.,
                    y: 0.,
                    w: 1.,
                    h: 1.
                }
            )
            .y,
            437.
        );
        DOC.with(|d| *d.borrow_mut() = Document::new(1920., 1200.));
    }
}
