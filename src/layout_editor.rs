use crate::*;
use std::cell::Cell;
thread_local! {static ZOOM:Cell<f32>=Cell::new(1.0); static PAN:Cell<(f32,f32)>=Cell::new((0.0,0.0));}
pub(super) fn zoom()->f32{ZOOM.with(|z|z.get())}
pub(super) fn pan()->(f32,f32){PAN.with(|p|p.get())}


pub(super) struct Editor {pub enabled:bool, selected:usize, drag:Option<(bool,f32,f32,layout::Rect)>, status:String, undo:Vec<(String,layout::Rect)>}
impl Editor {
    pub fn new()->Self {Self{enabled:false,selected:0,drag:None,status:"E: edit layout | Tab: select | Ctrl+S: save".into(),undo:Vec::new()}}
    #[cfg(not(target_os="android"))]
    fn save_layout(&mut self){
        self.status=match layout::save(){
            Ok(())=>"SAVED: assets/layouts/game_16_10.json".into(),
            Err(e)=>format!("SAVE FAILED: {e}"),
        };
    }
    pub fn close(&mut self){self.enabled=false;self.drag=None;ZOOM.with(|z|z.set(1.0));PAN.with(|p|p.set((0.0,0.0)));}
    pub fn draw(&mut self,preview:&Game){
        if is_key_pressed(KeyCode::E) {self.enabled=!self.enabled;self.drag=None;}
        if !self.enabled {ZOOM.with(|z|z.set(1.0));return;}
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
        if is_key_pressed(KeyCode::Tab){self.selected=(self.selected+1)%items.len();}
        let (mx,my)=game_mouse();
        if is_mouse_button_pressed(MouseButton::Left){
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
            if let Some(i)=hit {
                let (name,default)=&items[i];let r=layout::rect(name,*default);
                if mx>=r.x && mx<=r.x+r.w && my>=r.y && my<=r.y+r.h {
                    self.selected=i;
                    let resize=mx>r.x+r.w-20. && my>r.y+r.h-20.;
                    self.undo.push((name.clone(),r));
                    if self.undo.len()>100 {self.undo.remove(0);}
                    self.drag=Some((resize,mx,my,r));
                }
            }
        }
        if is_mouse_button_released(MouseButton::Left) && self.drag.is_some() {
            #[cfg(not(target_os="android"))]
            self.save_layout();
            self.drag=None;
        }
        if let Some((resize,start_x,start_y,original))=self.drag {
            let dx=(mx-start_x).round();let dy=(my-start_y).round();
            let mut r=original;
            if resize {r.w=(r.w+dx).max(40.);r.h=(r.h+dy).max(24.);}
            else {r.x+=dx;r.y+=dy;}
            r.x=r.x.round();r.y=r.y.round();r.w=r.w.round();r.h=r.h.round();
            layout::set(&items[self.selected].0,r);
        }
        #[cfg(not(target_os="android"))]
        if is_key_pressed(KeyCode::S) && (is_key_down(KeyCode::LeftControl)||is_key_down(KeyCode::RightControl)) {
            self.save_layout();
        }
        let undo_key=is_key_pressed(KeyCode::Z) && (is_key_down(KeyCode::LeftControl)||is_key_down(KeyCode::RightControl));
        if undo_key || button("UNDO",1330.,1127.,270.,54.,true) {
            if let Some((name,previous))=self.undo.pop(){
                layout::set(&name,previous);
                self.status=format!("UNDID {name}");
                #[cfg(not(target_os="android"))]
                self.save_layout();
            }
        }
        if button("SAVE LAYOUT",1620.,1127.,270.,54.,true) {
            #[cfg(not(target_os="android"))]
            self.save_layout();
        }
        for (i,(name,default)) in items.iter().enumerate(){
            let r=layout::rect(name,*default);
            let color=if i==self.selected {YELLOW}else{ACCENT};
            draw_rectangle_lines(r.x,r.y,r.w,r.h,3.,color);
            draw_rectangle(r.x+r.w-16.,r.y+r.h-16.,14.,14.,color);
            menu_label(name,r.x,r.y-5.,17.,color);
        }
        draw_rectangle(550.,123.,1050.,56.,Color::new(0.,0.,0.,0.9));
        menu_label(&format!("EDIT MODE | {} | {}",items[self.selected].0,format!("{} | zoom {:.2}x (Ctrl+wheel zoom to cursor | 0 reset)",self.status,zoom())),566.,157.,20.,WHITEISH);
    }
}
