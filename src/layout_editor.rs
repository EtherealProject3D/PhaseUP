use crate::*;
use std::cell::Cell;
thread_local! {static EDITING:Cell<bool>=const {Cell::new(false)}; static ZOOM:Cell<f32>=Cell::new(1.0); static PAN:Cell<(f32,f32)>=Cell::new((0.0,0.0));}
pub(super) fn editing()->bool{EDITING.with(|v|v.get())}
pub(super) fn zoom()->f32{ZOOM.with(|z|z.get())}
pub(super) fn pan()->(f32,f32){PAN.with(|p|p.get())}


#[derive(Clone)]
struct SelectionDrag {resize:bool,start:Vec2,originals:Vec<(String,layout::Rect)>,recorded:bool}

pub(super) struct Editor {selection:Vec<String>, panel_position:Vec2, panel_drag:Option<Vec2>, selected:usize, selected_name:String, resize_mode:bool, panel_visible:bool, show_all_bounds:bool, step:f32, next_repeat:f64, drag:Option<SelectionDrag>, status:String, undo:Vec<Vec<(String,layout::Rect)>>}
impl Editor {
    pub fn new()->Self {Self{selection:Vec::new(),panel_position:vec2(1430.,480.),panel_drag:None,selected:0,selected_name:String::new(),resize_mode:false,panel_visible:true,show_all_bounds:false,step:1.,next_repeat:0.,drag:None,status:"Select an element | Arrows: nudge | Ctrl+S: save".into(),undo:Vec::new()}}
    #[cfg(not(target_os="android"))]
    fn save_layout(&mut self){
        self.status=match layout::save(){
            Ok(())=>"SAVED: assets/layouts/game_16_10.json".into(),
            Err(e)=>format!("SAVE FAILED: {e}"),
        };
    }
    fn selected_rects(&self,items:&[(String,layout::Rect)])->Vec<(String,layout::Rect)> {
        items.iter().filter(|(name,_)|self.selection.contains(name))
            .map(|(name,default)|(name.clone(),layout::rect(name,*default))).collect()
    }
    fn remember(&mut self,batch:Vec<(String,layout::Rect)>){
        if batch.is_empty(){return;}
        self.undo.push(batch);
        if self.undo.len()>100{self.undo.remove(0);}
    }
    fn undo_last(&mut self)->bool {
        if let Some(batch)=self.undo.pop(){
            layout::set_many(&batch);
            self.status=format!("Undid {} element(s)",batch.len());
            true
        }else{false}
    }
    fn transform_selection(&mut self,items:&[(String,layout::Rect)],dx:f32,dy:f32,resize:bool){
        let originals=self.selected_rects(items);
        self.remember(originals.clone());
        let changes:Vec<_>=originals.into_iter().map(|(name,r)|(name,layout::adjusted(r,dx,dy,resize))).collect();
        layout::set_many(&changes);
    }
    pub fn open(&mut self){EDITING.with(|v|v.set(true));}
    pub fn close(&mut self){EDITING.with(|v|v.set(false));self.drag=None;self.panel_drag=None;ZOOM.with(|z|z.set(1.0));PAN.with(|p|p.set((0.0,0.0)));}
    pub fn draw(&mut self,preview:&mut Game,wireframe:&mut bool){
        self.open();
        if is_key_pressed(KeyCode::F8){self.panel_visible=!self.panel_visible;self.panel_drag=None;}
        let wheel=mouse_wheel().1;
        if wheel!=0.0 && (is_key_down(KeyCode::LeftControl)||is_key_down(KeyCode::RightControl)) {
            // Keep the world-space point under the cursor stationary as zoom changes.
            let (pw,ph)=(screen_width(),screen_height());
            let scale=(pw/1920.0).min(ph/game_canvas_height());
            let (mx,my)=mouse_position();
            let cursor_x=(mx-pw*0.5)/scale;
            let cursor_y=(my-ph*0.5)/scale;
            let old_zoom=zoom();
            let new_zoom=(old_zoom+wheel.signum()*0.25).clamp(1.0,3.0);
            PAN.with(|p|{
                let (x,y)=p.get();
                p.set((x+cursor_x*(1.0/old_zoom-1.0/new_zoom),
                       y+cursor_y*(1.0/old_zoom-1.0/new_zoom)));
            });
            ZOOM.with(|z|z.set(new_zoom));
        }
        if is_key_pressed(KeyCode::Key0) {
            ZOOM.with(|z|z.set(1.0));
            PAN.with(|p|p.set((0.0,0.0)));
        }
        let items=crate::layout::editor_items_for(preview);
        if items.is_empty(){return;}
        self.selection.retain(|key|items.iter().any(|(name,_)|name==key));
        if self.selected_name.is_empty(){self.selection=vec![items[0].0.clone()];}
        if let Some(index)=items.iter().position(|(name,_)|name==&self.selected_name){self.selected=index;}
        self.selected=self.selected.min(items.len()-1);
        if is_key_pressed(KeyCode::Tab){
            let backwards=is_key_down(KeyCode::LeftShift)||is_key_down(KeyCode::RightShift);
            self.selected=if backwards {(self.selected+items.len()-1)%items.len()}else{(self.selected+1)%items.len()};
            self.selection=vec![items[self.selected].0.clone()];
        }
        let (mx,my)=game_mouse();
        let pointer=tool_mouse();
        if self.panel_visible && is_mouse_button_pressed(MouseButton::Left)
            && tool_contains(self.panel_position.x,self.panel_position.y,470.,30.) {
            self.panel_drag=Some(pointer-self.panel_position);
            self.drag=None;
        }
        if let Some(offset)=self.panel_drag {
            self.panel_position=tool_panel_position(pointer-offset);
            if !is_mouse_button_down(MouseButton::Left){self.panel_drag=None;}
        }
        let on_controls=self.panel_visible && (self.panel_drag.is_some()
            || tool_contains(self.panel_position.x,self.panel_position.y,470.,620.));
        if is_mouse_button_pressed(MouseButton::Left) && !on_controls {
            self.drag=None;
            // Prefer the smallest containing element so foreground controls
            // remain selectable inside large background panels and banners.
            let hit=(0..items.len()).rev().filter(|&i|{
                let (name,default)=&items[i];let r=layout::rect(name,*default);
                mx>=r.x && mx<=r.x+r.w && my>=r.y && my<=r.y+r.h
            }).min_by(|&a,&b|{
                let ra=layout::rect(&items[a].0,items[a].1);
                let rb=layout::rect(&items[b].0,items[b].1);
                (ra.w*ra.h).total_cmp(&(rb.w*rb.h))
            });
            let additive=is_key_down(KeyCode::LeftControl)||is_key_down(KeyCode::RightControl);
            if let Some(i)=hit {
                let (name,default)=&items[i];let r=layout::rect(name,*default);
                selection_click(&mut self.selection,name,additive);
                self.selected=if self.selection.contains(name){i}
                    else {self.selection.last().and_then(|key|items.iter().position(|(n,_)|n==key)).unwrap_or(i)};
                if !additive {
                    let resize=mx>r.x+r.w-20. && my>r.y+r.h-20.;
                    self.drag=Some(SelectionDrag{resize,start:vec2(mx,my),originals:self.selected_rects(&items),recorded:false});
                }
            } else if !additive {self.selection.clear();}
        }
        if let Some(mut drag)=self.drag.take(){
            let delta=vec2((mx-drag.start.x).round(),(my-drag.start.y).round());
            if delta.x!=0. || delta.y!=0. || drag.recorded {
                if !drag.recorded {self.remember(drag.originals.clone());drag.recorded=true;}
                let changes:Vec<_>=drag.originals.iter().map(|(name,r)|(name.clone(),layout::adjusted(*r,delta.x,delta.y,drag.resize))).collect();
                layout::set_many(&changes);
            }
            if !is_mouse_button_down(MouseButton::Left){
                #[cfg(not(target_os="android"))]
                if drag.recorded {self.save_layout();}
            } else {self.drag=Some(drag);}
        }
        for (name,default) in &items {
            let included=self.selection.contains(name);
            if !self.show_all_bounds && !included {continue;}
            let r=layout::rect(name,*default);
            let color=if included {YELLOW}else{ACCENT};
            draw_rectangle_lines(r.x,r.y,r.w,r.h,3.,color);
            draw_rectangle(r.x+r.w-16.,r.y+r.h-16.,14.,14.,color);
            menu_label(name,r.x,r.y-5.,17.,color);
        }
        let (pw,ph)=(screen_width(),screen_height());
        let hud_scale=(pw/1920.).min(ph/game_canvas_height());
        set_camera(&Camera2D{target:vec2(960.,game_canvas_height()/2.),zoom:vec2(2.*hud_scale/pw,2.*hud_scale/ph),..Default::default()});
        let tx=self.panel_position.x;
        let ty=self.panel_position.y;
        let mut dx=0.;let mut dy=0.;
        if self.panel_visible {
        draw_rectangle(tx,ty,470.,620.,Color::new(0.01,0.025,0.045,0.97));
        draw_rectangle(tx,ty,470.,30.,Color::new(0.08,0.23,0.30,1.));
        menu_label("WORKSHOP TOOLS - DRAG HERE",tx+12.,ty+21.,17.,WHITEISH);
        if tool_button("PREVIOUS",tx+10.,ty+42.,218.,32.,true){self.selected=(self.selected+items.len()-1)%items.len();self.selection=vec![items[self.selected].0.clone()];}
        if tool_button("NEXT",tx+238.,ty+42.,218.,32.,true){self.selected=(self.selected+1)%items.len();self.selection=vec![items[self.selected].0.clone()];}
        let selected=layout::rect(&items[self.selected].0,items[self.selected].1);
        let selection_title=if self.selection.len()>1 {format!("{} selected | {}",self.selection.len(),items[self.selected].0)}
            else if self.selection.is_empty(){"No selection - click or Ctrl+click".to_owned()}else{items[self.selected].0.clone()};
        menu_label(&selection_title,tx+10.,ty+102.,16.,YELLOW);
        menu_label(&format!("X {:.0} Y {:.0}  W {:.0} H {:.0}",selected.x,selected.y,selected.w,selected.h),tx+10.,ty+130.,18.,WHITEISH);
        if tool_button(if self.resize_mode {"MODE: RESIZE"}else{"MODE: MOVE"},tx+10.,ty+145.,218.,36.,true){self.resize_mode=!self.resize_mode;}
        if tool_button(&format!("STEP: {:.0} PX",self.step),tx+238.,ty+145.,218.,36.,true){self.step=if self.step==1. {5.}else if self.step==5. {10.}else{1.};}
        if tool_button("LEFT",tx+10.,ty+194.,104.,40.,true){dx-=1.;}
        if tool_button("RIGHT",tx+124.,ty+194.,104.,40.,true){dx+=1.;}
        if tool_button("UP",tx+238.,ty+194.,104.,40.,true){dy-=1.;}
        if tool_button("DOWN",tx+352.,ty+194.,104.,40.,true){dy+=1.;}
        }
        let pressed=[KeyCode::Left,KeyCode::Right,KeyCode::Up,KeyCode::Down].iter().any(|k|is_key_pressed(*k));
        let held=[KeyCode::Left,KeyCode::Right,KeyCode::Up,KeyCode::Down].iter().any(|k|is_key_down(*k));
        if pressed || (held && get_time()>=self.next_repeat){
            if is_key_down(KeyCode::Left){dx-=1.;} if is_key_down(KeyCode::Right){dx+=1.;}
            if is_key_down(KeyCode::Up){dy-=1.;} if is_key_down(KeyCode::Down){dy+=1.;}
            self.next_repeat=get_time()+if pressed {0.35}else{0.08};
        }
        let shift=is_key_down(KeyCode::LeftShift)||is_key_down(KeyCode::RightShift);
        if (dx!=0. || dy!=0.) && !self.selection.is_empty() {
            let step=self.step*if shift {10.}else{1.};
            self.transform_selection(&items,dx*step,dy*step,self.resize_mode);
            #[cfg(not(target_os="android"))]
            self.save_layout();
        }
        if self.panel_visible {
        menu_label("Arrow keys nudge; Shift = 10x step",tx+10.,ty+268.,18.,WHITEISH);
        menu_label("Ctrl+click adds/removes; Tab cycles",tx+10.,ty+296.,18.,WHITEISH);
        menu_label("Resize: L/R width, U/D height",tx+10.,ty+324.,18.,WHITEISH);
        if tool_button("RESET SELECTED",tx+10.,ty+347.,446.,40.,true){
            self.remember(self.selected_rects(&items));
            let changes:Vec<_>=items.iter().filter(|(name,_)|self.selection.contains(name)).cloned().collect();
            layout::set_many(&changes);
            #[cfg(not(target_os="android"))]
            self.save_layout();
        }
        for (count,x) in [(1,tx+10.),(2,tx+238.)] {
            if tool_button(&format!("HUMAN: {count} GROUP{}",if count==1 {""}else{"S"}),x,ty+397.,218.,36.,true) {
                if let Some(phase)=PHASES.iter().position(|groups|groups.len()==count){
                    preview.players[0].phase=phase;
                    preview.players[0].laid=true;
                    preview.players[0].table=(0..count).map(|j|(0..3).map(|k|Card{id:9100+j as u32*10+k,kind:Kind::Number(5,(k%4) as u8)}).collect()).collect();
                    preview.staged=[vec![],vec![]];
                }
            }
        }
        if tool_button(if self.show_all_bounds {"ALL OUTLINES: ON"}else{"ALL OUTLINES: OFF"},tx+10.,ty+489.,446.,36.,true){self.show_all_bounds=!self.show_all_bounds;}
        if tool_button(if *wireframe {"SHOW ARTWORK (F7)"}else{"SHOW WIREFRAME (F7)"},tx+10.,ty+443.,446.,36.,true){*wireframe=!*wireframe;}
        }
        self.selected_name=items[self.selected].0.clone();
        #[cfg(not(target_os="android"))]
        if is_key_pressed(KeyCode::S) && (is_key_down(KeyCode::LeftControl)||is_key_down(KeyCode::RightControl)) {
            self.save_layout();
        }
        let undo_key=is_key_pressed(KeyCode::Z) && (is_key_down(KeyCode::LeftControl)||is_key_down(KeyCode::RightControl));
        if undo_key || (self.panel_visible && tool_button("UNDO",tx+10.,ty+550.,218.,36.,true)) {
            if self.undo_last(){
                #[cfg(not(target_os="android"))]
                self.save_layout();
            }
        }
        if self.panel_visible && tool_button("SAVE LAYOUT",tx+238.,ty+550.,218.,36.,true) {
            #[cfg(not(target_os="android"))]
            self.save_layout();
        }
        if self.panel_visible {
            let summary=if self.status.starts_with("SAVED") {"Saved"}else{self.status.as_str()};
            menu_label(&summary.chars().take(58).collect::<String>(),tx+10.,ty+608.,14.,MUTED);
        }
    }
}

// Tool coordinates are independent of board zoom/pan so the window stays usable.
fn tool_mouse()->Vec2 {
    let (pw,ph)=(screen_width(),screen_height());
    let scale=(pw/1920.).min(ph/game_canvas_height());
    let (mx,my)=mouse_position();
    vec2((mx-pw/2.)/scale+960.,(my-ph/2.)/scale+game_canvas_height()/2.)
}
fn tool_contains(x:f32,y:f32,w:f32,h:f32)->bool {
    let p=tool_mouse();p.x>=x && p.x<=x+w && p.y>=y && p.y<=y+h
}
pub(super) fn tool_panel_position(position:Vec2)->Vec2 {
    vec2(position.x.clamp(0.,1920.-470.),position.y.clamp(0.,1200.-620.))
}
fn tool_button(text:&str,x:f32,y:f32,w:f32,h:f32,enabled:bool)->bool {
    let hover=tool_contains(x,y,w,h);
    crate::theme::themed_button(x,y,w,h,hover && enabled);
    let tw=menu_text_width(text,19);
    menu_label(text,x+(w-tw)/2.,y+h/2.+7.,19.,if enabled {WHITEISH}else{MUTED});
    enabled && hover && is_mouse_button_pressed(MouseButton::Left)
}

// Ctrl toggles membership; dragging an existing member preserves its group.
pub(super) fn selection_click(selection:&mut Vec<String>,name:&str,additive:bool){
    if additive {
        if let Some(index)=selection.iter().position(|key|key==name){selection.remove(index);}
        else {selection.push(name.to_owned());}
    } else if !selection.iter().any(|key|key==name){
        selection.clear();selection.push(name.to_owned());
    }
}

#[cfg(test)]
mod multi_selection_tests {
    use super::*;
    #[test]
    fn ctrl_click_toggles_without_duplicates_and_normal_drag_preserves_group(){
        let mut selected=Vec::new();
        selection_click(&mut selected,"panel",false);
        selection_click(&mut selected,"label",true);
        assert_eq!(selected,vec!["panel","label"]);
        selection_click(&mut selected,"panel",false);
        assert_eq!(selected.len(),2);
        selection_click(&mut selected,"label",true);
        assert_eq!(selected,vec!["panel"]);
        selection_click(&mut selected,"other",false);
        assert_eq!(selected,vec!["other"]);
        selection_click(&mut selected,"other",true);
        assert!(selected.is_empty());
    }
    #[test]
    fn nudge_moves_all_members_and_undo_restores_the_batch(){
        let a=layout::Rect{x:10.,y:20.,w:100.,h:40.};
        let b=layout::Rect{x:130.,y:60.,w:90.,h:30.};
        let items=vec![("multi_test_a".to_owned(),a),("multi_test_b".to_owned(),b)];
        let mut editor=Editor::new();
        editor.selection=items.iter().map(|(name,_)|name.clone()).collect();
        editor.transform_selection(&items,5.,-10.,false);
        assert_eq!(editor.undo.len(),1);
        let moved_a=layout::rect(&items[0].0,a);let moved_b=layout::rect(&items[1].0,b);
        assert_eq!((moved_a.x,moved_a.y),(15.,10.));
        assert_eq!((moved_b.x,moved_b.y),(135.,50.));
        assert_eq!(moved_b.x-moved_a.x,b.x-a.x);
        assert!(editor.undo_last());
        let restored_a=layout::rect(&items[0].0,a);let restored_b=layout::rect(&items[1].0,b);
        assert_eq!((restored_a.x,restored_a.y),(a.x,a.y));
        assert_eq!((restored_b.x,restored_b.y),(b.x,b.y));
        assert!(!editor.undo_last());
    }
}
