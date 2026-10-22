use crate::*;

pub(super) struct Editor {pub enabled:bool, selected:usize, drag:Option<(bool,f32,f32,layout::Rect)>, status:String}
impl Editor {
    pub fn new()->Self {Self{enabled:false,selected:0,drag:None,status:"E: edit layout | Tab: select | Ctrl+S: save".into()}}
    #[cfg(not(target_os="android"))]
    fn save_layout(&mut self){
        self.status=match layout::save(){
            Ok(())=>"SAVED: assets/layouts/game_16_10.json".into(),
            Err(e)=>format!("SAVE FAILED: {e}"),
        };
    }
    pub fn draw(&mut self){
        if is_key_pressed(KeyCode::E) {self.enabled=!self.enabled;self.drag=None;}
        if !self.enabled {return;}
        const ITEMS:[(&str,layout::Rect);2]=[
            ("game_logo",layout::Rect{x:24.,y:10.,w:230.,h:42.}),
            ("turn_banner",layout::Rect{x:16.,y:55.,w:1888.,h:62.}),
        ];
        if is_key_pressed(KeyCode::Tab){self.selected=(self.selected+1)%ITEMS.len();}
        let (mx,my)=game_mouse();
        if is_mouse_button_pressed(MouseButton::Left){
            self.drag=None;
            for i in (0..ITEMS.len()).rev(){
                let (name,default)=ITEMS[i];let r=layout::rect(name,default);
                if mx>=r.x && mx<=r.x+r.w && my>=r.y && my<=r.y+r.h {
                    self.selected=i;
                    let resize=mx>r.x+r.w-20. && my>r.y+r.h-20.;
                    self.drag=Some((resize,mx,my,r));break;
                }
            }
        }
        if is_mouse_button_released(MouseButton::Left) && self.drag.is_some() {
            #[cfg(not(target_os="android"))]
            self.save_layout();
            self.drag=None;
        }
        if let Some((resize,start_x,start_y,original))=self.drag {
            let dx=mx-start_x;let dy=my-start_y;
            let mut r=original;
            if resize {r.w=(r.w+dx).max(40.);r.h=(r.h+dy).max(24.);}
            else {r.x+=dx;r.y+=dy;}
            layout::set(ITEMS[self.selected].0,r);
        }
        #[cfg(not(target_os="android"))]
        if is_key_pressed(KeyCode::S) && (is_key_down(KeyCode::LeftControl)||is_key_down(KeyCode::RightControl)) {
            self.save_layout();
        }
        if button("SAVE LAYOUT",1620.,1127.,270.,54.,true) {
            #[cfg(not(target_os="android"))]
            self.save_layout();
        }
        for (i,(name,default)) in ITEMS.iter().enumerate(){
            let r=layout::rect(name,*default);
            let color=if i==self.selected {YELLOW}else{ACCENT};
            draw_rectangle_lines(r.x,r.y,r.w,r.h,3.,color);
            draw_rectangle(r.x+r.w-16.,r.y+r.h-16.,14.,14.,color);
            menu_label(name,r.x,r.y-5.,17.,color);
        }
        draw_rectangle(550.,1130.,1050.,56.,Color::new(0.,0.,0.,0.9));
        menu_label(&format!("EDIT MODE | {} | {}",ITEMS[self.selected].0,self.status),566.,1164.,20.,WHITEISH);
    }
}
