// Android assets are packaged at the APK asset root; desktop assets live under assets/.
fn asset_prefix()->&'static str {if cfg!(target_os="android"){""}else{"assets/"}}
use std::cell::RefCell;
thread_local! {static BOARD_INPUT_BLOCKED:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
pub(super) fn board_input_blocked()->bool {BOARD_INPUT_BLOCKED.with(|b|b.get())}
pub(super) fn set_board_input_blocked(blocked:bool){BOARD_INPUT_BLOCKED.with(|b|b.set(blocked));}
thread_local! {
    static MENU_FONTS:RefCell<[Option<Font>;3]>=RefCell::new([None,None,None]);
    static NEON_BODY_FONT:RefCell<Option<Font>>=const {RefCell::new(None)};
}
pub(super) async fn load_menu_fonts(){
    // Windows fonts are optional: keep the original renderer as fallback.
    #[cfg(target_os="windows")]
    for (i,path) in ["C:/Windows/Fonts/segoeui.ttf","C:/Windows/Fonts/ariblk.ttf","C:/Windows/Fonts/comic.ttf"].iter().enumerate(){
        if let Ok(font)=load_ttf_font(path).await {
            MENU_FONTS.with(|f|f.borrow_mut()[i]=Some(font));
        }
    }
    // Bundle the Neon Forge family so desktop and Android use identical glyphs.
    if let Ok(font)=load_ttf_font(&format!("{}fonts/Oxanium-SemiBold.ttf",asset_prefix())).await {
        MENU_FONTS.with(|f|f.borrow_mut()[1]=Some(font));
    }
    if let Ok(font)=load_ttf_font(&format!("{}fonts/Oxanium-Medium.ttf",asset_prefix())).await {
        NEON_BODY_FONT.with(|f|*f.borrow_mut()=Some(font));
    }
}
fn with_menu_font<R>(size:u16,f:impl FnOnce(Option<&Font>)->R)->R {
    if active_theme()==Theme::Futuristic && size<=18 {
        return NEON_BODY_FONT.with(|font|{
            let font=font.borrow();
            if let Some(font)=font.as_ref(){f(Some(font))}
            else {MENU_FONTS.with(|fonts|f(fonts.borrow()[1].as_ref()))}
        });
    }
    let i=match active_theme(){Theme::Modern=>0,Theme::Futuristic=>1,Theme::Playful=>2};
    MENU_FONTS.with(|fonts|{let fonts=fonts.borrow();f(fonts[i].as_ref())})
}
pub(super) fn menu_text_width(text:&str,size:u16)->f32{
    with_menu_font(size,|font|measure_text(text,font,size,1.0).width)
}
pub(super) fn menu_text_dimensions(text:&str,size:u16)->TextDimensions {
    with_menu_font(size,|font|measure_text(text,font,size,1.0))
}
pub(super) fn menu_label(text:&str,x:f32,y:f32,size:f32,color:Color){
    with_menu_font(size as u16,|font|{
        draw_text_ex(text,x,y,TextParams{font,font_size:size as u16,font_scale:1.0,color:themed_color(color),..Default::default()});
    });
}
pub(super) fn menu_heading(text:&str,x:f32,y:f32,size:f32){
    if active_theme()!=Theme::Futuristic {menu_label(text,x,y,size,ACCENT);return;}
    // A quiet machined-metal bevel complements the brighter logo artwork.
    menu_label(text,x+1.5,y+2.0,size,Color::new(0.005,0.025,0.045,1.));
    menu_label(text,x,y-0.8,size,Color::new(0.64,0.80,0.85,1.));
    menu_label(text,x,y,size,Color::new(0.29,0.64,0.72,1.));
}
use crate::*;
thread_local! {static NEON_CARDS:RefCell<[Option<Texture2D>;7]>=RefCell::new([None,None,None,None,None,None,None]);}
pub(super) async fn load_neon_cards(){
    for (i,name) in ["red","blue","yellow","green","back","wild","skip"].iter().enumerate(){
        let path=format!("{}neon_forge/cards/{}.png",asset_prefix(),name);
        if let Ok(texture)=load_texture(&path).await {
            texture.set_filter(FilterMode::Linear);
            NEON_CARDS.with(|cards|cards.borrow_mut()[i]=Some(texture));
        }
    }
}
fn neon_card_texture(index:usize,x:f32,y:f32,w:f32,h:f32)->bool {
    neon_card_texture_tinted(index,x,y,w,h,WHITE)
}
fn neon_card_texture_tinted(index:usize,x:f32,y:f32,w:f32,h:f32,tint:Color)->bool {
    NEON_CARDS.with(|cards|{
        let cards=cards.borrow();
        if let Some(tex)=cards.get(index).and_then(|v|v.as_ref()){
            draw_texture_ex(tex,x,y,tint,DrawTextureParams{dest_size:Some(vec2(w,h)),..Default::default()});
            true
        }else{false}
    })
}


// Two layouts (single/double requirement), each with normal and active artwork.
thread_local! {
    static NEON_CPU_FRAMES:RefCell<[[Option<Texture2D>;2];2]>=RefCell::new([[None,None],[None,None]]);
}
pub(super) async fn load_neon_ui(){
    load_neon_player_phase().await;
    if let Ok(texture)=load_texture(&format!("{}neon_forge/ui/hand_panel.png",asset_prefix())).await {
        texture.set_filter(FilterMode::Linear);
        NEON_HAND_PANEL.with(|slot|*slot.borrow_mut()=Some(texture));
    }
    for (i,name) in ["staging_slot_single.png","staging_slot_double.png"].iter().enumerate(){
        if let Ok(texture)=load_texture(&format!("{}neon_forge/ui/{}",asset_prefix(),name)).await {
            texture.set_filter(FilterMode::Linear);
            NEON_STAGING.with(|slots|slots.borrow_mut()[i]=Some(texture));
        }
    }
    let path=format!("{}neon_forge/ui/game_background.png",asset_prefix());
    if let Ok(texture)=load_texture(&path).await {
        texture.set_filter(FilterMode::Linear);
        NEON_BACKGROUND.with(|slot|*slot.borrow_mut()=Some(texture));
    }
    let path=format!("{}neon_forge/ui/button_metal.png",asset_prefix());
    if let Ok(texture)=load_texture(&path).await {
        texture.set_filter(FilterMode::Linear);
        NEON_METAL_BUTTON.with(|slot|*slot.borrow_mut()=Some(texture));
    }
    let path=format!("{}neon_forge/ui/phaseup_logo.png",asset_prefix());
    if let Ok(texture)=load_texture(&path).await {
        texture.set_filter(FilterMode::Linear);
        NEON_LOGO.with(|slot|*slot.borrow_mut()=Some(texture));
    }
    let path=format!("{}neon_forge/ui/upper_banner.png",asset_prefix());
    if let Ok(texture)=load_texture(&path).await {
        texture.set_filter(FilterMode::Linear);
        NEON_UPPER_BANNER.with(|slot|*slot.borrow_mut()=Some(texture));
    }
    for (layout,names) in [
        ["cpu_panel_single.png","cpu_panel_single_active.png"],
        ["cpu_panel_double.png","cpu_panel_double_active.png"],
    ].iter().enumerate(){
        for (state,name) in names.iter().enumerate(){
            let path=format!("{}neon_forge/ui/{}",asset_prefix(),name);
            if let Ok(texture)=load_texture(&path).await {
                texture.set_filter(FilterMode::Linear);
                NEON_CPU_FRAMES.with(|slots|slots.borrow_mut()[layout][state]=Some(texture));
            }
        }
    }
}
pub(super) fn neon_cpu_panel(x:f32,y:f32,w:f32,h:f32,active_alpha:f32,single:bool){
    let layout=if single{0}else{1};
    let painted=NEON_CPU_FRAMES.with(|slots|{
        let slots=slots.borrow();
        if let Some(texture)=slots[layout][0].as_ref(){
            draw_texture_ex(texture,x,y,WHITE,DrawTextureParams{dest_size:Some(vec2(w,h)),..Default::default()});
            true
        }else{false}
    });
    if !painted {neon_frame(x,y,w,h,true);}
    if active_alpha>0.0 {
        NEON_CPU_FRAMES.with(|slots|{
            if let Some(texture)=slots.borrow()[layout][1].as_ref(){
                draw_texture_ex(texture,x,y,Color::new(1.0,1.0,1.0,active_alpha.clamp(0.0,1.0)),DrawTextureParams{
                    dest_size:Some(vec2(w,h)),..Default::default()
                });
            }
        });
    }
}
thread_local! {
    static NEON_PLAYER_PHASE_FRAMES:RefCell<[Option<Texture2D>;2]>=RefCell::new([None,None]);
    static NEON_UPPER_BANNER:RefCell<Option<Texture2D>>=const {RefCell::new(None)};
    static NEON_LOGO:RefCell<Option<Texture2D>>=const {RefCell::new(None)};
    static NEON_METAL_BUTTON:RefCell<Option<Texture2D>>=const {RefCell::new(None)};
    static NEON_BACKGROUND:RefCell<Option<Texture2D>>=const {RefCell::new(None)};
    static NEON_STAGING:RefCell<[Option<Texture2D>;2]>=RefCell::new([None,None]);
    static NEON_HAND_PANEL:RefCell<Option<Texture2D>>=const {RefCell::new(None)};
}
pub(super) fn neon_hand_panel(x:f32,y:f32,w:f32,h:f32){
    let painted=NEON_HAND_PANEL.with(|slot|{
        if let Some(texture)=slot.borrow().as_ref(){
            draw_texture_ex(texture,x,y,WHITE,DrawTextureParams{dest_size:Some(vec2(w,h)),..Default::default()});
            true
        }else{false}
    });
    if !painted {neon_frame(x,y,w,h,true);}
}
// Keep the authored border and corners at 2x asset scale on taller menu panels.
pub(super) fn neon_menu_panel(x:f32,y:f32,w:f32,h:f32){
    neon_panel_scaled(x,y,w,h,1.0,WHITE);
}
pub(super) fn neon_compact_panel(x:f32,y:f32,w:f32,h:f32){
    neon_panel_scaled(x,y,w,h,0.65,WHITE);
}
fn neon_panel_scaled(x:f32,y:f32,w:f32,h:f32,border_scale:f32,tint:Color){
    let painted=NEON_HAND_PANEL.with(|slot|{
        let slot=slot.borrow();
        let Some(texture)=slot.as_ref() else {return false;};
        let sx=[0.,100.,texture.width()-100.,texture.width()];
        let sy=[0.,64.,texture.height()-64.,texture.height()];
        let cap_x=(50.*border_scale).min(w/2.);
        let cap_y=(32.*border_scale).min(h/2.);
        let dx=[x,x+cap_x,x+w-cap_x,x+w];
        let dy=[y,y+cap_y,y+h-cap_y,y+h];
        for row in 0..3 {for col in 0..3 {
            draw_texture_ex(texture,dx[col],dy[row],tint,DrawTextureParams{
                source:Some(Rect::new(sx[col],sy[row],sx[col+1]-sx[col],sy[row+1]-sy[row])),
                dest_size:Some(vec2(dx[col+1]-dx[col],dy[row+1]-dy[row])),..Default::default()
            });
        }}
        true
    });
    if !painted {neon_frame(x,y,w,h,true);}
}
pub(super) fn neon_empty_pile(x:f32,y:f32,w:f32,h:f32){
    neon_panel_scaled(x,y,w,h,0.36,Color::new(0.62,0.72,0.78,1.));
    // Recessed card silhouette: muted enough to distinguish it from a live card.
    let (cx,cy)=(x+w/2.,y+h*0.40);
    let ink=Color::new(0.16,0.34,0.40,1.);
    let (iw,ih,cut)=(28.,38.,5.);
    let points=[vec2(cx-iw/2.+cut,cy-ih/2.),vec2(cx+iw/2.-cut,cy-ih/2.),
        vec2(cx+iw/2.,cy-ih/2.+cut),vec2(cx+iw/2.,cy+ih/2.-cut),
        vec2(cx+iw/2.-cut,cy+ih/2.),vec2(cx-iw/2.+cut,cy+ih/2.),
        vec2(cx-iw/2.,cy+ih/2.-cut),vec2(cx-iw/2.,cy-ih/2.+cut)];
    for i in 0..8 {let a=points[i];let b=points[(i+1)%8];draw_line(a.x,a.y,b.x,b.y,1.,ink);}
}
pub(super) fn neon_pile_layers(cards:&[Card],x:f32,y:f32,w:f32,h:f32,face_down:bool){
    for layer in (1..=cards.len().saturating_sub(1).min(2)).rev(){
        let card=cards[cards.len()-1-layer];
        let index=if face_down {4}else{match card.kind {Kind::Number(_,color)=>color as usize,Kind::Wild=>5,Kind::Skip=>6}};
        let offset=layer as f32*2.5;
        neon_card_texture_tinted(index,x+offset+1.,y+offset+2.,w,h,Color::new(0.,0.,0.,0.5));
        neon_card_texture_tinted(index,x+offset,y+offset,w,h,Color::new(0.52,0.58,0.64,1.));
    }
}
pub(super) fn neon_menu_rail(y:f32){
    // Sample the straight metal rail; cyan accents remain short and segmented.
    NEON_HAND_PANEL.with(|slot|{
        if let Some(texture)=slot.borrow().as_ref(){
            draw_texture_ex(texture,-160.,y-8.,WHITE,DrawTextureParams{
                source:Some(Rect::new(400.,7.,texture.width()-800.,28.)),
                dest_size:Some(vec2(2240.,14.)),..Default::default()
            });
        }
    });
    for x in [180.,480.,1360.,1660.] {
        draw_line(x,y+7.,x+80.,y+7.,6.,Color::new(0.,0.7,0.9,0.12));
        draw_line(x,y+7.,x+80.,y+7.,2.,ACCENT);
    }
}
pub(super) fn neon_staging_slot(x:f32,y:f32,w:f32,h:f32,single:bool)->bool {
    NEON_STAGING.with(|slots|{
        if let Some(texture)=slots.borrow()[if single {0}else{1}].as_ref(){
            draw_texture_ex(texture,x,y,WHITE,DrawTextureParams{dest_size:Some(vec2(w,h)),..Default::default()});
            true
        }else{false}
    })
}
pub(super) fn neon_game_background(w:f32,h:f32){
    NEON_BACKGROUND.with(|slot|{
        if let Some(texture)=slot.borrow().as_ref(){
            // Cover the logical board uniformly, preserving the artwork angles.
            let scale=(w/texture.width()).max(h/texture.height());
            let source_w=w/scale;let source_h=h/scale;
            draw_texture_ex(texture,0.,0.,WHITE,DrawTextureParams{
                dest_size:Some(vec2(w,h)),
                source:Some(Rect::new((texture.width()-source_w)/2.,(texture.height()-source_h)/2.,source_w,source_h)),
                ..Default::default()
            });
        }
    });
}
pub(super) fn neon_metal_button(x:f32,y:f32,w:f32,h:f32,hover:bool,enabled:bool)->bool {
    NEON_METAL_BUTTON.with(|slot|{
        if let Some(texture)=slot.borrow().as_ref(){
            let source=Rect::new(43.,155.,1897.,473.);
            let cap=160.;
            let cap_w=(cap*h/source.h).min(w/2.);
            let brightness=if !enabled {0.42}
                else if hover && is_mouse_button_down(MouseButton::Left) {0.65}
                else if hover {1.0}else{0.85};
            let tint=Color::new(brightness,brightness,brightness,1.);
            for (sx,sw,dx,dw) in [
                (source.x,cap,x,cap_w),
                (source.x+cap,source.w-2.*cap,x+cap_w,(w-2.*cap_w).max(0.)),
                (source.x+source.w-cap,cap,x+w-cap_w,cap_w),
            ] {
                if dw>0. {
                    draw_texture_ex(texture,dx,y,tint,DrawTextureParams{
                        dest_size:Some(vec2(dw,h)),source:Some(Rect::new(sx,source.y,sw,source.h)),
                        ..Default::default()
                    });
                }
            }
            true
        }else{false}
    })
}
pub(super) fn neon_turn_instruction(x:f32,y:f32,w:f32,h:f32,color:Color,actionable:bool)->bool {
    let hover=actionable && inside(x,y,w,h);
    if !neon_metal_button(x,y,w,h,hover,true) {return false;}
    // A state-colored chamfered rim follows the metal silhouette, with a
    // stronger halo than ordinary controls. Only the ready state is clickable.
    let cut=(h*0.22).min(w*0.1);
    let inset=2.;
    let points=[vec2(x+cut,y+inset),vec2(x+w-cut,y+inset),
        vec2(x+w-inset,y+cut),vec2(x+w-inset,y+h-cut),
        vec2(x+w-cut,y+h-inset),vec2(x+cut,y+h-inset),
        vec2(x+inset,y+h-cut),vec2(x+inset,y+cut)];
    for (width,alpha) in [(10.,0.05),(6.,0.12),(2.,0.9)] {
        let glow=Color::new(color.r,color.g,color.b,alpha);
        for i in 0..points.len() {
            let a=points[i];let b=points[(i+1)%points.len()];
            draw_line(a.x,a.y,b.x,b.y,width,glow);
        }
    }
    true
}
pub(super) fn neon_logo(x:f32,y:f32,w:f32,h:f32)->bool {
    NEON_LOGO.with(|slot|{
        if let Some(texture)=slot.borrow().as_ref(){
            draw_texture_ex(texture,x,y,WHITE,DrawTextureParams{dest_size:Some(vec2(w,h)),..Default::default()});
            true
        }else{false}
    })
}
pub(super) fn neon_upper_banner(x:f32,y:f32,w:f32,h:f32){
    let painted=NEON_UPPER_BANNER.with(|slot|{
        if let Some(texture)=slot.borrow().as_ref(){
            // The authored canvas matches the banner's default aspect ratio.
            // Honor explicit Workshop width/height edits without recentering.
            draw_texture_ex(texture,x,y,WHITE,
                DrawTextureParams{dest_size:Some(vec2(w,h)),..Default::default()});
            true
        }else{false}
    });
    if !painted {neon_frame(x,y,w,h,true);}
}
pub(super) async fn load_neon_player_phase(){
    for (i,name) in ["player_phase_single.png","player_phase_double.png"].iter().enumerate(){
        let path=format!("{}neon_forge/ui/{}",asset_prefix(),name);
        if let Ok(texture)=load_texture(&path).await {
            texture.set_filter(FilterMode::Linear);
            NEON_PLAYER_PHASE_FRAMES.with(|slots|slots.borrow_mut()[i]=Some(texture));
        }
    }
}
pub(super) fn neon_player_phase_panel(x:f32,y:f32,w:f32,h:f32,single:bool)->bool {
    NEON_PLAYER_PHASE_FRAMES.with(|slots|{
        let slots=slots.borrow();
        if let Some(texture)=slots[if single{0}else{1}].as_ref(){
            // Player phase PNGs are tightly cropped; render the full texture.
            draw_texture_ex(texture,x,y,WHITE,DrawTextureParams{
                dest_size:Some(vec2(w,h)),
                ..Default::default()
            });
            true
        }else{false}
    })
}
pub(super) fn has_neon_cpu_panel()->bool {
    NEON_CPU_FRAMES.with(|slots|slots.borrow()[1][0].is_some())
}

// One fixed 16:10 design canvas for all window presets.
pub(super) fn game_canvas_height()->f32 {if GAME_VIEW.with(|v|v.get()){return layout::designer::canvas().map(|(_,h)|h).unwrap_or(1200.);}crate::menu_runtime::canvas().1}
thread_local! {static GAME_VIEW:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
pub(super) fn set_game_view(value:bool){GAME_VIEW.with(|v|v.set(value));}
pub(super) fn game_canvas_width()->f32 {
    if !GAME_VIEW.with(|v|v.get()){return crate::menu_runtime::canvas().0;}
    if let Some((w,_))=layout::designer::canvas(){return w;} let mode=crate::read_display_layout();
    let (w,h)=crate::read_selected_resolution().unwrap_or(if mode==crate::DisplayLayout::Mobile {(2400,1080)}else{(1600,1000)});
    1200.0*w as f32/h as f32
}
pub(super) fn game_mouse()->(f32,f32) {
    let (pw,ph)=(screen_width(),screen_height());
    let sw=game_canvas_width();let sh=game_canvas_height();
    let (mx,my)=mouse_position();
    let zoom=if GAME_VIEW.with(|v|v.get()){crate::layout_editor::zoom()}else{1.};
    let pan=if GAME_VIEW.with(|v|v.get()){crate::layout_editor::pan()}else{(0.,0.)}; let point=crate::menu_model::pointer_to_canvas((sw,sh),(pw,ph),(mx,my)); (sw/2.+pan.0+(point.0-sw/2.)/zoom,sh/2.+pan.1+(point.1-sh/2.)/zoom)
}
pub(super) fn label(text:&str,x:f32,y:f32,size:f32,color:Color){draw_text(text,x,y,size,crate::theme::themed_color(color));}
pub(super) fn button(text:&str,x:f32,y:f32,w:f32,h:f32,enabled:bool)->bool{
    let mouse=game_mouse();let hover=mouse.0>=x&&mouse.0<=x+w&&mouse.1>=y&&mouse.1<=y+h;
    if enabled {crate::theme::themed_button(x,y,w,h,hover);}
    else if active_theme()!=Theme::Futuristic || !neon_metal_button(x,y,w,h,false,false) {
        draw_rectangle(x,y,w,h,Color::new(0.18,0.22,0.28,1.));
    }
    // Keep compact gameplay controls readable with the active theme font.
    let size:u16=if h<=31.0 {16}else{19};
    let t=menu_text_width(text,size);
    menu_label(text,x+(w-t)/2.,y+h/2.+size as f32*0.36,size as f32,if enabled{WHITEISH}else{MUTED});
    enabled&&hover&&is_mouse_button_pressed(MouseButton::Left)
}

pub(super) fn neon_frame(x:f32,y:f32,w:f32,h:f32,bright:bool){
    if active_theme()!=Theme::Futuristic{return;}
    let cut=12.0_f32.min(w*0.12).min(h*0.22);
    let metal=Color::new(0.19,0.34,0.43,1.0);
    let cyan=Color::new(0.12,0.92,1.0,1.0);
    draw_rectangle(x+4.0,y+5.0,w,h,Color::new(0.0,0.0,0.0,0.45));
    draw_rectangle(x,y,w,h,Color::new(0.012,0.045,0.075,0.97));
    draw_rectangle(x+5.0,y+5.0,w-10.0,h-10.0,Color::new(0.035,0.09,0.13,0.50));
    // Broad transparent halos simulate bloom without requiring a postprocessing shader.
    for (offset,alpha,thickness) in [(6.0,0.045,9.0),(3.5,0.10,5.0),(1.0,0.22,2.5)] {
        let strength=if bright{1.0}else{0.65};
        let color=Color::new(0.0,0.79,1.0,alpha*strength);
        draw_rectangle_lines(x+offset,y+offset,w-2.0*offset,h-2.0*offset,thickness,color);
    }
    // Layered dark-metal rails and inset bevels give the frame physical depth.
    let outer_rail=Color::new(0.11,0.23,0.30,0.95);
    let inner_rail=Color::new(0.025,0.16,0.22,0.90);
    draw_rectangle_lines(x+2.0,y+2.0,w-4.0,h-4.0,3.0,outer_rail);
    draw_rectangle_lines(x+8.0,y+8.0,w-16.0,h-16.0,1.5,inner_rail);
    // Corner armor: short stacked bevels, with a lit inset seam.
    let arm=(w*0.10).min(h*0.28).min(28.0).max(7.0);
    for &(sx,sy,dx,dy) in &[(x,y,1.0,1.0),(x+w,y,-1.0,1.0),(x,y+h,1.0,-1.0),(x+w,y+h,-1.0,-1.0)] {
        let dark=Color::new(0.015,0.035,0.055,1.0);
        draw_line(sx+dx*3.0,sy+dy*(arm+6.0),sx+dx*(arm+6.0),sy+dy*3.0,7.0,dark);
        draw_line(sx+dx*4.0,sy+dy*(arm+6.0),sx+dx*(arm+6.0),sy+dy*4.0,2.0,metal);
        draw_line(sx+dx*9.0,sy+dy*(arm+4.0),sx+dx*(arm+4.0),sy+dy*9.0,1.3,Color::new(0.0,0.70,0.91,0.65));
    }
    draw_rectangle_lines(x,y,w,h,2.0,metal);
    let edge=Color::new(0.0,0.80,1.0,if bright{0.95}else{0.78});
    for &(sx,sy,dx,dy) in &[(x,y,1.0,1.0),(x+w,y,-1.0,1.0),(x,y+h,1.0,-1.0),(x+w,y+h,-1.0,-1.0)]{
        draw_line(sx+dx*2.0,sy+dy*cut,sx+dx*cut,sy+dy*2.0,3.0,edge);
        draw_line(sx+dx*4.0,sy+dy*(cut+4.0),sx+dx*(cut+4.0),sy+dy*4.0,1.2,cyan);
    }
    draw_line(x+cut,y+2.0,x+w-cut,y+2.0,2.0,edge);
    draw_line(x+cut,y+h-2.0,x+w-cut,y+h-2.0,2.0,edge);
    draw_line(x+2.0,y+cut,x+2.0,y+h-cut,2.0,edge);
    draw_line(x+w-2.0,y+cut,x+w-2.0,y+h-cut,2.0,edge);
    // Bright edge segments imitate illuminated strips on the concept-art chassis.
    let strip=(w*0.16).min(120.0).max(12.0);
    for (sy,sx) in [(y+3.0,x+cut+12.0),(y+h-3.0,x+w-cut-12.0-strip)] {
        draw_line(sx,sy,sx+strip,sy,8.0,Color::new(0.0,0.84,1.0,if bright{0.17}else{0.10}));
        draw_line(sx,sy,sx+strip,sy,2.5,cyan);
    }
}

fn clean_rgb(rgb:[u8;3])->Color {Color::from_rgba(rgb[0],rgb[1],rgb[2],255)}
fn clean_round(x:f32,y:f32,w:f32,h:f32,r:f32,color:Color){
    let r=r.min(w/2.).min(h/2.).max(0.);
    draw_rectangle(x+r,y,w-2.*r,h,color);draw_rectangle(x,y+r,w,h-2.*r,color);
    for (cx,cy) in [(x+r,y+r),(x+w-r,y+r),(x+r,y+h-r),(x+w-r,y+h-r)]{draw_circle(cx,cy,r,color);}
}
pub(super) fn clean_card_face(card:Card,x:f32,y:f32,w:f32,h:f32,corner:bool,selected:bool){
    let r=(w*0.08).min(10.);let border=(w*0.025).max(1.);
    if selected{clean_round(x-2.,y-2.,w+4.,h+4.,r+2.,ACCENT);}
    clean_round(x,y,w,h,r,clean_rgb(crate::clean_cards::CREAM));
    clean_round(x+border,y+border,w-2.*border,h-2.*border,(r-border).max(0.),clean_rgb(crate::clean_cards::face(card.kind)));
    let value=card_text(card.kind);let ink=clean_rgb(crate::clean_cards::ink(card.kind));
    let size=layout::designer::card_number_size(if corner{28.}else{(h*0.36).min(w*0.6)});
    if !corner{let m=menu_text_dimensions(&value,size as u16);menu_label(&value,x+(w-m.width)/2.,y+(h-m.height)/2.+m.offset_y,size,ink);}
    let index_size=if corner{size}else{(size*0.43).clamp(8.,24.)};
    let m=menu_text_dimensions(&value,index_size as u16);
    menu_label(&value,x+w*0.09,y+h*0.06+m.offset_y,index_size,ink);
}
pub(super) fn clean_card_back(x:f32,y:f32,w:f32,h:f32,compact:bool){
    use crate::clean_cards as style;
    let r=(w*0.08).min(10.);let border=(w*0.025).max(1.);
    clean_round(x,y,w,h,r,clean_rgb(style::CREAM));
    clean_round(x+border,y+border,w-2.*border,h-2.*border,(r-border).max(0.),clean_rgb(style::NAVY));
    let thickness=w*if compact{0.045}else{0.07};
    for mirrored in [false,true]{
        for (path,color) in [(&style::P_BOWL[..],style::CREAM),(&style::P_STEM[..],style::TEAL)]{
            for segment in path.windows(2){let a=style::mark_point(segment[0],mirrored,compact);let b=style::mark_point(segment[1],mirrored,compact);
                draw_line(x+a[0]*w,y+a[1]*h,x+b[0]*w,y+b[1]*h,thickness,clean_rgb(color));
            }
        }
    }
    let center=style::mark_point([0.5,0.5],false,compact);let cx=x+center[0]*w;let cy=y+center[1]*h;let d=w*if compact{0.035}else{0.055};
    draw_triangle(vec2(cx,cy-d),vec2(cx+d,cy),vec2(cx,cy+d),clean_rgb(style::TEAL));
    draw_triangle(vec2(cx,cy-d),vec2(cx-d,cy),vec2(cx,cy+d),clean_rgb(style::TEAL));
}
pub(super) fn selected_card_lift(selected:bool)->f32 {
    if !selected {0.}else if layout::designer::mobile() || active_theme()!=Theme::Futuristic{15.}else{0.}
}
pub(super) fn card_ui(card:Card,x:f32,y:f32,w:f32,h:f32,selected:bool)->bool {
    let yy=y-selected_card_lift(selected);let c=color_for(card.kind);
    if active_theme()==Theme::Modern {clean_card_face(card,x,yy,w,h,false,selected);}
    else if active_theme()==Theme::Futuristic && matches!(card.kind,Kind::Wild|Kind::Skip) && neon_card_texture(if matches!(card.kind,Kind::Wild){5}else{6},x,yy,w,h) {
        // Special cards share the Neon Forge texture family until dedicated art is available.
        let symbol=card_text(card.kind);
        let size=layout::designer::card_number_size(if w<45.0{22.}else{35.}) as u16;
        let metrics=menu_text_dimensions(&symbol,size);
        let tx=x+(w-metrics.width)/2.0;
        // Font ascent differs between W and S: center the actual glyph bounds.
        let ty=yy+(h-metrics.height)/2.0+metrics.offset_y;
        menu_label(&symbol,tx+1.5,ty+2.0,size as f32,BLACK);
        menu_label(&symbol,tx,ty,size as f32,WHITEISH);
        if selected && active_theme()!=Theme::Futuristic {draw_rectangle_lines(x-1.0,yy-1.0,w+2.0,h+2.0,2.0,ACCENT);}
    } else if active_theme()==Theme::Futuristic && matches!(card.kind,Kind::Number(_, _)) && neon_card_texture(match card.kind {Kind::Number(_,color)=>color as usize,_=>0},x,yy,w,h) {
        let t=card_text(card.kind);
        let size=layout::designer::card_number_size(if w<45.0{22.}else{35.}) as u16;
        let tw=menu_text_width(&t,size);
        let tx=x+(w-tw)/2.0;
        let ty=yy+h*0.60;
        menu_label(&t,tx+1.5,ty+2.0,size as f32,BLACK);
        menu_label(&t,tx,ty,size as f32,WHITEISH);
        if selected && active_theme()!=Theme::Futuristic {draw_rectangle_lines(x-1.0,yy-1.0,w+2.0,h+2.0,2.0,ACCENT);}
    } else if active_theme()==Theme::Futuristic && matches!(card.kind,Kind::Number(_, _)) {
        // Neon Forge / Dark Metallic: layered machined chassis and colored light core.
        // All detailing scales with the actual card dimensions.
        let sx=w/75.0;
        let sy=h/105.0;
        let px=|v:f32|x+v*sx;
        let py=|v:f32|yy+v*sy;
        let ink=Color::new(0.015,0.032,0.060,1.0);
        let steel=Color::new(0.12,0.19,0.26,1.0);
        let rim=Color::new(0.32,0.43,0.52,1.0);
        let light=match card.kind {
            Kind::Number(_,0)=>Color::new(1.0,0.23,0.27,1.0),
            Kind::Number(_,1)=>Color::new(0.16,0.68,1.0,1.0),
            Kind::Number(_,2)=>Color::new(1.0,0.82,0.24,1.0),
            Kind::Number(_,3)=>Color::new(0.25,1.0,0.54,1.0),
            _=>c,
        };
        draw_rectangle(x+4.0,yy+6.0,w,h,Color::new(0.0,0.0,0.0,0.42));
        draw_rectangle(x,yy,w,h,ink);
        draw_rectangle_lines(x,yy,w,h,2.0,steel);
        draw_rectangle(px(3.0),py(3.0),69.0*sx,99.0*sy,Color::new(0.045,0.080,0.125,1.0));
        draw_rectangle_lines(px(3.0),py(3.0),69.0*sx,99.0*sy,1.0,rim);
        // Inset beveled plate and layered light channels.
        draw_rectangle(px(8.0),py(12.0),59.0*sx,81.0*sy,Color::new(0.020,0.046,0.082,1.0));
        draw_rectangle_lines(px(8.0),py(12.0),59.0*sx,81.0*sy,1.0,Color::new(light.r,light.g,light.b,0.48));
        draw_rectangle(px(12.0),py(18.0),51.0*sx,69.0*sy,Color::new(light.r*0.09,light.g*0.09,light.b*0.09,1.0));
        draw_rectangle_lines(px(12.0),py(18.0),51.0*sx,69.0*sy,1.4,light);
        draw_rectangle(px(15.0),py(21.0),45.0*sx,63.0*sy,Color::new(0.015,0.035,0.065,1.0));
        // Fine circuit traces, corner braces and subtle colored illumination.
        let faint=Color::new(light.r,light.g,light.b,0.32);
        draw_line(px(16.0),py(27.0),px(28.0),py(27.0),1.0,faint);
        draw_line(px(47.0),py(78.0),px(59.0),py(78.0),1.0,faint);
        for (cx,cy,dx,dy) in [(7.0,7.0,1.0,1.0),(68.0,7.0,-1.0,1.0),(7.0,98.0,1.0,-1.0),(68.0,98.0,-1.0,-1.0)] {
            draw_line(px(cx),py(cy),px(cx+dx*10.0),py(cy),2.0,light);
            draw_line(px(cx),py(cy),px(cx),py(cy+dy*10.0),2.0,light);
        }
        draw_rectangle(px(29.0),py(5.0),17.0*sx,2.0*sy,light);
        draw_rectangle(px(29.0),py(98.0),17.0*sx,2.0*sy,light);
        // Central glyph has a dark offset shadow and a crisp white foreground.
        let t=card_text(card.kind);
        let size=layout::designer::card_number_size(if w<45.0{22.}else{35.}) as u16;
        let tw=menu_text_width(&t,size);
        let tx=x+(w-tw)/2.0;
        let ty=yy+h*0.63;
        menu_label(&t,tx+1.5,ty+2.0,size as f32,Color::new(0.0,0.0,0.0,0.95));
        menu_label(&t,tx,ty,size as f32,WHITEISH);
        if selected && active_theme()!=Theme::Futuristic {
            draw_rectangle_lines(x-1.0,yy-1.0,w+2.0,h+2.0,2.0,ACCENT);
        }
    } else {
        draw_rectangle(x+2.,yy+3.,w,h,Color::new(0.,0.,0.,0.3));draw_rectangle(x,yy,w,h,WHITEISH);
        draw_rectangle(x+5.,yy+5.,w-10.,h-10.,c);
        let t=card_text(card.kind);let size=layout::designer::card_number_size(if w<45.0{23.0}else{32.0});let dims=measure_text(&t,None,size as u16,1.0);
        label(&t,x+(w-dims.width)/2.,yy+h/2.+dims.height/2.,size,if matches!(card.kind,Kind::Number(_,2)){BG}else{WHITE});
    }
    if selected && layout::designer::mobile() && active_theme()!=Theme::Modern{draw_rectangle_lines(x-2.,yy-2.,w+4.,h+4.,4.,ACCENT);}
    let m=game_mouse();!board_input_blocked()&&!layout_editor::pointer_over_tools()&&m.0>=x&&m.0<=x+w&&m.1>=yy&&m.1<=yy+h&&is_mouse_button_pressed(MouseButton::Left)
}
pub(super) fn column_hand_width(count:usize,card_w:f32)->f32 {
    if count<=1 {0.0}else{(count-1) as f32*24.0_f32.min((450.0-card_w)/(count-1) as f32)}
}
pub(super) fn neon_draw_back(x:f32,y:f32,w:f32,h:f32)->bool {
    active_theme()==Theme::Futuristic && neon_card_texture(4,x,y,w,h)
}
pub(super) fn cpu_card_back(x:f32,y:f32,w:f32,h:f32) {
    if active_theme()==Theme::Modern{clean_card_back(x,y,w,h,false);return;}
    if active_theme()==Theme::Futuristic && neon_card_texture(4,x,y,w,h){return;}
    draw_rectangle(x+2.0,y+3.0,w,h,Color::new(0.0,0.0,0.0,0.28));
    draw_rectangle(x,y,w,h,WHITEISH);
    draw_rectangle(x+3.0,y+3.0,w-6.0,h-6.0,Color::new(0.12,0.39,0.48,1.0));
    draw_rectangle_lines(x+6.0,y+6.0,w-12.0,h-12.0,1.5,ACCENT);
    draw_line(x+8.0,y+8.0,x+w-8.0,y+h-8.0,1.0,ACCENT);
    draw_line(x+w-8.0,y+8.0,x+8.0,y+h-8.0,1.0,ACCENT);
}

// Explicit persistent IDs keep layout edits independent of changing text.
pub(super) fn editable_label(key:&str,text:&str,x:f32,y:f32,size:f32,color:Color){
    let key=layout::scene_key(key);
    editable_label_id(&key,text,x,y,size,color);
}
// Use explicit IDs when a state change can alter which group is drawn first.
pub(super) fn editable_label_id(key:&str,text:&str,x:f32,y:f32,size:f32,color:Color){ if layout::designer::runtime_label(key,text,size,color){return;}
    let r=layout::rect(key,layout::Rect{x,y:y-size,w:menu_text_width(text,size as u16).max(20.),h:size});
    menu_label(text,r.x,r.y+r.h,r.h,color);
}
pub(super) fn editable_heading(key:&str,text:&str,x:f32,y:f32,size:f32){
    let key=layout::scene_key(key);
    let r=layout::rect(&key,layout::Rect{x,y:y-size,w:menu_text_width(text,size as u16).max(20.),h:size});
    menu_heading(text,r.x,r.y+r.h,r.h);
}
pub(super) fn editable_button(key:&str,text:&str,x:f32,y:f32,w:f32,h:f32,enabled:bool)->bool {
    let key=layout::scene_key(key);
    let r=layout::rect(&key,layout::Rect{x,y,w,h});
    let hover=inside(r.x,r.y,r.w,r.h);
    if enabled {crate::theme::themed_button(r.x,r.y,r.w,r.h,hover);}
    else if active_theme()!=Theme::Futuristic || !neon_metal_button(r.x,r.y,r.w,r.h,false,false){draw_rectangle(r.x,r.y,r.w,r.h,Color::new(0.18,0.22,0.28,1.));}
    let size=if r.h<=31. {16.}else{19.};
    editable_label(&format!("{key}_text"),text,r.x+(r.w-menu_text_width(text,size as u16))/2.,r.y+r.h/2.+size*0.36,size,if enabled {WHITEISH}else{MUTED});
    enabled && hover && is_mouse_button_pressed(MouseButton::Left) && !layout_editor::editing()
}
