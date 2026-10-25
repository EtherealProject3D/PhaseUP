use crate::*;

pub(super) fn help_page(rules:bool, scroll:&mut f32) {
    let offset=(game_canvas_height()-860.0)/2.0-185.0;
    let heading=if rules {"GAME RULES"}else{"HOW TO PLAY"};
    // Center the heading on the 1920-wide canvas, below the background
    // accent stripe at y=100 and above the help content panel.
    let tw=menu_text_width(heading,45);
    menu_heading(heading,960.0-tw/2.0,153.0+offset,45.0);
    // Center the 1190px panel on the 1920px logical canvas.
    let panel_x=(1920.0-1190.0)/2.0;
    let content_x=panel_x+50.0;
    let scrollbar_x=panel_x+1110.0;
    panel_decoration(panel_x,185.0+offset,1190.0,860.0);
    let content_top=245.0+offset;
    let content_bottom=975.0+offset;
    let viewport_height=content_bottom-content_top;

    let mut rows:Vec<(bool,String)>=Vec::new();
    if rules {
        rows.push((true,"THE GOAL".into()));
        rows.push((false,"Complete all ten phases in order. Each round, players work on their own current phase. A player who completes a phase advances to the next one at the end of that round.".into()));
        rows.push((true,"THE TEN PHASES".into()));
        for i in 0..PHASES.len() {rows.push((false,format!("{}: {}",i+1,phase_name(i))));}
        rows.push((true,"CARD COMBINATIONS".into()));
        rows.push((false,"Set: cards with the same number, regardless of color. Run: consecutive numbers, regardless of color. Color group: cards of the same color, regardless of number.".into()));
        rows.push((true,"WILD CARDS".into()));
        rows.push((false,"Wilds can stand in for a number or color in a phase or hit. Every group must contain at least one ordinary numbered card. Wilds cannot replace Skip cards.".into()));
        rows.push((true,"SKIP CARDS".into()));
        rows.push((false,"A Skip card is discarded to make another player miss a turn. It cannot be used in a phase or hit, and cannot be drawn from the discard pile.".into()));
        rows.push((true,"HITTING".into()));
        rows.push((false,"After laying your own phase, you can add cards to any completed phase if the resulting set, run, or color group is valid.".into()));
        rows.push((true,"ENDING A ROUND".into()));
        rows.push((false,"A round ends when someone gets rid of all their cards. Players who laid their phase advance one phase; others retry the same phase next round.".into()));
        rows.push((true,"SCORING".into()));
        rows.push((false,"Cards remaining in your hand count against you: numbers 1-9 = 5 points; 10-12 = 10 points; Skip = 15 points; Wild = 25 points.".into()));
        rows.push((true,"WINNING".into()));
        rows.push((false,"The game ends when a player completes phase 10 and the round finishes.".into()));
    } else {
        rows.push((true,"1. DRAW A CARD".into()));
        rows.push((false,"Start your turn by clicking the draw deck or the top discard card. You cannot take a Skip from the discard pile.".into()));
        rows.push((true,"2. BUILD YOUR PHASE".into()));
        rows.push((false,"Check the phase shown above your hand. Drag cards into the staging groups. A set uses matching numbers, a run uses consecutive numbers, and a color group uses one color.".into()));
        rows.push((true,"3. LAY YOUR PHASE".into()));
        rows.push((false,"When your staged cards satisfy the phase, click LAY PHASE. Your cards move to your table area. You can only lay your phase once per round.".into()));
        rows.push((true,"4. HIT OTHER PHASES".into()));
        rows.push((false,"Once your phase is laid, drag a card onto your phase or another player's completed phase to extend a valid group. For example, add a 7 to a run ending in 6.".into()));
        rows.push((true,"5. DISCARD TO END YOUR TURN".into()));
        rows.push((false,"Click a card in your hand to select it, then discard it. Only one hand card is selected at a time. You can also use a Skip to make an opponent miss a turn.".into()));
        rows.push((true,"6. FINISH THE ROUND".into()));
        rows.push((false,"Be the first to get rid of all your cards, or keep working on your phase until another player goes out. Completed phases advance next round.".into()));
        rows.push((true,"TIP: SORT YOUR HAND".into()));
        rows.push((false,"Use RUNS, COLORS, or SETS below your hand to change how cards are arranged. Sorting does not change which cards you own.".into()));
        rows.push((true,"WINDOW SHORTCUTS".into()));
        rows.push((false,"Right-click the PHASEUP logo during a game, or press Ctrl+0, to reset the game window to its default size. You can also reset the window size from Settings.".into()));
        rows.push((true,"NEED THE DETAILS?".into()));
        rows.push((false,"Open GAME RULES from the main menu for the ten phase requirements, special cards, scoring, and win conditions.".into()));
    }

    // Use the same theme-specific menu font as the rest of PhaseUP.
    // Larger type and wider line spacing make these pages easier to scan.
    let mut laid_out:Vec<(bool,String,f32)>=Vec::new();
    let mut height=0.0;
    for (is_heading,line) in rows {
        let font=if is_heading {25_u16}else{23_u16};
        let max_width=1035.0;
        let mut current=String::new();
        for word in line.split_whitespace() {
            let candidate=if current.is_empty(){word.to_string()}else{format!("{} {}",current,word)};
            if !current.is_empty() && menu_text_width(&candidate,font)>max_width {
                laid_out.push((is_heading,current,height));
                height+=if is_heading {39.0}else{36.0};
                current=word.to_string();
            } else {current=candidate;}
        }
        if !current.is_empty() {
            laid_out.push((is_heading,current,height));
            height+=if is_heading {37.0}else{31.0};
        }
        height+=if is_heading {13.0}else{21.0};
    }
    let content_height=height+28.0;
    let max_scroll=(content_height-viewport_height).max(0.0);
    let (_,wheel)=mouse_wheel();
    *scroll=(*scroll-wheel*50.0).clamp(0.0,max_scroll);
    if is_key_down(KeyCode::Down){*scroll=(*scroll+7.0).min(max_scroll);}
    if is_key_down(KeyCode::Up){*scroll=(*scroll-7.0).max(0.0);}
    for (is_heading,line,y) in laid_out {
        let screen_y=content_top+28.0+y-*scroll;
        let font=if is_heading {25}else{23};
        let metrics=menu_text_dimensions(&line,font);
        let glyph_top=screen_y-metrics.offset_y;
        if glyph_top>=content_top && glyph_top+metrics.height<=content_bottom {
            menu_label(&line,content_x,screen_y,font as f32,if is_heading {ACCENT}else{WHITEISH});
        }
    }
    menu_label("Mouse wheel / Up-Down arrows to scroll",content_x,1010.0+offset,17.0,MUTED);
    if max_scroll>0.0 {
        let track_y=content_top;let track_h=viewport_height;
        let thumb_h=(viewport_height/content_height*track_h).clamp(50.0,track_h);
        let thumb_y=track_y+(*scroll/max_scroll)*(track_h-thumb_h);
        draw_rectangle(scrollbar_x,track_y,9.0,track_h,BG);
        draw_rectangle(scrollbar_x,thumb_y,9.0,thumb_h,ACCENT);
    }
}
