use crate::*;

pub(super) fn game_mouse()->(f32,f32) {
    let (pw,ph)=(screen_width(),screen_height());
    let scale=(pw/1600.0).min(ph/1200.0);
    let (mx,my)=mouse_position();
    ((mx-(pw-1600.0*scale)/2.0)/scale,(my-(ph-1200.0*scale)/2.0)/scale)
}
pub(super) fn label(text:&str,x:f32,y:f32,size:f32,color:Color){draw_text(text,x,y,size,crate::theme::themed_color(color));}
pub(super) fn button(text:&str,x:f32,y:f32,w:f32,h:f32,enabled:bool)->bool{
    let mouse=game_mouse();let hover=mouse.0>=x&&mouse.0<=x+w&&mouse.1>=y&&mouse.1<=y+h;
    draw_rectangle(x,y,w,h,if !enabled{Color::new(0.18,0.22,0.28,1.)}else if hover{crate::theme::active_theme().accent()}else{crate::theme::active_theme().panel()});
    let t=measure_text(text,None,19,1.);label(text,x+(w-t.width)/2.,y+h/2.+7.,19.,if enabled{WHITEISH}else{MUTED});
    enabled&&hover&&is_mouse_button_pressed(MouseButton::Left)
}
pub(super) fn card_ui(card:Card,x:f32,y:f32,w:f32,h:f32,selected:bool)->bool {
    let yy=if selected{y-15.}else{y};let c=color_for(card.kind);
    draw_rectangle(x+2.,yy+3.,w,h,Color::new(0.,0.,0.,0.3));draw_rectangle(x,yy,w,h,WHITEISH);
    draw_rectangle(x+5.,yy+5.,w-10.,h-10.,c);
    let t=card_text(card.kind);let size: f32=if w<45.0{23.0}else{32.0};let dims=measure_text(&t,None,size as u16,1.0);
    label(&t,x+(w-dims.width)/2.,yy+h/2.+dims.height/2.,size,if matches!(card.kind,Kind::Number(_,2)){BG}else{WHITE});
    let m=game_mouse();m.0>=x&&m.0<=x+w&&m.1>=yy&&m.1<=yy+h&&is_mouse_button_pressed(MouseButton::Left)
}
pub(super) fn column_hand_width(count:usize,card_w:f32)->f32 {
    if count<=1 {0.0}else{(count-1) as f32*24.0_f32.min((450.0-card_w)/(count-1) as f32)}
}
pub(super) fn cpu_card_back(x:f32,y:f32,w:f32,h:f32) {
    draw_rectangle(x+2.0,y+3.0,w,h,Color::new(0.0,0.0,0.0,0.28));
    draw_rectangle(x,y,w,h,WHITEISH);
    draw_rectangle(x+3.0,y+3.0,w-6.0,h-6.0,Color::new(0.12,0.39,0.48,1.0));
    draw_rectangle_lines(x+6.0,y+6.0,w-12.0,h-12.0,1.5,ACCENT);
    draw_line(x+8.0,y+8.0,x+w-8.0,y+h-8.0,1.0,ACCENT);
    draw_line(x+w-8.0,y+8.0,x+8.0,y+h-8.0,1.0,ACCENT);
}
