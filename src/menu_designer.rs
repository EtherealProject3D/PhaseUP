//! Independent menu workspace: it never rewrites gameplay layouts.
use crate::menu_model::{self, Bounds, Document, ElementKind, PAGES};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use serde_json::Value;
pub(super) struct MenuDesigner {
    dirty: bool,
    doc: Document,
    mobile: bool,
    aspect: usize,
    page: usize,
    selected: Option<String>,
    zoom: f32,
    pan: Vec2,
    undo: Vec<Value>,
    redo: Vec<Value>,
    drag: Option<(Pos2, Value, String)>,
    status: String,
    preview: bool,
    scroll: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn moving_a_panel_carries_controls_and_undo_redo_restores_the_document() {
        let mut editor = MenuDesigner::new();
        editor.doc = Document::new(2666.6667, 1200.);
        let original = editor.doc.root.clone();
        let p = editor.doc.element("home", "panel").unwrap().bounds;
        let button = editor.doc.element("home", "new_game").unwrap().bounds;
        editor.checkpoint();
        editor.apply_bounds(
            "home",
            "panel",
            Bounds {
                x: p.x + 120.,
                y: p.y + 30.,
                ..p
            },
        );
        let moved = editor.doc.element("home", "new_game").unwrap().bounds;
        assert_eq!((moved.x, moved.y), (button.x + 120., button.y + 30.));
        let edited = editor.doc.root.clone();
        editor.undo();
        assert_eq!(editor.doc.root, original);
        editor.redo();
        assert_eq!(editor.doc.root, edited);
    }
    #[test]
    fn preview_navigation_does_not_change_the_saved_document() {
        let mut editor = MenuDesigner::new();
        let root = editor.doc.root.clone();
        editor.navigate("settings");
        assert_eq!(PAGES[editor.page].0, "settings");
        editor.navigate("preset");
        assert_eq!(PAGES[editor.page].0, "resolution_choices");
        editor.navigate("resolution_0");
        editor.navigate("quit");
        assert_eq!(editor.doc.root, root);
        assert!(!editor.dirty);
    }
}
impl MenuDesigner {
    pub(super) fn new() -> Self {
        let mut s = Self {
            dirty: false,
            doc: Document::new(2666.6667, 1200.),
            mobile: true,
            aspect: 0,
            page: 0,
            selected: None,
            zoom: 0.,
            pan: Vec2::ZERO,
            undo: vec![],
            redo: vec![],
            drag: None,
            status: String::new(),
            preview: false,
            scroll: 0.,
        };
        s.load();
        if let Some(page) = std::env::args().skip_while(|a| a != "--menu-page").nth(1) {
            s.page = PAGES.iter().position(|(key, _)| *key == page).unwrap_or(0);
        }
        if let Some(id) = std::env::args()
            .skip_while(|a| a != "--menu-element")
            .nth(1)
        {
            if s.doc.element(PAGES[s.page].0, &id).is_some() {
                s.selected = Some(id);
            }
        }
        s
    }
    fn aspect_name(&self) -> &'static str {
        ["20_9", "19_5_9", "16_9", "16_10"][self.aspect]
    }
    fn path(&self) -> String {
        menu_model::path(self.mobile, self.aspect_name())
    }
    fn load(&mut self) {
        let width = 1200. * [20. / 9., 19.5 / 9., 16. / 9., 16. / 10.][self.aspect];
        match std::fs::read_to_string(self.path()) {
            Ok(text) => match Document::parse(&text) {
                Ok(doc) => {
                    self.doc = doc;
                    self.status = format!("Loaded {}", self.path());
                }
                Err(e) => {
                    self.doc = Document::new(width, 1200.);
                    self.status = format!("Load failed: {e}; showing defaults");
                }
            },
            Err(_) => {
                self.doc = Document::new(width, 1200.);
                self.status = "Using menu defaults; Save creates a menu layout file".into();
            }
        }
        self.dirty = false;
        self.selected = None;
        self.undo.clear();
        self.redo.clear();
        self.drag = None;
        self.zoom = 0.;
        self.pan = Vec2::ZERO;
        self.scroll = 0.;
    }
    fn save(&mut self) {
        let result = (|| -> Result<(), String> {
            let path = self.path();
            std::fs::create_dir_all("assets/layouts").map_err(|e| e.to_string())?;
            let text = serde_json::to_string_pretty(&self.doc.root).map_err(|e| e.to_string())?;
            std::fs::write(path, text + "\n").map_err(|e| e.to_string())
        })();
        self.status = match result {
            Ok(()) => {
                self.dirty = false;
                format!("Saved {} — press F5 in the game", self.path())
            }
            Err(e) => format!("Save failed: {e}"),
        };
    }
    fn checkpoint(&mut self) {
        self.dirty = true;
        self.status = "Unsaved menu changes".into();
        self.undo.push(self.doc.root.clone());
        if self.undo.len() > 100 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }
    fn undo(&mut self) {
        if let Some(root) = self.undo.pop() {
            self.redo.push(self.doc.root.clone());
            self.doc.root = root;
            self.dirty = true;
        }
    }
    fn redo(&mut self) {
        if let Some(root) = self.redo.pop() {
            self.undo.push(self.doc.root.clone());
            self.doc.root = root;
            self.dirty = true;
        }
    }
    fn apply_bounds(&mut self, page: &str, id: &str, bounds: Bounds) {
        if id == "panel" {
            let old = self.doc.element(page, id).unwrap().bounds;
            let elements = self.doc.elements(page);
            for e in elements {
                if e.id == id {
                    continue;
                }
                let r = e.bounds;
                self.doc.set_bounds(
                    page,
                    &e.id,
                    Bounds {
                        x: r.x + bounds.x - old.x,
                        y: r.y + bounds.y - old.y,
                        ..r
                    },
                );
            }
        }
        self.doc.set_bounds(page, id, bounds);
    }
    fn navigate(&mut self, id: &str) {
        let current = PAGES[self.page].0;
        let page = match id {
            "preset" => "resolution_choices",
            "how_to_play" => "how_to_play",
            "rules" => "rules",
            "settings" => {
                if current == "home_resume" {
                    "settings_resume"
                } else {
                    "settings"
                }
            }
            "resolution_0" | "resolution_1" | "resolution_2" | "resolution_3" | "resolution_4"
            | "resolution_5" | "resolution_6" | "resolution_7" | "resolution_8" => "settings",
            "new_game" => {
                if current == "home_resume" {
                    "confirm_new"
                } else {
                    "home_resume"
                }
            }
            "confirm" | "resume" => "home_resume",
            "cancel" => "home_resume",
            "developer_tools" => "developer_tools",
            "workshop" => "ui_workshop",
            "back" => match current {
                "developer_tools" => "settings",
                "ui_workshop" => "developer_tools",
                "settings_resume" => "home_resume",
                _ => "home",
            },
            _ => {
                self.status = "Preview only: this control does not change game settings".into();
                return;
            }
        };
        self.page = PAGES.iter().position(|(key, _)| *key == page).unwrap_or(0);
        self.selected = None;
        self.scroll = 0.;
    }
    pub(super) fn show(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("Menu Designer");
                let mut mobile = self.mobile;
                let mut aspect = self.aspect;
                egui::ComboBox::from_id_salt("menu_profile")
                    .selected_text(if mobile { "Mobile" } else { "Desktop" })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut mobile, true, "Mobile");
                        ui.selectable_value(&mut mobile, false, "Desktop");
                    });
                if mobile != self.mobile {
                    aspect = if mobile { 0 } else { 3 };
                }
                egui::ComboBox::from_id_salt("menu_aspect")
                    .selected_text(["20:9", "19.5:9", "16:9", "16:10"][aspect])
                    .show_ui(ui, |ui| {
                        for i in if mobile { 0..2 } else { 2..4 } {
                            ui.selectable_value(
                                &mut aspect,
                                i,
                                ["20:9", "19.5:9", "16:9", "16:10"][i],
                            );
                        }
                    });
                if mobile != self.mobile || aspect != self.aspect {
                    if self.dirty {
                        self.save();
                    }
                    if !self.dirty {
                        self.mobile = mobile;
                        self.aspect = aspect;
                        self.load();
                    }
                }
                let old = self.page;
                egui::ComboBox::from_id_salt("menu_page")
                    .selected_text(PAGES[self.page].1)
                    .show_ui(ui, |ui| {
                        for (i, (_, label)) in PAGES.iter().enumerate() {
                            ui.selectable_value(&mut self.page, i, *label);
                        }
                    });
                if old != self.page {
                    self.selected = None;
                    self.drag = None;
                    self.scroll = 0.;
                }
                if ui.button("Save  Ctrl+S").clicked() {
                    self.save();
                }
                if ui.button("Reload").clicked() {
                    self.load();
                }
                if ui.button("Undo").clicked() {
                    self.undo();
                }
                if ui.button("Redo").clicked() {
                    self.redo();
                }
                if ui.button("Fit board  F").clicked() {
                    self.zoom = 0.;
                    self.pan = Vec2::ZERO;
                }
                ui.checkbox(&mut self.preview, "Preview navigation");
            });
        });
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.save();
        }
        if !ctx.wants_keyboard_input() {
            if ctx.input(|i| i.key_pressed(egui::Key::F)) {
                self.zoom = 0.;
                self.pan = Vec2::ZERO;
            }
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
                self.undo();
            }
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Y)) {
                self.redo();
            }
        }
        let page = PAGES[self.page].0;
        egui::SidePanel::left("menu_elements")
            .default_width(220.)
            .show(ctx, |ui| {
                ui.heading("Menu elements");
                ui.label(PAGES[self.page].1);
                ui.separator();
                for e in self.doc.elements(page) {
                    if ui
                        .selectable_label(self.selected.as_deref() == Some(&e.id), &e.id)
                        .clicked()
                    {
                        self.selected = Some(e.id);
                    }
                }
            });
        egui::SidePanel::right("menu_properties").default_width(300.).show(ctx,|ui|{egui::ScrollArea::vertical().show(ui,|ui|{
            ui.heading("Properties");ui.label("Drag an element to move it. Drag the board with the middle mouse button to pan. Ctrl+wheel zooms.");
            if let Some(id)=self.selected.clone(){if let Some(e)=self.doc.element(page,&id){
                ui.separator();ui.label(&id);let mut r=e.bounds;let mut changed=false;
                egui::Grid::new("menu_bounds").show(ui,|ui|{for (label,value,min) in [("X",&mut r.x,-10000.),("Y",&mut r.y,-10000.),("Width",&mut r.w,1.),("Height",&mut r.h,1.)]{ui.label(label);changed|=ui.add(egui::DragValue::new(value).speed(1.).range(min..=10000.)).changed();ui.end_row();}});
                if changed{self.checkpoint();self.apply_bounds(page,&id,r);}
                if e.kind!=ElementKind::Panel{ui.heading("Text");ui.label("Blank keeps live game text. {value} inserts the current value.");
                    let mut text=self.doc.custom_text(page,&id).unwrap_or("").to_owned();let mut size=e.size;let mut dx=e.dx;let mut dy=e.dy;
                    let mut changed=if e.kind==ElementKind::Content{ui.text_edit_multiline(&mut text).changed()}else{ui.text_edit_singleline(&mut text).changed()};
                    for (label,value,min,max) in [("Text size",&mut size,8.,120.),("Text X offset",&mut dx,-10000.,10000.),("Text Y offset",&mut dy,-10000.,10000.)]{ui.horizontal(|ui|{ui.label(label);changed|=ui.add(egui::DragValue::new(value).speed(1.).range(min..=max)).changed();});}
                    if changed{self.checkpoint();self.doc.set_style(page,&id,&text,size,dx,dy);}
                }
                if ui.button("Reset this element").clicked(){self.checkpoint();if let Some(s)=self.doc.root["screens"][page].as_object_mut(){for section in ["elements","labels"]{if let Some(m)=s.get_mut(section).and_then(Value::as_object_mut){m.remove(&id);}}}}
            }}else{ui.label("Select a menu element on the left or on the canvas.");}
            ui.separator();ui.label(&self.status);ui.label("Menu files are separate from gameplay layouts. Save, then F5 in the game. Match the profile and aspect in Settings.");
        });});
        egui::CentralPanel::default().show(ctx, |ui| {
            let viewport = ui.max_rect();
            let (w, h) = self.doc.canvas();
            let fit = ((viewport.width() - 32.) / w)
                .min((viewport.height() - 32.) / h)
                .max(0.05);
            if self.zoom <= 0. {
                self.zoom = fit;
            }
            let response = ui.allocate_rect(viewport, Sense::click_and_drag());
            if response.hovered() {
                let wheel = ctx.input(|i| {
                    if i.modifiers.ctrl {
                        i.raw_scroll_delta.y
                    } else {
                        0.
                    }
                });
                if wheel != 0. {
                    self.zoom = (self.zoom * (wheel * 0.002).exp()).clamp(0.05, 4.);
                }
            }
            if response.dragged_by(egui::PointerButton::Middle) {
                self.pan += ctx.input(|i| i.pointer.delta());
            }
            let board =
                Rect::from_center_size(viewport.center() + self.pan, Vec2::new(w, h) * self.zoom);
            let painter = ui.painter_at(viewport);
            painter.rect_filled(viewport, 0., Color32::from_rgb(20, 25, 34));
            painter.rect_filled(board, 0., Color32::from_rgb(14, 23, 38));
            let elements = self.doc.elements(page);
            let mut hits = vec![];
            for e in &elements {
                let b = e.bounds;
                let r = Rect::from_min_size(
                    board.min + Vec2::new(b.x, b.y) * self.zoom,
                    Vec2::new(b.w, b.h) * self.zoom,
                );
                hits.push((e.id.clone(), r, e.kind));
                if matches!(
                    e.kind,
                    ElementKind::Panel | ElementKind::Content | ElementKind::Button
                ) {
                    let accent = Color32::from_rgb(64, 207, 184);
                    let offset = if e.kind == ElementKind::Button {
                        Vec2::new(3., 5.)
                    } else {
                        Vec2::new(9., 12.)
                    } * self.zoom;
                    painter.rect_filled(r.translate(offset), 0., Color32::from_black_alpha(64));
                    painter.rect_filled(
                        r,
                        0.,
                        if e.kind == ElementKind::Button {
                            Color32::from_rgb(33, 59, 79)
                        } else {
                            Color32::from_rgb(26, 41, 61)
                        },
                    );
                    painter.rect_stroke(
                        r,
                        0.,
                        Stroke::new(
                            self.zoom.max(0.5),
                            if e.kind == ElementKind::Button {
                                accent
                            } else {
                                Color32::from_rgb(82, 120, 140)
                            },
                        ),
                    );
                    painter.rect_filled(
                        Rect::from_min_size(r.min, Vec2::new(5. * self.zoom, r.height())),
                        0.,
                        accent,
                    );
                }
                if e.kind == ElementKind::Content {
                    let inset = Vec2::new(50., 60.) * self.zoom;
                    let content =
                        Rect::from_min_max(r.min + inset, r.max - Vec2::new(80., 70.) * self.zoom);
                    let galley = painter.layout(
                        e.text.clone(),
                        FontId::proportional(e.size * self.zoom),
                        Color32::WHITE,
                        content.width(),
                    );
                    if self.preview
                        && content
                            .contains(ctx.input(|i| i.pointer.hover_pos()).unwrap_or(Pos2::ZERO))
                    {
                        self.scroll = (self.scroll - ctx.input(|i| i.raw_scroll_delta.y))
                            .clamp(0., (galley.size().y - content.height()).max(0.));
                    }
                    painter.with_clip_rect(content).galley(
                        content.min + Vec2::new(e.dx, e.dy) * self.zoom
                            - Vec2::new(0., self.scroll),
                        galley,
                        Color32::WHITE,
                    );
                } else if e.kind != ElementKind::Panel {
                    painter.text(
                        r.center() + Vec2::new(e.dx, e.dy) * self.zoom,
                        Align2::CENTER_CENTER,
                        &e.text,
                        FontId::proportional((e.size * self.zoom).max(1.)),
                        if e.id == "title" {
                            Color32::from_rgb(64, 207, 184)
                        } else if e.id.ends_with("heading")
                            || e.id.ends_with("hint")
                            || e.id == "difficulty_note"
                            || e.id == "note"
                        {
                            Color32::from_rgb(153, 173, 199)
                        } else {
                            Color32::WHITE
                        },
                    );
                }
                if self.selected.as_deref() == Some(&e.id) {
                    painter.rect_stroke(r, 0., Stroke::new(2_f32, Color32::from_rgb(40, 210, 180)));
                }
            }
            let pointer = response.interact_pointer_pos();
            let hit = pointer.and_then(|p| {
                hits.iter()
                    .rev()
                    .find(|(_, r, _)| r.contains(p))
                    .map(|(id, _, kind)| (id.clone(), *kind))
            });
            if response.clicked_by(egui::PointerButton::Primary) {
                if let Some((id, kind)) = hit.clone() {
                    if self.preview && kind == ElementKind::Button {
                        self.navigate(&id);
                    } else {
                        self.selected = Some(id);
                    }
                } else {
                    self.selected = None;
                }
            }
            if !self.preview && response.drag_started_by(egui::PointerButton::Primary) {
                if let (Some(p), Some((id, _))) = (pointer, hit) {
                    self.selected = Some(id.clone());
                    self.checkpoint();
                    self.drag = Some((p, self.doc.root.clone(), id));
                }
            }
            if !self.preview && response.dragged_by(egui::PointerButton::Primary) {
                if let (Some(p), Some((start, original, id))) = (pointer, self.drag.clone()) {
                    let delta = (p - start) / self.zoom;
                    self.doc.root = original;
                    let r = self.doc.element(page, &id).unwrap().bounds;
                    self.apply_bounds(
                        page,
                        &id,
                        Bounds {
                            x: (r.x + delta.x).round(),
                            y: (r.y + delta.y).round(),
                            ..r
                        },
                    );
                }
            }
            if response.drag_stopped_by(egui::PointerButton::Primary) {
                self.drag = None;
            }
        });
    }
}
