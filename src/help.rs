use crate::*;

pub(super) fn help_page(rules:bool, scroll:&mut f32) {
    let offset=(game_canvas_height()-860.0)/2.0-185.0;
    let heading=if rules {"GAME RULES"}else{"HOW TO PLAY"};
    // Saved menu bounds place the heading and scrolling content together.
    menu_runtime::label("title",heading,menu_model::Bounds{x:365.,y:108.,w:1190.,h:62.},45.,ACCENT,true);
    let panel=menu_runtime::bounds("content",menu_model::Bounds{x:365.,y:185.+offset,w:1190.,h:860.});
    let (text_dx,text_dy)=menu_runtime::offsets("content");
    let content_x=panel.x+50.+text_dx;let scrollbar_x=panel.x+panel.w-80.;
    panel_decoration(panel.x,panel.y,panel.w,panel.h);
    let content_top=panel.y+60.;let content_bottom=panel.y+panel.h-70.;
    let viewport_height=(content_bottom-content_top).max(1.);
    let body_font=menu_runtime::font_size("content",23.).clamp(8.,120.) as u16;
    let mut rows=crate::menu_model::help_sections(rules);
    if let Some(custom)=menu_runtime::custom_text("content"){rows=custom.lines().map(|line|(false,line.to_owned())).collect();}
    // Use the same theme-specific menu font as the rest of PhaseUP.
    // Larger type and wider line spacing make these pages easier to scan.
    let text_scale=body_font as f32/23.;
    let mut laid_out:Vec<(bool,String,f32)>=Vec::new();
    let mut height=0.0;
    for (is_heading,line) in rows {
        let font=if is_heading {body_font.saturating_add(2)}else{body_font};
        let max_width=(panel.w-155.).max(20.);
        let mut current=String::new();
        for word in line.split_whitespace() {
            let candidate=if current.is_empty(){word.to_string()}else{format!("{} {}",current,word)};
            if !current.is_empty() && menu_text_width(&candidate,font)>max_width {
                laid_out.push((is_heading,current,height));
                height+=(if is_heading {39.0}else{36.0})*text_scale;
                current=word.to_string();
            } else {current=candidate;}
        }
        if !current.is_empty() {
            laid_out.push((is_heading,current,height));
            height+=(if is_heading {37.0}else{31.0})*text_scale;
        }
        height+=(if is_heading {13.0}else{21.0})*text_scale;
    }
    let content_height=height+28.0;
    let max_scroll=(content_height-viewport_height).max(0.0);
    let (_,wheel)=mouse_wheel();
    *scroll=(*scroll-wheel*50.0).clamp(0.0,max_scroll);
    if is_key_down(KeyCode::Down){*scroll=(*scroll+7.0).min(max_scroll);}
    if is_key_down(KeyCode::Up){*scroll=(*scroll-7.0).max(0.0);}
    for (is_heading,line,y) in laid_out {
        let screen_y=content_top+28.0*text_scale+y-*scroll+text_dy;
        let font=if is_heading {body_font.saturating_add(2)}else{body_font};
        let metrics=menu_text_dimensions(&line,font);
        let glyph_top=screen_y-metrics.offset_y;
        if glyph_top>=content_top && glyph_top+metrics.height<=content_bottom {
            menu_label(&line,content_x,screen_y,font as f32,if is_heading {ACCENT}else{WHITEISH});
        }
    }
    menu_runtime::label("scroll_hint","Mouse wheel / Up-Down arrows to scroll",menu_model::Bounds{x:content_x,y:panel.y+panel.h-55.,w:panel.w-100.,h:32.},17.,MUTED,false);
    if max_scroll>0.0 {
        let track_y=content_top;let track_h=viewport_height;
        let thumb_h=(viewport_height/content_height*track_h).clamp(50.0,track_h);
        let thumb_y=track_y+(*scroll/max_scroll)*(track_h-thumb_h);
        draw_rectangle(scrollbar_x,track_y,9.0,track_h,BG);
        draw_rectangle(scrollbar_x,thumb_y,9.0,thumb_h,ACCENT);
    }
}
