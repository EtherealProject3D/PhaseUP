use crate::*;

pub(super) fn help_page(rules:bool, scroll:&mut f32) {
    let heading=if rules {"GAME RULES"}else{"HOW TO PLAY"};
    let tw=measure_text(heading,None,45,1.0).width;
    label(heading,800.0-tw/2.0,120.0,45.0,ACCENT);
    draw_rectangle(205.0,157.0,1190.0,888.0,PANEL);
    draw_rectangle_lines(205.0,157.0,1190.0,888.0,2.0,ACCENT);

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
        rows.push((true,"NEED THE DETAILS?".into()));
        rows.push((false,"Open GAME RULES from the main menu for the ten phase requirements, special cards, scoring, and win conditions.".into()));
    }

    let mut laid_out:Vec<(bool,String,f32)>=Vec::new();
    let mut height=0.0;
    for (is_heading,line) in rows {
        let font=if is_heading {25.0}else{21.0};
        let max_width=1060.0;
        let mut current=String::new();
        for word in line.split_whitespace() {
            let candidate=if current.is_empty(){word.to_string()}else{format!("{} {}",current,word)};
            if !current.is_empty() && measure_text(&candidate,None,font as u16,1.0).width>max_width {
                laid_out.push((is_heading,current,height));
                height+=if is_heading {37.0}else{31.0};
                current=word.to_string();
            } else {current=candidate;}
        }
        if !current.is_empty() {
            laid_out.push((is_heading,current,height));
            height+=if is_heading {37.0}else{31.0};
        }
        height+=if is_heading {10.0}else{19.0};
    }
    let viewport_height=795.0;
    let max_scroll=(height-viewport_height+25.0).max(0.0);
    let (_,wheel)=mouse_wheel();
    *scroll=(*scroll-wheel*50.0).clamp(0.0,max_scroll);
    if is_key_down(KeyCode::Down){*scroll=(*scroll+7.0).min(max_scroll);}
    if is_key_down(KeyCode::Up){*scroll=(*scroll-7.0).max(0.0);}
    for (is_heading,line,y) in laid_out {
        let screen_y=200.0+y-*scroll;
        if screen_y>=187.0 && screen_y<=997.0 {
            label(&line,255.0,screen_y,if is_heading {25.0}else{21.0},if is_heading {ACCENT}else{WHITEISH});
        }
    }
    label("Mouse wheel / Up-Down arrows to scroll",255.0,1030.0,17.0,MUTED);
    if max_scroll>0.0 {
        let track_y=195.0;let track_h=790.0;
        let thumb_h=(viewport_height/(height+25.0)*track_h).clamp(50.0,track_h);
        let thumb_y=track_y+(*scroll/max_scroll)*(track_h-thumb_h);
        draw_rectangle(1350.0,track_y,9.0,track_h,BG);
        draw_rectangle(1350.0,thumb_y,9.0,thumb_h,ACCENT);
    }
}
