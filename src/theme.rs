use macroquad::prelude::*;
use std::cell::Cell;

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub(super) enum Theme { Modern, Futuristic, Playful }
impl Theme {
    pub(super) fn next(self)->Self {match self {Self::Modern=>Self::Futuristic,Self::Futuristic=>Self::Playful,Self::Playful=>Self::Modern}}
    pub(super) fn name(self)->&'static str {match self {Self::Modern=>"MODERN & CLEAN",Self::Futuristic=>"BOLD & FUTURISTIC",Self::Playful=>"COLORFUL & PLAYFUL"}}
    pub(super) fn key(self)->&'static str {match self {Self::Modern=>"modern",Self::Futuristic=>"futuristic",Self::Playful=>"playful"}}
    pub(super) fn from_key(s:&str)->Self {match s {"futuristic"=>Self::Futuristic,"playful"=>Self::Playful,_=>Self::Modern}}
    pub(super) fn accent(self)->Color {match self {Self::Modern=>Color::new(0.25,0.81,0.72,1.),Self::Futuristic=>Color::new(0.0,0.88,1.0,1.),Self::Playful=>Color::new(1.0,0.75,0.30,1.)}}
    pub(super) fn panel(self)->Color {match self {Self::Modern=>Color::new(0.10,0.16,0.24,1.),Self::Futuristic=>Color::new(0.035,0.11,0.20,1.),Self::Playful=>Color::new(0.12,0.24,0.22,1.)}}
    pub(super) fn background(self)->Color {match self {Self::Modern=>Color::new(0.055,0.09,0.15,1.),Self::Futuristic=>Color::new(0.015,0.045,0.10,1.),Self::Playful=>Color::new(0.035,0.13,0.12,1.)}}
}
thread_local! {static ACTIVE:Cell<Theme>=const {Cell::new(Theme::Modern)};}
pub(super) fn active_theme()->Theme {ACTIVE.with(|v|v.get())}
pub(super) fn set_theme(theme:Theme) {ACTIVE.with(|v|v.set(theme));}
pub(super) fn themed_color(color:Color)->Color {
    let theme=active_theme();
    if theme==Theme::Modern {return color;}
    if color==crate::ACCENT {return theme.accent();}
    if color==crate::PANEL {return theme.panel();}
    if color==crate::BG {return theme.background();}
    color
}

pub(super) fn panel_decoration(x:f32,y:f32,w:f32,h:f32){
    let t=active_theme();let a=t.accent();
    match t {
        Theme::Modern=>{
            draw_rectangle(x+9.,y+12.,w,h,Color::new(0.,0.,0.,0.25));
            draw_rectangle(x,y,w,h,t.panel());
            draw_rectangle(x,y,5.,h,a);
            draw_rectangle_lines(x,y,w,h,1.,Color::new(0.32,0.47,0.55,0.6));
        }
        Theme::Futuristic=>{
            draw_rectangle(x,y,w,h,t.panel());
            draw_rectangle_lines(x-3.,y-3.,w+6.,h+6.,1.,Color::new(0.,0.50,0.66,0.6));
            draw_rectangle_lines(x,y,w,h,2.,a);
            let s=32.;
            for (xx,yy,dx,dy) in [(x,y,1.,1.),(x+w,y,-1.,1.),(x,y+h,1.,-1.),(x+w,y+h,-1.,-1.)]{
                draw_line(xx,yy,xx+dx*s,yy,4.,a);
                draw_line(xx,yy,xx,yy+dy*s,4.,a);
            }
        }
        Theme::Playful=>{
            draw_rectangle(x+9.,y+12.,w,h,Color::new(0.01,0.06,0.07,0.45));
            draw_rectangle(x,y,w,h,t.panel());
            draw_rectangle(x+12.,y+12.,w-24.,h-24.,Color::new(0.14,0.29,0.28,0.65));
            draw_rectangle_lines(x,y,w,h,5.,a);
            for (cx,cy) in [(x+12.,y+12.),(x+w-12.,y+12.),(x+12.,y+h-12.),(x+w-12.,y+h-12.)]{
                draw_circle(cx,cy,9.,a);
            }
        }
    }
}
pub(super) fn menu_background(){
    let t=active_theme();
    clear_background(t.background());
    match t {
        Theme::Modern=>{
            draw_rectangle(0.,0.,1600.,8.,t.accent());
            draw_circle(1500.,110.,180.,Color::new(0.10,0.22,0.28,0.22));
            draw_circle(110.,1040.,240.,Color::new(0.08,0.25,0.24,0.18));
        }
        Theme::Futuristic=>{
            let grid=Color::new(0.04,0.22,0.32,0.36);
            for x in (0..=1600).step_by(80){draw_line(x as f32,0.,x as f32,1200.,1.,grid);}
            for y in (0..=1200).step_by(80){draw_line(0.,y as f32,1600.,y as f32,1.,grid);}
            draw_line(0.,100.,1600.,100.,2.,t.accent());
            draw_line(0.,1100.,1600.,1100.,2.,t.accent());
        }
        Theme::Playful=>{
            let dots=[(120.,150.,27.),(1410.,190.,43.),(205.,920.,36.),(1370.,960.,28.),(315.,400.,18.),(1240.,620.,22.)];
            for (i,(x,y,r)) in dots.iter().enumerate(){
                let color=if i%2==0 {Color::new(1.,0.52,0.43,0.30)}else{Color::new(0.40,0.81,0.72,0.28)};
                draw_circle(*x,*y,*r,color);
            }
            draw_rectangle(0.,0.,1600.,12.,t.accent());
        }
    }
}
pub(super) fn themed_button(x:f32,y:f32,w:f32,h:f32,hover:bool){
    let t=active_theme();let a=t.accent();
    match t {
        Theme::Modern=>{
            draw_rectangle(x+3.,y+5.,w,h,Color::new(0.,0.,0.,0.25));
            draw_rectangle(x,y,w,h,if hover {Color::new(0.18,0.34,0.40,1.)}else{Color::new(0.13,0.23,0.31,1.)});
            draw_rectangle(x,y,5.,h,a);
            draw_rectangle_lines(x,y,w,h,1.,a);
        }
        Theme::Futuristic=>{
            if hover {draw_rectangle(x-3.,y-3.,w+6.,h+6.,Color::new(0.,0.45,0.55,0.35));}
            draw_rectangle(x,y,w,h,if hover{Color::new(0.06,0.25,0.34,1.)}else{t.panel()});
            draw_rectangle_lines(x,y,w,h,if hover{3.}else{2.},a);
            draw_line(x,y+h,x+30.,y+h,4.,a);
            draw_line(x+w-30.,y,x+w,y,4.,a);
        }
        Theme::Playful=>{
            draw_rectangle(x+5.,y+7.,w,h,Color::new(0.,0.05,0.06,0.4));
            draw_rectangle(x,y,w,h,if hover{Color::new(0.28,0.51,0.46,1.)}else{Color::new(0.20,0.39,0.37,1.)});
            draw_rectangle_lines(x,y,w,h,4.,a);
            draw_circle(x+13.,y+13.,8.,a);
            draw_circle(x+w-13.,y+13.,8.,a);
            draw_circle(x+13.,y+h-13.,8.,a);
            draw_circle(x+w-13.,y+h-13.,8.,a);
        }
    }
}
