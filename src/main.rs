// Launch as a GUI application on Windows, without a separate console window.
#![cfg_attr(all(target_os = "windows", not(test)), windows_subsystem = "windows")]

use macroquad::prelude::*;

const BG: Color = Color::new(0.055, 0.09, 0.15, 1.);
const PANEL: Color = Color::new(0.10, 0.16, 0.24, 1.);
const ACCENT: Color = Color::new(0.25, 0.81, 0.72, 1.);
const WHITEISH: Color = Color::new(0.92, 0.96, 1., 1.);
const MUTED: Color = Color::new(0.60, 0.68, 0.78, 1.);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind { Number(u8, u8), Wild, Skip }
#[derive(Clone, Copy)]
struct Card { id: u32, kind: Kind }
struct Player { hand: Vec<Card>, phase: usize, laid: bool, score: u32, table: Vec<Vec<Card>> }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stage { Draw, Play, Cpu, RoundOver, GameOver }
#[derive(Clone, Copy, PartialEq, Eq)]
enum SortMode { Runs, Colors, Sets }
fn inside(x:f32,y:f32,w:f32,h:f32)->bool {
    let (mx,my)=game_mouse();mx>=x&&mx<=x+w&&my>=y&&my<=y+h
}
struct Game { players: Vec<Player>, deck: Vec<Card>, discard: Vec<Card>, turn: usize, stage: Stage, selected: Vec<u32>, message: String, round: u32, sort: SortMode, staged: [Vec<u32>;2], drag: Option<(u32,f32,f32)>, pile_drag: Option<(bool,f32,f32)>, drawn_card: Option<u32>, cpu_names: [String;3], cpu_roster: [CpuIdentity;3], move_log: Vec<String>, turn_count:u32, staged_drag: Option<(u32,f32,f32)>, cpu_step:u8, cpu_skip:Option<usize>, cpu_next_at:f64, cpu_speed:u8, cpu_draw_flight:Option<CpuDrawFlight>, cpu_phase_flight:Option<CpuPhaseFlight>, cpu_hit_flight:Option<CpuHitFlight>, human_draw_flight:Option<HumanDrawFlight>, human_phase_flight:Option<HumanPhaseFlight>, cpu_discard_flight:Option<CpuDiscardFlight>, human_discard_flight:Option<HumanDiscardFlight>, hover_lifts:Vec<(u32,f32,f32,f64)> }

mod game_rules;
mod phases;
use phases::*;
mod cards;
use cards::*;
mod ai;
use ai::*;
mod roster;
use roster::*;
mod cpu;
mod animations;
use animations::*;
mod ui;
use ui::*;
mod layout;
mod layout_editor;
mod theme;
use theme::*;
mod help;
use help::*;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod simulation;
#[cfg(test)]
fn phase_animation_start_time()->f64 {0.0}
#[cfg(not(test))]
fn phase_animation_start_time()->f64 {get_time()}

impl Game {
    fn new()->Self {
        let players=(0..4).map(|_|Player{hand:vec![],phase:0,laid:false,score:0,table:vec![]}).collect();
        let cpu_roster=select_opponents(read_cpu_difficulty());
        let cpu_names=cpu_roster.map(|cpu|cpu.name.to_string());
        let mut g=Self{players,deck:vec![],discard:vec![],turn:0,stage:Stage::Draw,selected:vec![],message:String::new(),round:0,sort:SortMode::Runs,staged:[vec![],vec![]],drag:None,pile_drag:None,drawn_card:None,cpu_names,cpu_roster,move_log:Vec::new(),turn_count:1,staged_drag:None,cpu_step:0,cpu_skip:None,cpu_next_at:0.0,cpu_speed:2,cpu_draw_flight:None,cpu_phase_flight:None,cpu_hit_flight:None,human_draw_flight:None,human_phase_flight:None,cpu_discard_flight:None,human_discard_flight:None,hover_lifts:Vec::new()};g.deal();g
    }
    fn cpu_name(&self,i:usize)->String {if i==0 {"You".to_string()}else{self.cpu_names[i-1].clone()}}
    fn log_move(&mut self,event:String){self.move_log.push(event);if self.move_log.len()>8{self.move_log.remove(0);}}
    fn stage_card(&mut self,id:u32,group:usize) {
        if self.stage!=Stage::Play||self.turn!=0{return;}
        if group>=PHASES[self.players[0].phase].len(){return;}
        if !self.players[0].hand.iter().any(|c|c.id==id){return;}
        for bucket in &mut self.staged {bucket.retain(|v|*v!=id);}
        self.staged[group].push(id);
        // Auto-sort newly staged run cards, but leave explicit drag reordering alone.
        if requirement(self.players[0].phase,group)=="run" && !self.staged[group].iter().any(|card_id|self.players[0].hand.iter().any(|c|c.id==*card_id && c.kind==Kind::Wild)) {
            let hand=&self.players[0].hand;
            self.staged[group].sort_by_key(|card_id| {
                hand.iter().find(|c|c.id==*card_id).map(|c|match c.kind {
                    Kind::Number(n,_) => (0u8,n),
                    Kind::Wild => (1u8,0),
                    Kind::Skip => (2u8,0),
                }).unwrap_or((3u8,0))
            });
        }
        self.selected.retain(|v|*v!=id);
    }
    fn stage_card_at(&mut self,id:u32,group:usize,index:usize) {
        self.stage_card(id,group);
        // Explicit drops choose a position; only automatic staging sorts Wilds last.
        if self.staged.iter().any(|bucket|bucket.contains(&id)) {
            self.move_staged_card(id,group,index);
        }
    }
    fn move_staged_card(&mut self,id:u32,group:usize,index:usize) {
        if self.stage!=Stage::Play||group>=self.staged.len(){return;}
        if !self.staged.iter().any(|bucket|bucket.contains(&id)){return;}
        for bucket in &mut self.staged {bucket.retain(|v|*v!=id);}
        let at=index.min(self.staged[group].len());self.staged[group].insert(at,id);
    }
    fn unstage_card(&mut self,id:u32) {
        for bucket in &mut self.staged {bucket.retain(|v|*v!=id);}
    }
    fn stage_selected(&mut self,group:usize) {
        let ids=self.selected.clone();
        for id in ids {self.stage_card(id,group);}
    }
    fn draw(&mut self,discard:bool,animate:bool) {
        if self.stage!=Stage::Draw||self.turn!=0{return;}
        if discard && self.discard.last().is_some_and(|c|c.kind==Kind::Skip) {self.message="Skip cards cannot be taken from discard".into();return;}
        let c=if discard {self.discard.pop()} else {self.draw_deck()};
        if let Some(c)=c {self.drawn_card=Some(c.id);self.players[0].hand.push(c);self.human_draw_flight=if animate {Some(HumanDrawFlight{card_id:c.id,from_discard:discard,started:get_time()})}else{None};self.stage=Stage::Play;self.log_move(format!("You drew from {}",if discard {"the Discard pile"}else{"the Deck"}));self.message="Select cards to lay your phase, or select one to discard".into();}
    }
    fn lay(&mut self) {
        if self.stage!=Stage::Play||self.turn!=0||self.players[0].laid{return;}
        let cards:Vec<Card>=self.players[0].hand.iter().copied().filter(|c|self.selected.contains(&c.id)).collect();
        let phase=self.players[0].phase;
        // Preserve the player's explicit staging groups when they form a valid phase.
        let staged_groups=if self.staged.iter().flatten().count()==cards.len()
            && self.staged.iter().flatten().all(|id|self.selected.contains(id))
            && (0..PHASES[phase].len()).all(|i|{
                self.staged[i].len()>=PHASES[phase][i] &&
                valid_group(&self.staged[i].iter().filter_map(|id|cards.iter().find(|c|c.id==*id).copied()).collect::<Vec<_>>(),requirement(phase,i))
            }){
            Some((0..PHASES[phase].len()).map(|i|self.staged[i].iter().filter_map(|id|cards.iter().find(|c|c.id==*id).copied()).collect::<Vec<_>>()).collect::<Vec<_>>())
        }else{None};
        if let Some(mut groups)=staged_groups.or_else(||partition_phase_flexible(&cards,phase)) {
            order_table_groups(&mut groups,phase);
            // Capture the original staging positions before clearing the staging area.
            let stage_w=if PHASES[phase].len()==1 {800.0}else{680.0};
            let stage_left=(1920.0-(PHASES[phase].len() as f32*stage_w
                +(PHASES[phase].len().saturating_sub(1)) as f32*6.0))/2.0;
            let mut flights=Vec::new();
            for (group_idx,group_cards) in groups.iter().enumerate() {
                for (slot,card) in group_cards.iter().copied().enumerate() {
                    let staged=self.staged.iter().enumerate().find_map(|(g,ids)|
                        ids.iter().position(|id|*id==card.id).map(|idx|(g,idx)));
                    let (from_x,from_y)=if let Some((g,idx))=staged {
                        (stage_left+g as f32*(stage_w+6.0)+10.0+idx as f32*57.0,1200.0-347.0+29.0)
                    }else{
                        (960.0,1200.0-166.0)
                    };
                    flights.push((card,group_idx,slot,from_x,from_y));
                }
            }
            self.human_phase_flight=Some(HumanPhaseFlight{
                cards:flights,started:phase_animation_start_time(),per_card:0.18
            });
            Self::remove_ids(&mut self.players[0].hand,&self.selected);
            self.players[0].table=groups;self.players[0].laid=true;self.selected.clear();self.staged=[vec![],vec![]];
            self.log_move(format!("You laid phase {}",phase+1));
            self.message="Phase completed! Hit any laid phase or discard to finish.".into();
        }else{self.message=format!("Selected cards do not match phase {} ({})",phase+1,phase_name(phase));}
    }
    fn hit(&mut self, target:usize,group:usize) {
        if self.stage!=Stage::Play||self.turn!=0 {self.message="Draw a card before playing onto a phase.".into();return;}
        if !self.players[0].laid {self.message="Complete your own phase before adding cards to the table.".into();return;}
        if self.selected.len()!=1 {self.message="Select exactly one card, then click a table group to add it.".into();return;}
        let Some(card)=self.players[0].hand.iter().find(|c|c.id==self.selected[0]).copied() else{return;};
        if !self.players[target].laid||group>=self.players[target].table.len(){return;}
        let phase=self.players[target].phase;
        if can_hit(&self.players[target].table[group],card,phase,group) {
            Self::remove_ids(&mut self.players[0].hand,&[card.id]);self.players[target].table[group].push(card);if requirement(phase,group)=="run"{order_run(&mut self.players[target].table[group]);}self.unstage_card(card.id);self.selected.clear();
            self.log_move(if target==0 {format!("You hit with {} {}",card_article(card.kind),card_text(card.kind))}else{format!("You hit {} with {} {}",self.cpu_name(target),card_article(card.kind),card_text(card.kind))});
            self.message=format!("Added {} to {}'s {}!",card_text(card.kind),if target==0 {"your".to_string()}else{format!("CPU {}",target)},requirement(phase,group));
            if self.players[0].hand.is_empty(){self.end_round(0);}
        }else{self.message="That card does not fit this group. For runs, add a number at either end.".into();}
    }
    fn discard_selected(&mut self) {
        if self.stage!=Stage::Play||self.turn!=0||self.selected.len()!=1{return;}
        let id=self.selected[0];
        // Capture the card's actual sorted hand slot before removing it.
        let mut sorted:Vec<Card>=self.players[0].hand.iter().copied()
            .filter(|c|!self.staged.iter().any(|bucket|bucket.contains(&c.id))).collect();
        sorted.sort_by_key(|c|sort_key(*c,self.sort));
        let n=sorted.len();
        let width=((1920.0-48.0)/(n.max(1) as f32)).min(75.0);
        let spacing=if n>1 {((1600.0-48.0-width)/(n-1) as f32).min(81.0)}else{0.0};
        let left=(1920.0-(spacing*(n.saturating_sub(1) as f32)+width))/2.0;
        let index=sorted.iter().position(|c|c.id==id).unwrap_or(n.saturating_sub(1));
        let from_x=left+index as f32*spacing;
        let from_y=1200.0-166.0;
        let card=Self::remove_ids(&mut self.players[0].hand,&[id]);
        if card.is_empty(){return;}
        let c=card[0];
        self.human_draw_flight=None;
        self.human_discard_flight=Some(HumanDiscardFlight{card:c,from_x,from_y,started:get_time()});
        self.discard.push(c);self.unstage_card(c.id);self.selected.clear();
        if c.kind!=Kind::Skip {self.log_move(format!("You discarded {} {}",card_article(c.kind),card_text(c.kind)));}
        if self.players[0].hand.is_empty(){self.end_round(0);return;}
        if c.kind==Kind::Skip {let target=(1..4).find(|i|self.players[*i].hand.len()>0).unwrap_or(1);self.message=format!("Skip played on CPU {}",target);self.log_move(if target==0 {"You were skipped".to_string()}else{format!("{} was skipped",self.cpu_name(target))});self.ai_cycle(Some(target));}
        else {self.ai_cycle(None);}
    }
    fn ai_cycle(&mut self,skip:Option<usize>) {
        self.cpu_skip=skip;
        self.cpu_draw_flight=None;
        self.turn=1;
        self.cpu_step=0;
        self.stage=Stage::Cpu;
        // Let a button-triggered discard finish before the next CPU draws.
        // Drag-and-drop discards clear this flight and remain immediate.
        let discard_remaining=self.human_discard_flight.as_ref()
            .map(|flight|(0.60-(get_time()-flight.started)).max(0.0))
            .unwrap_or(0.0);
        self.cpu_next_at=get_time()+if discard_remaining>0.0 {discard_remaining+0.12}else{0.15};
        if self.cpu_speed==0 && discard_remaining==0.0 {
            while self.stage==Stage::Cpu {self.cpu_action();}
        }
    }
}
// Use "an" for the spoken card names eight and eleven.
fn card_article(kind:Kind)->&'static str {
    match kind {Kind::Number(8,_)|Kind::Number(11,_)=>"an",_=>"a"}
}
// Stable player identity colors used in the scoreboard and move history.
fn player_color(i:usize)->Color {
    match i {
        0=>Color::new(0.35,0.87,0.98,1.0), // You: cyan
        1=>Color::new(1.0,0.73,0.37,1.0),  // CPU 1: amber
        2=>Color::new(0.53,0.91,0.60,1.0), // CPU 2: mint
        _=>Color::new(0.89,0.62,1.0,1.0), // CPU 3: lavender
    }
}
// Fixed logical canvas: preserve card proportions and letterbox on narrow windows.
fn draw_game(g:&mut Game,show_debug:bool,show_moves:bool,show_bounds:bool){
    set_default_camera();
    clear_background(active_theme().background());
    let sw=1920.0;let sh=game_canvas_height();
    let (pw,ph)=(screen_width(),screen_height());
    let scale=(pw/sw).min(ph/sh);
    let camera=Camera2D {
        target:vec2(sw/2.0+layout_editor::pan().0,sh/2.0+layout_editor::pan().1),
        zoom:vec2(2.0*scale/pw,2.0*scale/ph)*layout_editor::zoom(),
        ..Default::default()
    };
    set_camera(&camera);
    expire_human_flights(g);
    if active_theme()==Theme::Futuristic { neon_frame(12.0,8.0,sw-24.0,48.0,true); }
    let logo=layout::rect("game_logo",layout::Rect{x:24.,y:10.,w:230.,h:42.});
    menu_label("PHASEUP",logo.x,logo.y+logo.h-10.,32.,ACCENT);
    let logo_width=menu_text_width("PHASEUP",32);
    let version="v0.3.2-dev";
    let version_x=logo.x+logo_width+22.0;
    label(version,version_x,40.,19.,MUTED);
    let round_status=format!("ROUND {}  |  TURN {}",g.round,g.turn_count);
    let round_status_width=menu_text_width(&round_status,22);
    // Right-align before the MENU button, leaving a clear gutter for wider theme fonts.
    menu_label(&round_status,1710.0-22.0-round_status_width,38.0,22.0,MUTED);
    let (turn_text,turn_color)=match g.stage {
        Stage::Draw=>("YOUR TURN  -  DRAW 1 CARD TO BEGIN",ACCENT),
        Stage::Cpu=>("CPU TURN  -  WATCH THE ACTIONS",YELLOW),
        Stage::Play if g.players[0].laid=>("YOUR TURN  -  HIT OR DISCARD",YELLOW),
        Stage::Play=>("YOUR TURN  -  LAY PHASE OR DISCARD",YELLOW),
        Stage::RoundOver=>("ROUND COMPLETE",MUTED),
        Stage::GameOver=>("GAME COMPLETE",MUTED),
    };
    let banner=layout::rect("turn_banner",layout::Rect{x:16.,y:55.,w:sw-32.,h:62.});
    if active_theme()!=Theme::Futuristic {draw_rectangle(banner.x,banner.y,banner.w,banner.h,active_theme().panel());}
    else {for i in 0..3 {let panel=layout::rect(match i {0=>"cpu_1_panel",1=>"cpu_2_panel",_=>"cpu_3_panel"},layout::Rect{x:8.0+i as f32*636.0,y:72.,w:632.,h:376.});neon_cpu_panel(panel.x,panel.y,panel.w,panel.h,if g.stage==Stage::Cpu && g.turn==i+1 {1.0}else{0.0},PHASES[g.players[i+1].phase.min(PHASES.len()-1)].len()==1);}}
    for i in 1..4 {let x=if active_theme()==Theme::Futuristic {28.0+(i-1) as f32*632.0}else{28.+(i-1) as f32*(sw-50.)/3.};let p=&g.players[i];if g.stage==Stage::Cpu&&g.turn==i && !(active_theme()==Theme::Futuristic && has_neon_cpu_panel()){draw_rectangle_lines(16.0+(i-1) as f32*632.0,72.0,626.0,376.0,3.0,YELLOW);}let cpu_status=format!("Score {}{}  |  {} / {}",p.score,if p.laid{"  - LAID"}else{""},g.cpu_roster[i-1].difficulty.name(),g.cpu_roster[i-1].style.name());
        let status_x=if active_theme()==Theme::Futuristic && has_neon_cpu_panel(){16.0+(i-1) as f32*632.0+(626.0-menu_text_width(&cpu_status,16))/2.0}else{x+12.0};
        menu_label(&cpu_status,status_x,124.,16.,MUTED); 
        // Face-down fanned hand: the visible backs represent the actual hand count.
        let count=p.hand.len();
        let card_w=if active_theme()==Theme::Futuristic && has_neon_cpu_panel(){72.6}else{54.0};let card_h=if active_theme()==Theme::Futuristic && has_neon_cpu_panel(){90.2}else{70.0};
        let step=if count>1 {if active_theme()==Theme::Futuristic && has_neon_cpu_panel(){26.0}else{column_hand_width(count,card_w)/(count-1) as f32}}else{0.0};
        for k in 0..count {
            if g.cpu_draw_flight.as_ref().is_some_and(|a|a.player==i && a.card_id==p.hand[k].id) {continue;}
            cpu_card_back(x+10.0+k as f32*step,if active_theme()==Theme::Futuristic && has_neon_cpu_panel(){153.0}else{162.0},card_w,card_h);
        }
        // Identify the player directly beneath their face-down hand.
        let name=g.cpu_name(i);
        menu_label(&name,x+12.0,146.,18.,player_color(i));
        let name_width=menu_text_width(&name,18);
        menu_label(&format!("  |  Phase {}  |  {} cards",p.phase.min(9)+1,p.hand.len()),x+12.0+name_width,146.,18.,WHITEISH);
    }
    let table_y=241.;
    let mut hit_click=None;
    for i in 0..4 {
        let p=&g.players[i];
        // Unlaid phases retain dim silhouettes for every player, including you.
        // The human placeholder is separate from the interactive staging area.
        // These placeholders are decorative, not valid hit targets.
        if !p.laid {
            let x=if i==0 {sw/2.0-400.0}else if active_theme()==Theme::Futuristic && has_neon_cpu_panel(){32.0+(i-1) as f32*632.0}else{24.0+(i-1) as f32*((sw-50.0)/3.0)};
            let y=if i==0 {sh-532.0}else{table_y+22.0};
            if i==0 {menu_label("YOUR LAID PHASE",x,sh-544.0,17.0,player_color(i));}
            let phase=p.phase.min(PHASES.len()-1);
            let group_count=PHASES[phase].len();
            let player_art=i==0 && active_theme()==Theme::Futuristic && neon_player_phase_panel(x-16.0,y-46.0,832.0,181.0,group_count==1);
            for j in 0..group_count {
                let width=if i==0 {if group_count==1 {800.0}else{397.0}}else{if group_count==1 {((sw-50.0)/3.0-8.0).max(95.0)}else{305.0}};
                let base_x=x+j as f32*if i==0 {403.0}else{312.0};
                let group_rect=layout::rect(if i==0 {"player_phase"}else{layout::cpu_group_key(i,j)},layout::Rect{x:base_x,y,w:width,h:116.});
                let gx=group_rect.x;
                let y=group_rect.y;
                let width=group_rect.w;
                if !player_art && !(active_theme()==Theme::Futuristic && i>0 && has_neon_cpu_panel()) {
                    draw_rectangle(gx,y,width,116.0,Color::new(0.075,0.105,0.145,1.0));
                    draw_rectangle_lines(gx,y,width,116.0,1.0,Color::new(0.22,0.29,0.35,1.0));
                }
                if !player_art && active_theme()==Theme::Futuristic && (i==0 || !has_neon_cpu_panel()) {neon_frame(gx,y,width,116.0,false);}
                let desc=format!("{}  -  {} cards",requirement(phase,j).to_uppercase(),PHASES[phase][j]);
                // Single-requirement phases occupy the full CPU frame: center
                // their labels across both decorative lower panels.
                let cpu_full_span=i>0 && active_theme()==Theme::Futuristic && has_neon_cpu_panel() && group_count==1;
                let label_x=if cpu_full_span {16.0+(i-1) as f32*632.0+(626.0-menu_text_width(&desc,17))/2.0}
                    else if i>0 && has_neon_cpu_panel(){gx+(width-menu_text_width(&desc,17))/2.0-10.0}else if player_art {gx+(width-menu_text_width(&desc,17))/2.0}else{gx+8.0};
                menu_label(&desc,label_x,y+if player_art {-7.0}else if i>0 && has_neon_cpu_panel(){30.0}else{22.0},17.0,Color::new(0.62,0.71,0.78,1.0));
                let not_laid_x=if cpu_full_span {16.0+(i-1) as f32*632.0+(626.0-menu_text_width("NOT LAID",17))/2.0}
                    else if i>0 && has_neon_cpu_panel(){gx+(width-menu_text_width("NOT LAID",17))/2.0-10.0}else if player_art {gx+(width-menu_text_width("NOT LAID",17))/2.0}else{gx+10.0};
                menu_label("NOT LAID",not_laid_x,y+if player_art {72.0}else if i>0 && has_neon_cpu_panel(){83.0}else{62.0},17.0,Color::new(0.33,0.41,0.48,1.0));
            }
        }
        if !p.laid {continue;}
        let x=if i==0 {sw/2.0-400.0}else if active_theme()==Theme::Futuristic && has_neon_cpu_panel(){32.0+(i-1) as f32*632.0}else{24.0+(i-1) as f32*((sw-50.0)/3.0)};
        let y=if i==0 {sh-544.0}else{table_y+10.0};
        if i==0 {menu_label("YOUR LAID PHASE",x,y,17.0,player_color(i));}
        let player_art=i==0 && active_theme()==Theme::Futuristic && neon_player_phase_panel(x-16.0,y-34.0,832.0,181.0,p.table.len()==1);
        for (j,group) in p.table.iter().enumerate() {
            let yy=if i==0 {y+12.0}else{y+9.0};
            let width=if i==0 {if p.table.len()==1 {800.0}else{397.0}}else{if p.table.len()==1 {((sw-50.0)/3.0-8.0).max(95.0)}else{305.0}};
            let base_x=if i==0 {x+j as f32*403.0}else{x+j as f32*312.0};
            let group_rect=layout::rect(if i==0 {"player_phase"}else{layout::cpu_group_key(i,j)},layout::Rect{x:base_x,y:yy,w:width,h:116.});
            let x=group_rect.x;
            let yy=group_rect.y;
            let width=group_rect.w;
            let selected_card=if g.selected.len()==1 {g.players[0].hand.iter().find(|c|c.id==g.selected[0]).copied()}else{None};
            let can_place=g.stage==Stage::Play&&g.players[0].laid&&selected_card.is_some_and(|c|can_hit(group,c,p.phase,j));
            if !player_art && !(i>0 && active_theme()==Theme::Futuristic && has_neon_cpu_panel()) {
                draw_rectangle(x,yy,width,116.0,if can_place {Color::new(0.13,0.34,0.29,1.0)}else{PANEL});
                draw_rectangle_lines(x,yy,width,116.0,if can_place {3.0}else{1.0},if can_place {ACCENT}else{MUTED});
            }
            let desc=format!("{}  -  {} cards",requirement(p.phase.min(PHASES.len()-1),j).to_uppercase(),group.len());
            let desc_x=if i>0 && active_theme()==Theme::Futuristic && has_neon_cpu_panel() && p.table.len()==1 {
                16.0+(i-1) as f32*632.0+(626.0-menu_text_width(&desc,17))/2.0
            } else if i>0 && has_neon_cpu_panel(){x+(width-menu_text_width(&desc,17))/2.0-10.0}else if player_art {x+(width-menu_text_width(&desc,17))/2.0}else{x+8.0};
            // Place the requirement at the top of the frame, above laid cards.
            let desc_x=if player_art {x+(width-menu_text_width(&desc,17))/2.0}else{desc_x};
            menu_label(&desc,desc_x,yy+if player_art {-7.0}else if i>0 && has_neon_cpu_panel(){30.0}else{22.0},17.0,WHITEISH);
            let card_w=55.0;
            let card_h=84.0;
            // While dragging a valid card from the hand, preview the space it
            // will occupy and shift the existing table cards around that gap.
            let preview=g.drag.and_then(|(id,sx,sy)| {
                let (mx,my)=game_mouse();
                if !is_mouse_button_down(MouseButton::Left)
                    || (mx-sx).abs()+(my-sy).abs()<=12.0
                    || !inside(x,yy,width,116.0) || !g.players[0].laid
                    || g.stage!=Stage::Play {return None;}
                let card=g.players[0].hand.iter().find(|c|c.id==id).copied()?;
                if !can_hit(group,card,p.phase,j){return None;}
                let slot=if requirement(p.phase.min(9),j)=="run" {
                    if let Kind::Number(n,_)=card.kind {
                        group.iter().position(|existing|matches!(existing.kind,Kind::Number(m,_) if m>n)).unwrap_or(group.len())
                    }else{group.len()}
                }else{group.len()};
                Some(slot)
            });
            let shown_len=group.len()+usize::from(preview.is_some());
            // Keep card artwork at its normal size. As the group grows, reduce
            // the horizontal stride rather than shrinking the cards. The final
            // card always fits inside the group frame.
            let inset=if i==0 && player_art {16.0}
                else if i>0 && has_neon_cpu_panel(){if j==0 {6.0}else{2.0}}
                else {6.0};
            // The neon panel has a thick decorative frame and the card artwork
            // visually extends beyond its nominal 55px hitbox. Reserve equal
            // safe margins on both sides before distributing the stack.
            let side_margin=if i>0 && has_neon_cpu_panel(){24.0}else{16.0};
            let usable_w=(width-2.0*side_margin).max(card_w);
            let available=(usable_w-card_w).max(0.0);
            let step=if shown_len>1 {
                (available/(shown_len-1) as f32).min(card_w+1.0)
            }else{0.0};
            let stack_w=card_w+shown_len.saturating_sub(1) as f32*step;
            let stack_x=x+(width-stack_w)/2.0;
            if let Some(slot)=preview {
                let cx=stack_x+slot as f32*step;
                let cy=yy+if i==0 && player_art {6.0}else if i>0 && has_neon_cpu_panel(){36.0}else{26.0};
                draw_rectangle_lines(cx,cy,card_w,card_h,2.5,YELLOW);
            }
            for (k,card) in group.iter().enumerate() {
                if i==0 && g.human_phase_flight.as_ref().is_some_and(|flight|!flight.visible(card.id,get_time())) {continue;}
                if i!=0 && g.cpu_phase_flight.as_ref().is_some_and(|flight|flight.player==i && !flight.visible(j,k,get_time())) {continue;}
                if g.cpu_hit_flight.as_ref().is_some_and(|flight|flight.target==i && flight.group==j && flight.card.id==card.id) {continue;}
                let shown=k+usize::from(preview.is_some_and(|slot|k>=slot));
                let cx=stack_x+shown as f32*step;
                let cy=yy+if i==0 && player_art {6.0}else if i>0 && has_neon_cpu_panel(){36.0}else{26.0};
                let card_pos=layout::rect(&format!("laid_card_{}",card.id),layout::Rect{x:cx,y:cy,w:card_w,h:card_h});
                let (cx,cy)=(card_pos.x,card_pos.y);
                if active_theme()==Theme::Futuristic {
                    let _=card_ui(*card,cx,cy,card_w,card_h,false);
                } else {
                    let color=color_for(card.kind);
                    draw_rectangle(cx+1.0,cy+2.0,card_w,card_h,BG);
                    draw_rectangle(cx,cy,card_w,card_h,WHITEISH);
                    draw_rectangle(cx+3.0,cy+3.0,card_w-6.0,card_h-6.0,color);
                    let value=card_text(card.kind);
                    label(&value,cx+8.0,cy+36.0,28.0,if matches!(card.kind,Kind::Number(_,2)){BG}else{WHITE});
                }
            }
            if can_place {label("CLICK TO ADD",x+7.0,yy+114.0,12.0,ACCENT);}
            if is_mouse_button_pressed(MouseButton::Left)&&inside(x,yy,width,116.0) {
                hit_click=Some((i,j));
            }
            // Dropping a dragged hand card on a laid group plays it directly.
            if is_mouse_button_released(MouseButton::Left)&&inside(x,yy,width,116.0) {
                if let Some((id,sx,sy))=g.drag {
                    let (mx,my)=game_mouse();
                    if (mx-sx).abs()+(my-sy).abs()>12.0 {
                        g.selected=vec![id];
                        hit_click=Some((i,j));
                    }
                }
            }
        }
    }
    // Larger, layered piles share a stable hitbox with drag/drop and animations.
    let center=if active_theme()==Theme::Futuristic {510.0}else{535.0};
    let pile_y=center-50.0;
    let pile_w=112.0;
    let pile_h=150.0;
    let deck_x=sw/2.0-200.0;
    let discard_x=sw/2.0+88.0;
    menu_label("DRAW",deck_x,pile_y-17.0,19.0,MUTED);
    menu_label("DISCARD",discard_x,pile_y-17.0,19.0,MUTED);
    if active_theme()==Theme::Futuristic && neon_draw_back(deck_x,pile_y,pile_w,pile_h) {
        if g.stage==Stage::Draw {
            draw_rectangle_lines(deck_x-2.0,pile_y-2.0,pile_w+4.0,pile_h+4.0,2.0,active_theme().accent());
        }
        let count_text=format!("{} left",g.deck.len());
        menu_label(&count_text,deck_x+(pile_w-menu_text_width(&count_text,17))/2.0,pile_y+pile_h+22.0,17.0,WHITEISH);
    } else {
    // Offset layers expose the thickness of the remaining draw deck.
    let layers=g.deck.len().min(3);
    for layer in (0..layers).rev() {
        let offset=layer as f32*4.0;
        draw_rectangle(deck_x+offset+3.0,pile_y+offset+4.0,pile_w,pile_h,Color::new(0.035,0.055,0.085,1.0));
        draw_rectangle(deck_x+offset,pile_y+offset,pile_w,pile_h,Color::new(0.13,0.29,0.40,1.0));
        draw_rectangle_lines(deck_x+offset,pile_y+offset,pile_w,pile_h,1.5,active_theme().accent());
    }
    draw_rectangle(deck_x,pile_y,pile_w,pile_h,Color::new(0.19,0.39,0.52,1.0));
    draw_rectangle_lines(deck_x,pile_y,pile_w,pile_h,if g.stage==Stage::Draw {4.0}else{2.0},if g.stage==Stage::Draw {YELLOW}else{ACCENT});
    draw_line(deck_x+13.0,pile_y+15.0,deck_x+pile_w-13.0,pile_y+pile_h-15.0,1.0,active_theme().accent());
    draw_line(deck_x+pile_w-13.0,pile_y+15.0,deck_x+13.0,pile_y+pile_h-15.0,1.0,active_theme().accent());
    draw_rectangle(deck_x+8.0,pile_y+55.0,pile_w-16.0,51.0,Color::new(0.12,0.28,0.38,0.96));
    let draw_text="DRAW";
    menu_label(draw_text,deck_x+(pile_w-menu_text_width(draw_text,19))/2.0,pile_y+77.0,19.0,WHITEISH);
    let count_text=format!("{} left",g.deck.len());
    menu_label(&count_text,deck_x+(pile_w-menu_text_width(&count_text,17))/2.0,pile_y+99.0,17.0,WHITEISH);
    }
    let m=game_mouse();
    if g.stage==Stage::Draw&&is_mouse_button_pressed(MouseButton::Left)&&inside(deck_x,pile_y,pile_w,pile_h){g.pile_drag=Some((false,m.0,m.1));}
    let discard_hover=g.stage==Stage::Play&&g.drag.is_some()&&inside(discard_x,pile_y,pile_w,pile_h);
    let discard_layers=g.discard.len().saturating_sub(1).min(2);
    for layer in (1..=discard_layers).rev() {
        let offset=layer as f32*4.0;
        draw_rectangle(discard_x+offset+3.0,pile_y+offset+4.0,pile_w,pile_h,Color::new(0.035,0.055,0.085,1.0));
        draw_rectangle(discard_x+offset,pile_y+offset,pile_w,pile_h,Color::new(0.17,0.24,0.30,1.0));
        draw_rectangle_lines(discard_x+offset,pile_y+offset,pile_w,pile_h,1.5,MUTED);
    }
    draw_rectangle(discard_x,pile_y,pile_w,pile_h,Color::new(0.10,0.18,0.23,1.0));
    draw_rectangle_lines(discard_x,pile_y,pile_w,pile_h,if discard_hover||g.stage==Stage::Draw {3.0}else{2.0},if discard_hover {GREEN}else if g.stage==Stage::Draw {YELLOW}else{ACCENT});
    if let Some(c)=g.discard.last().copied(){
        if !g.cpu_discard_flight.as_ref().is_some_and(|a|a.card.id==c.id) && !g.human_discard_flight.as_ref().is_some_and(|a|a.card.id==c.id){
            let _=card_ui(c,discard_x,pile_y,pile_w,pile_h,false);
        }
        if g.stage==Stage::Draw&&is_mouse_button_pressed(MouseButton::Left)&&inside(discard_x,pile_y,pile_w,pile_h){g.pile_drag=Some((true,m.0,m.1));}
    }else{
        menu_label("EMPTY",discard_x+20.0,pile_y+82.0,20.0,MUTED);
    }
    // Recent CPU actions stay visible after their turns resolve instantly.
    if show_moves {
    let log_x=1538.0;let log_y=480.0;
    draw_rectangle(log_x,log_y,374.0,300.0,active_theme().panel());
    draw_rectangle_lines(log_x,log_y,374.0,300.0,1.5,active_theme().accent());
    menu_label("RECENT MOVES",log_x+12.0,log_y+27.0,20.0,ACCENT);
    // Separate the panel heading from the history, including the newest-move highlight.
    draw_line(log_x+10.0,log_y+39.0,log_x+364.0,log_y+39.0,1.0,active_theme().accent());
    // Put the live action *inside* the panel so laid phases cannot cover it.
    let events_y=if g.stage==Stage::Cpu {
        let action=match g.cpu_step {0=>"DRAWING",1=>"CHECKING PHASE",2=>"HITTING TABLE",_=>"DISCARDING"};
        let notice=format!("{} - {}",g.cpu_name(g.turn.min(3)).to_uppercase(),action);
        draw_rectangle(log_x+9.0,log_y+34.0,356.0,36.0,BG);
        draw_rectangle_lines(log_x+9.0,log_y+34.0,356.0,36.0,1.5,YELLOW);
        let notice_size=17;
        let width=menu_text_width(&notice,notice_size);
        menu_label(&notice,log_x+187.0-width/2.0,log_y+59.0,notice_size as f32,YELLOW);
        log_y+113.0
    }else{log_y+69.0};
    let max_events=8;
    for (j,event) in g.move_log.iter().rev().take(max_events).enumerate(){
        // Highlight the newest completed move, while retaining the live CPU action above.
        let entry_y=events_y+j as f32*23.0;
        if j==0 {
            draw_rectangle(log_x+7.0,entry_y-17.0,360.0,23.0,Color::new(0.14,0.27,0.35,1.0));
            draw_rectangle(log_x+7.0,entry_y-17.0,3.0,23.0,active_theme().accent());
        }
        // Color the actor's name, or the skipped player's name for passive skip events.
        let actor=(0..4).find_map(|i|{
            let name=g.cpu_name(i);
            let is_actor=event==&name || event.starts_with(&format!("{} ",name));
            if is_actor {Some((i,name))}else{None}
        });
        if let Some((i,name))=actor {
            menu_label(&name,log_x+14.0,entry_y,18.0,player_color(i));
            let rest=&event[name.len()..];
            menu_label(rest,log_x+14.0+menu_text_width(&name,18),entry_y,18.0,WHITEISH);
        }else{
            menu_label(event,log_x+14.0,entry_y,18.0,WHITEISH);
        }
    }
    }
    // Phase staging: cards remain in the player's actual hand until played.
    let stage_y=sh-347.0;
    // The completed player advances to phase index 10, beyond the ten phase definitions.
    // Keep rendering the final phase while the GameOver overlay is displayed.
    let groups=PHASES[g.players[0].phase.min(PHASES.len()-1)];
    let stage_w=if groups.len()==1 {800.0}else{680.0};
    let stage_left=(sw-(groups.len() as f32*stage_w+(groups.len().saturating_sub(1)) as f32*6.0))/2.0;
    let mut staged_click=None;
    let mut stage_drop=None;
    let mut staged_reorder=None;
    for group in 0..groups.len() {
        let slot=layout::rect(if group==0 {"stage_1"}else{"stage_2"},layout::Rect{x:stage_left+group as f32*(stage_w+6.0),y:stage_y,w:stage_w,h:117.});
        let x=slot.x;
        let y=slot.y;
        let stage_w=slot.w;
        draw_rectangle(x,y,stage_w,117.0,PANEL);
        let hovering=inside(x,y,stage_w,117.0);
        let dragging=g.drag.is_some_and(|(_,sx,sy)| {
            let (mx,my)=game_mouse();
            is_mouse_button_down(MouseButton::Left)&&(mx-sx).abs()+(my-sy).abs()>10.0
        });
        if hovering&&dragging&&g.stage==Stage::Play {draw_rectangle(x+2.0,y+2.0,stage_w-4.0,113.0,Color::new(0.12,0.43,0.39,0.65));}
        draw_rectangle_lines(x,y,stage_w,117.0,if hovering&&dragging&&g.stage==Stage::Play {3.0}else{1.5},if g.stage==Stage::Draw && g.turn_count==1 {MUTED}else if hovering&&dragging {YELLOW}else{ACCENT});
        let ids=g.staged[group].clone();
        let cards:Vec<Card>=ids.iter().filter_map(|id|g.players[0].hand.iter().find(|c|c.id==*id).copied()).collect();
        let ready=cards.len()>=groups[group] && valid_group(&cards,requirement(g.players[0].phase.min(PHASES.len()-1),group));
        menu_label(&format!("STAGE {} {} / {}{}",requirement(g.players[0].phase.min(PHASES.len()-1),group).to_uppercase(),cards.len(),groups[group],if ready{"  READY"}else{""}),x+10.0,y+20.0,16.0,if ready{GREEN}else{WHITEISH});
        let drag_preview=g.staged_drag.or(g.drag).and_then(|(id,sx,sy)|{
            let (mx,my)=game_mouse();
            if is_mouse_button_down(MouseButton::Left)
                && (mx-sx).abs()+(my-sy).abs()>10.0
                && inside(x,y,stage_w,117.0) {
                let slot=(((mx-(x+10.0)+28.5)/57.0).floor().max(0.0)) as usize;
                Some((id,slot))
            } else {None}
        });
        let visible_cards:Vec<Card>=cards.iter().copied()
            .filter(|card|drag_preview.is_none_or(|(id,_)|card.id!=id)).collect();
        let insert_at=drag_preview.map(|(_,slot)|slot.min(visible_cards.len()));
        if let Some(slot)=insert_at {
            let gap_x=x+10.0+slot as f32*57.0;
            draw_rectangle_lines(gap_x,y+29.0,50.0,76.0,2.0,YELLOW);
        }
        for (i,card) in visible_cards.iter().enumerate() {
            let shifted=i+usize::from(insert_at.is_some_and(|slot|i>=slot));
            let cx=x+10.0+shifted as f32*57.0;
            let card_pos=layout::rect(&format!("stage_card_{}",card.id),layout::Rect{x:cx,y:y+29.0,w:50.0,h:76.0});
            let cx=card_pos.x;
            if cx+51.0<x+stage_w {
                let floating=g.staged_drag.is_some_and(|(id,sx,sy)|{
                    let (mx,my)=game_mouse();
                    id==card.id&&is_mouse_button_down(MouseButton::Left)&&(mx-sx).abs()+(my-sy).abs()>10.0
                });
                if floating {
                    draw_rectangle(cx,y+29.0,50.0,76.0,Color::new(0.24,0.32,0.40,0.45));
                    draw_rectangle_lines(cx,y+29.0,50.0,76.0,1.0,MUTED);
                } else {let _=card_ui(*card,cx,y+29.0,50.0,76.0,false);}
                if g.stage==Stage::Play&&is_mouse_button_pressed(MouseButton::Left)&&inside(cx,y+29.0,50.0,76.0){g.staged_drag=Some((card.id,game_mouse().0,game_mouse().1));}
            }
        }
        if g.stage==Stage::Play&&is_mouse_button_released(MouseButton::Left) && inside(x,y,stage_w,117.0) {
            stage_drop=Some(group);
            if let Some((id,sx,sy))=g.staged_drag {
                let (mx,my)=game_mouse();
                if (mx-sx).abs()+(my-sy).abs()>10.0 {
                    let slot=(((mx-(x+10.0)+28.5)/57.0).floor().max(0.0)) as usize;
                    let source=g.staged.iter().position(|bucket|bucket.contains(&id));
                    let target_index=if source==Some(group){let old=g.staged[group].iter().position(|v|*v==id).unwrap_or(0);if slot>old{slot.saturating_sub(1)}else{slot}}else{slot};
                    staged_reorder=Some((id,group,target_index));
                }
            }
        }
        // The whole empty portion of the staging panel acts as the click target
        // for selected hand cards; no separate STAGE 1 / STAGE 2 button needed.
        if g.stage==Stage::Play && !g.selected.is_empty()
            && is_mouse_button_pressed(MouseButton::Left)
            && inside(x+10.0,y+32.0,stage_w-20.0,75.0)
            && g.drag.is_none() && g.staged_drag.is_none()
            && !cards.iter().enumerate().any(|(j,_)|inside(x+10.0+j as f32*57.0,y+30.0,50.0,78.0))
        {
            g.stage_selected(group);
        }
    }
    if is_mouse_button_released(MouseButton::Left){
        if let Some((id,sx,sy))=g.staged_drag {
            let (mx,my)=game_mouse();
            let moved=(mx-sx).abs()+(my-sy).abs()>10.0;
            if !moved {
                staged_click=Some(id);
            } else if inside(12.0,sh-175.0-72.0,sw-24.0,247.0) {
                // Dragging a staged card back into the hand unstages it,
                // without discarding it or changing the player's selection.
                g.unstage_card(id);
            }
        }
        g.staged_drag=None;
    }
    if let Some((id,group,index))=staged_reorder {g.move_staged_card(id,group,index);}
    if let Some(id)=staged_click {g.unstage_card(id);}
    if g.stage==Stage::Draw && g.turn_count==1 {for group in 0..groups.len(){let x=stage_left+group as f32*(stage_w+6.0);draw_rectangle(x,stage_y,stage_w,117.0,Color::new(0.04,0.07,0.10,0.72));let t="DRAW FIRST";let tw=menu_text_width(t,24);menu_label(t,x+(stage_w-tw)/2.0,stage_y+67.0,24.0,MUTED);}}
    let hand_panel=layout::rect("hand_panel",layout::Rect{x:12.,y:sh-229.,w:sw-24.,h:217.});
    let py=hand_panel.y+72.0;
    draw_rectangle(hand_panel.x,hand_panel.y,hand_panel.w,hand_panel.h,active_theme().panel());
    if active_theme()==Theme::Futuristic {neon_frame(hand_panel.x,hand_panel.y,hand_panel.w,hand_panel.h,true);}
    // Draw inside the hand panel, after its background and before the cards.
    // Taller, high-contrast banner; hand cards start lower to clear it.
    let phase_bottom=(sh-532.0)+116.0;
    let guidance_y=phase_bottom+(stage_y-phase_bottom-43.0)/2.0;
    // When a phase is ready, the entire guidance banner becomes the
    // LAY PHASE button. Draw it later, after staging validation.
    let guidance_x=(sw-770.0)/2.0;
    let guidance_w=770.0;
    let guidance_h=43.0;
    let p=&g.players[0];
    let hand_info=format!("YOUR HAND  |  Phase {}: {}  |  Score {}",p.phase.min(9)+1,phase_name(p.phase),p.score);
    let hand_info_width=menu_text_width(&hand_info,21);
    menu_label(&hand_info,(sw-hand_info_width)/2.0,py-46.0,21.0,WHITEISH);
    let sorts=[(SortMode::Runs,"RUNS"),(SortMode::Colors,"COLORS"),(SortMode::Sets,"SETS")];
    for (i,(mode,name)) in sorts.iter().enumerate() {
        let x=(sw-321.0)/2.0+i as f32*109.0;
        let sort_y=sh-51.0;
        if button(name,x,sort_y,103.0,28.0,true) {g.sort=*mode;}
        if g.sort==*mode {draw_rectangle_lines(x,sort_y,103.0,28.0,2.0,ACCENT);}
    }
    let mut hand:Vec<Card>=p.hand.iter().copied().filter(|c|!g.staged.iter().any(|v|v.contains(&c.id))).collect();
    hand.sort_by_key(|c|sort_key(*c,g.sort));
    let n=hand.len();let w=((sw-48.0)/(n.max(1) as f32)).min(75.0);
    let spacing=if n>1{((sw-48.0-w)/(n-1) as f32).min(81.0)}else{0.0};
    let start=(sw-(spacing*(n.saturating_sub(1) as f32)+w))/2.0;
    let card_y=py-9.0;
    // Hover lifts a card without changing its selection or gameplay state.
    let (hand_mx,hand_my)=game_mouse();
    let hovered_hand_card=if g.drag.is_none() {
        hand.iter().enumerate().rev().find(|(i,c)| {
            !g.human_draw_flight.as_ref().is_some_and(|a|a.card_id==c.id) && {
                let x=start+*i as f32*spacing;
                let selected_lift=if active_theme()!=Theme::Futuristic && g.selected.contains(&c.id){15.0}else{0.0};
                hand_mx>=x&&hand_mx<=x+w&&hand_my>=card_y-selected_lift-18.0&&hand_my<=card_y+105.0
            }
        }).map(|(_,c)|c.id)
    } else {None};
    // Animate each card independently over 100ms with smoothstep easing.
    // Retarget from its current visual height when the hover changes mid-flight.
    let now=get_time();
    let ease=|t:f32| t*t*(3.0-2.0*t);
    g.hover_lifts.retain(|(id,_,_,_)|hand.iter().any(|c|c.id==*id));
    for c in &hand {
        let target=if hovered_hand_card==Some(c.id) || (active_theme()==Theme::Futuristic && (g.selected.contains(&c.id) || g.drag.is_some_and(|(id,_,_)|id==c.id && is_mouse_button_down(MouseButton::Left)))){18.0}else{0.0};
        if let Some(state)=g.hover_lifts.iter_mut().find(|state|state.0==c.id) {
            if (state.2-target).abs()>0.01 {
                let t=((now-state.3)/0.10).clamp(0.0,1.0) as f32;
                let current=state.1+(state.2-state.1)*ease(t);
                *state=(c.id,current,target,now);
            }
        } else if target>0.0 {
            g.hover_lifts.push((c.id,0.0,target,now));
        }
    }
    let mut toggle=None;
    for (i,c) in hand.iter().enumerate() {
        let x=start+i as f32*spacing;
        if g.human_draw_flight.as_ref().is_some_and(|a|a.card_id==c.id) {continue;}
        let is_dragging=g.drag.is_some_and(|(id,sx,sy)| {
            let (mx,my)=game_mouse();
            id==c.id&&is_mouse_button_down(MouseButton::Left)&&(mx-sx).abs()+(my-sy).abs()>10.0
        });
        let selected=g.selected.contains(&c.id);
        let hover_lift=g.hover_lifts.iter().find(|state|state.0==c.id).map(|state|{
            let t=((now-state.3)/0.10).clamp(0.0,1.0) as f32;
            state.1+(state.2-state.1)*ease(t)
        }).unwrap_or(0.0);
        let lifted_y=card_y-hover_lift;
        if is_dragging {
            draw_rectangle(x,card_y,w,105.0,Color::new(0.24,0.32,0.40,0.45));
            draw_rectangle_lines(x,card_y,w,105.0,1.0,MUTED);
        } else {
            if card_ui(*c,x,lifted_y,w,105.0,selected){toggle=Some(c.id);}
            if g.drawn_card==Some(c.id){
                let y=lifted_y-if selected && active_theme()!=Theme::Futuristic{15.0}else{0.0};
                draw_rectangle_lines(x-3.0,y-3.0,w+6.0,111.0,4.0,YELLOW);
            }
        }
        if is_mouse_button_pressed(MouseButton::Left)&&inside(x,lifted_y-if selected && active_theme()!=Theme::Futuristic{15.0}else{0.0},w,105.0){
            g.drag=Some((c.id,game_mouse().0,game_mouse().1));
        }
    }
    // Clicking a hand card toggles only that card; phase staging still supports multiple cards.
    if let Some(id)=toggle {if g.selected.contains(&id){g.selected.clear();}else{g.selected=vec![id];}}
    // Draw the dragged card last so it floats above every other UI element.
    if let Some((id,ox,oy))=g.drag {
        let (mx,my)=game_mouse();
        if is_mouse_button_down(MouseButton::Left)&&(mx-ox).abs()+(my-oy).abs()>10.0 {
            if let Some(card)=g.players[0].hand.iter().find(|c|c.id==id).copied() {
                let drag_w=77.0;
                let drag_h=110.0;
                let dx=mx-drag_w/2.0;
                let dy=my-drag_h/2.0;
                draw_rectangle(dx+6.0,dy+8.0,drag_w,drag_h,Color::new(0.0,0.0,0.0,0.42));
                let _=card_ui(card,dx,dy,drag_w,drag_h,false);
            }
        }
    }
    // Click a pile to draw, or drag its card into the hand.
    if let Some((from_discard,sx,sy))=g.pile_drag {
        let (mx,my)=game_mouse();
        if is_mouse_button_down(MouseButton::Left)&&(mx-sx).abs()+(my-sy).abs()>12.0 {
            let x=mx-41.0;let y=my-55.0;
            draw_rectangle(x,y,82.0,110.0,if from_discard {WHITEISH}else{ACCENT});
            draw_rectangle_lines(x,y,82.0,110.0,3.0,YELLOW);
            label(if from_discard {"DISCARD"}else{"DRAW"},x+6.0,y+59.0,17.0,BG);
        }
    }
    if is_mouse_button_released(MouseButton::Left) {
        let (mx,my)=game_mouse();
        if let Some((id,ox,oy))=g.drag {
            if (mx-ox).abs()+(my-oy).abs()>12.0 {
                if g.stage==Stage::Play&&inside(discard_x,pile_y,pile_w,pile_h) {
                    g.selected=vec![id];
                    g.discard_selected();
                    // Drag-and-drop is already a direct physical movement.
                    // Only button-triggered discards should fly.
                    g.human_discard_flight=None;
                } else if let Some(group)=stage_drop {
                    let x=stage_left+group as f32*(stage_w+6.0);
                    let index=(((mx-(x+10.0)+28.5)/57.0).floor().max(0.0)) as usize;
                    g.stage_card_at(id,group,index);
                }
            }
        }
        g.drag=None;
        if let Some((from_discard,sx,sy))=g.pile_drag {
            let moved=(mx-sx).abs()+(my-sy).abs()>12.0;
            let source=if from_discard {inside(discard_x,pile_y,pile_w,pile_h)}else{inside(deck_x,pile_y,pile_w,pile_h)};
            if g.stage==Stage::Draw&&((!moved&&source)||(moved&&inside(12.0,py-72.0,sw-24.0,247.0))) {g.draw(from_discard,!moved);}
        }
        g.pile_drag=None;
    }
    // Render staged-card drags at the end of the game scene, above the hand,
    // overlays and table, just like cards dragged out of the hand.
    if let Some((id,sx,sy))=g.staged_drag {
        let (mx,my)=game_mouse();
        if is_mouse_button_down(MouseButton::Left)&&(mx-sx).abs()+(my-sy).abs()>10.0 {
            if let Some(card)=g.players[0].hand.iter().find(|c|c.id==id).copied() {
                let _=card_ui(card,mx-38.5,my-55.0,77.0,110.0,false);
            }
        }
    }
    // Completion banner sits between the staging slots and turn guidance.
    let phase=g.players[0].phase.min(PHASES.len()-1);
    let staged_complete=g.stage==Stage::Play&&!g.players[0].laid
        && (0..groups.len()).all(|group|{
            let cards:Vec<Card>=g.staged[group].iter()
                .filter_map(|id|g.players[0].hand.iter().find(|c|c.id==*id).copied()).collect();
            cards.len()>=groups[group]&&valid_group(&cards,requirement(phase,group))
        });
    if staged_complete {
        // A single ready-to-lay banner avoids overlapping the laid-phase area.
        if button("PHASE COMPLETE  -  LAY PHASE",guidance_x,guidance_y,guidance_w,guidance_h,true) {
            g.selected=g.staged.iter().flatten().copied().collect();
            g.lay();
        }
        // Keep the original phase-complete green fill, inside the clickable button.
        draw_rectangle(guidance_x+4.0,guidance_y+4.0,guidance_w-8.0,guidance_h-8.0,Color::new(0.08,0.28,0.22,1.0));
        draw_rectangle_lines(guidance_x,guidance_y,guidance_w,guidance_h,3.0,GREEN);
        let ready_text="PHASE COMPLETE  -  LAY PHASE";
        let ready_w=menu_text_width(ready_text,21);
        menu_label(ready_text,guidance_x+(guidance_w-ready_w)/2.0,guidance_y+guidance_h/2.0+7.0,21.0,GREEN);
    } else {
        draw_rectangle(guidance_x,guidance_y,guidance_w,guidance_h,BG);
        draw_rectangle_lines(guidance_x,guidance_y,guidance_w,guidance_h,2.0,turn_color);
        let tw=menu_text_width(turn_text,25);
        menu_label(turn_text,sw/2.0-tw/2.0,guidance_y+30.0,25.0,turn_color);
    }
    // Hand-area controls match the mockup: clears in the left margin,
    // discard at the right, and sorting buttons remain centered.
    // Position the clear controls immediately left of the phase staging slots.
    // Keep clear buttons entirely in the gutter left of the staging panels.
    // Derive their position from the actual current staging width.
    let control_w=154.0;
    let control_h=30.0;
    let control_x=(stage_left-control_w)/2.0;
    let control_y=stage_y+2.0;
    let has_staged=g.staged.iter().any(|group|!group.is_empty());
    if groups.len()==1 {
        if button("CLEAR",control_x,control_y+39.0,control_w,control_h,!g.staged[0].is_empty()) {
            g.staged[0].clear();
        }
    } else {
        if button("CLEAR ALL",control_x,control_y,control_w,control_h,has_staged) {
            g.staged=[vec![],vec![]];
        }
        if button("CLEAR 1",control_x,control_y+39.0,control_w,control_h,!g.staged[0].is_empty()) {
            g.staged[0].clear();
        }
        if button("CLEAR 2",control_x,control_y+78.0,control_w,control_h,!g.staged[1].is_empty()) {
            g.staged[1].clear();
        }
    }
    let discard_ready=g.stage==Stage::Play&&g.selected.len()==1;
    let discard_y=sh-51.0;
    let discard_width=103.0;
    // Mirror the right discard button across the center of the game board.
    for discard_x in [1920.0-1250.0-discard_width,1250.0] {
        if button("DISCARD",discard_x,discard_y,discard_width,28.0,discard_ready){
            g.discard_selected();
        }
        if discard_ready {
            draw_rectangle_lines(discard_x,discard_y,discard_width,28.0,3.0,YELLOW);
        }
    }
    // Show contextual instructions above staging, not beneath the hand cards
    // where they collided with the sorting controls.
    // Contextual text is available through the game log; do not paint it
    // over the narrow gap reserved for the turn-status banner.
    if let Some((target,group))=hit_click {g.hit(target,group);}
    // Keep the board visible until the winning discard reaches the pile.
    // The round result is already calculated, but its dialog waits for the flight.
    let discard_in_motion=g.human_discard_flight.is_some()||g.cpu_discard_flight.is_some();
    if (g.stage==Stage::RoundOver||g.stage==Stage::GameOver)&&!discard_in_motion {
        draw_rectangle(0.,0.,sw,sh,Color::new(0.,0.,0.,0.77));let x=sw/2.-265.;let y=sh/2.-164.;draw_rectangle(x,y,530.,328.,PANEL);
        let game_over=g.stage==Stage::GameOver;
        let title=if game_over {"GAME COMPLETE".to_string()}else{format!("ROUND {} COMPLETE",g.round)};
        let title_w=menu_text_width(&title,32);
        menu_label(&title,x+(530.0-title_w)/2.0,y+47.,32.,ACCENT);
        let winner_name=g.cpu_name(g.turn).to_uppercase();
        let announcement=if game_over {if g.turn==0 {"YOU FINISHED THE GAME!".to_string()}else{format!("{} FINISHED THE GAME!",winner_name)}}else if g.turn==0 {"YOU WIN THE ROUND!".to_string()}else{format!("{} WINS THE ROUND!",winner_name)};
        let announce_w=menu_text_width(&announcement,27);
        menu_label(&announcement,x+(530.0-announce_w)/2.0,y+91.,27.,YELLOW);
        menu_label("TOTAL SCORES",x+29.,y+128.,18.,MUTED);
        for i in 0..4 {
            let p=&g.players[i];
            let name=g.cpu_name(i);
            let line=format!("{}{}  |  Phase {}  |  {} pts",name,if i==g.turn {"  (OUT)"}else{""},p.phase.min(10),p.score);
            menu_label(&line,x+29.,y+159.+i as f32*29.,20.,if i==g.turn {YELLOW}else{WHITEISH});
        }
        if game_over {
            if button("NEW GAME",x+60.,y+277.,190.,40.,true){*g=Game::new();}
            let _=button("MAIN MENU",x+280.,y+277.,190.,40.,true);
        } else if button("NEXT ROUND",x+170.,y+277.,190.,40.,true){g.deal();}
    }
    draw_card_flights(g,sw,sh);
    if show_bounds {draw_cpu_bounds(g);}
    if show_debug {debug_overlay(g);}
}

// F4 visual diagnostic: show the actual layout coordinates used by the renderer.
// Red = panel/group rectangles, orange = card footprint, green = card-safe area.
// These guides are intentionally drawn after cards so their edges remain visible.
fn draw_cpu_bounds(g:&Game) {
    let red=Color::new(1.0,0.12,0.15,1.0);
    let orange=Color::new(1.0,0.68,0.12,1.0);
    let green=Color::new(0.1,1.0,0.5,1.0);
    let cyan=Color::new(0.3,0.85,1.0,1.0);
    for i in 1..4 {
        let p=&g.players[i];
        let panel=layout::rect(match i {1=>"cpu_1_panel",2=>"cpu_2_panel",_=>"cpu_3_panel"},
            layout::Rect{x:8.0+(i-1) as f32*636.0,y:72.0,w:632.0,h:376.0});
        draw_rectangle_lines(panel.x,panel.y,panel.w,panel.h,2.5,red);
        draw_text(&format!("CPU {} PANEL ({:.0},{:.0}) {:.0}x{:.0}",i,panel.x,panel.y,panel.w,panel.h),
            panel.x+8.0,panel.y+16.0,14.0,red);
        let group_count=if p.laid {p.table.len()}else{PHASES[p.phase.min(PHASES.len()-1)].len()};
        for j in 0..group_count.min(2) {
            let x=32.0+(i-1) as f32*632.0+j as f32*312.0;
            let y=if p.laid {250.0+9.0}else{241.0+22.0};
            let width=if group_count==1 {((1920.0-50.0)/3.0-8.0_f32).max(95.0)}else{305.0};
            let group=layout::rect(layout::cpu_group_key(i,j),layout::Rect{x,y,w:width,h:116.0});
            draw_rectangle_lines(group.x,group.y,group.w,group.h,2.0,red);
            draw_text(&format!("GROUP {} ({:.0},{:.0}) {:.0}x{:.0}",j+1,group.x,group.y,group.w,group.h),
                group.x+4.0,group.y+13.0,13.0,red);
            let margin=if has_neon_cpu_panel(){24.0}else{16.0};
            draw_rectangle_lines(group.x+margin,group.y+28.0,(group.w-2.0*margin).max(0.0),88.0,1.5,green);
            if p.laid {
                if let Some(cards)=p.table.get(j) {
                    let count=cards.len();
                    let card_w=55.0;
                    let available=(group.w-2.0*margin-card_w).max(0.0);
                    let step=if count>1 {(available/(count-1) as f32).min(card_w+1.0)}else{0.0};
                    let stack_w=card_w+count.saturating_sub(1) as f32*step;
                    let stack_x=group.x+(group.w-stack_w)/2.0;
                    let cy=group.y+if has_neon_cpu_panel(){36.0}else{26.0};
                    if count>0 {
                        draw_rectangle_lines(stack_x,cy,stack_w,84.0,2.0,orange);
                        draw_text(&format!("STACK x={:.0} w={:.0}",stack_x,stack_w),stack_x,cy+98.0,13.0,orange);
                    }
                    for (k,card) in cards.iter().enumerate(){
                        let default=layout::Rect{x:stack_x+k as f32*step,y:cy,w:card_w,h:84.0};
                        let actual=layout::rect(&format!("laid_card_{}",card.id),default);
                        draw_rectangle_lines(actual.x,actual.y,actual.w,actual.h,1.0,orange);
                    }
                }
            }
        }
    }
    draw_text("F4 BOUNDS: RED panel/group  GREEN safe region  ORANGE card/stack  |  F4 hide",
        28.0,game_canvas_height()-12.0,18.0,cyan);
}

// Desktop window dimensions are stored outside the repository, in the user's
// app-data directory. Invalid/old files safely fall back to the defaults.
mod settings;
use settings::*;
fn debug_overlay(g:&Game) {
    let (pw,ph)=(screen_width(),screen_height());
    let (mx,my)=game_mouse();
    let lines=[
        format!("FPS: {}",get_fps()),
        format!("Window: {} x {}",pw.round() as i32,ph.round() as i32),
        format!("Canvas: 1920 x 1200 (16:10, {:.2}x)",(pw/1920.0).min(ph/1200.0)),
        format!("Mouse (canvas): {:.0}, {:.0}",mx,my),
        format!("Round: {}  Turn: {}",g.round,if g.turn==0{"You".to_string()}else{format!("CPU {}",g.turn)}),
        format!("Stage: {}",match g.stage {Stage::Draw=>"Draw",Stage::Play=>"Play",Stage::Cpu=>"CPU",Stage::RoundOver=>"RoundOver",Stage::GameOver=>"GameOver"}),
        format!("Selected: {}  Dragging: {}",g.selected.len(),g.drag.is_some()),
        "F3: hide debug info".to_string(),
    ];
    let x=1490.0;let y=65.0;
    draw_rectangle(x,y,410.0,lines.len() as f32*23.0+20.0,Color::new(0.02,0.04,0.08,0.93));
    draw_rectangle_lines(x,y,410.0,lines.len() as f32*23.0+20.0,1.5,ACCENT);
    for (i,line) in lines.iter().enumerate(){draw_text(line,x+12.0,y+29.0+i as f32*23.0,18.0,WHITEISH);}
}

#[derive(Clone,Copy,PartialEq,Eq)]
enum MenuScreen { Home, Settings, DeveloperTools, UiWorkshop, HowToPlay, Rules, ConfirmNew, Playing, Resume }
fn menu_button(title:&str,y:f32)->bool {
    let (mx,my)=game_mouse();
    let hovered=mx>=730.0&&mx<=1190.0&&my>=y&&my<=y+64.0;
    themed_button(730.0,y,460.0,64.0,hovered);
    // Long theme names must fit inside the standard-width menu buttons.
    let size=if menu_text_width(title,28)>420.0 {23}else{28};
    let w=menu_text_width(title,size);
    menu_label(title,960.0-w/2.0,y+42.0,size as f32,if hovered {BG} else {WHITEISH});
    hovered&&is_mouse_button_pressed(MouseButton::Left)
}
fn draw_menu(screen:MenuScreen,has_game:bool,fullscreen:&mut bool,show_moves:&mut bool,help_scroll:&mut f32,resolution_open:&mut bool,cpu_difficulty:&mut Difficulty)->Option<MenuScreen> {
    set_default_camera();
    clear_background(active_theme().background());
    let (pw,ph)=(screen_width(),screen_height());
    let scale=(pw/1920.0).min(ph/game_canvas_height());
    set_camera(&Camera2D{target:vec2(960.0,game_canvas_height()/2.0),zoom:vec2(2.0*scale/pw,2.0*scale/ph),..Default::default()});
    menu_background();
    if matches!(screen,MenuScreen::HowToPlay|MenuScreen::Rules) {
        help_page(screen==MenuScreen::Rules,help_scroll);
        if button("BACK TO MENU",730.0,1070.0,460.0,60.0,true){return Some(MenuScreen::Home);}
        return None;
    }
    let (panel_y,panel_h)=(190.0,880.0);
    panel_decoration(620.0,panel_y,680.0,panel_h);
    let title=match screen {MenuScreen::Home=>"PHASEUP",MenuScreen::Settings=>"SETTINGS",MenuScreen::DeveloperTools=>"DEVELOPER TOOLS",MenuScreen::UiWorkshop=>"UI WORKSHOP",MenuScreen::ConfirmNew=>"NEW GAME?",MenuScreen::Playing|MenuScreen::Resume|MenuScreen::HowToPlay|MenuScreen::Rules=>""};
    // Home layout uses the same panel coordinates for every window preset.
    let title_y=match screen {MenuScreen::Home=>panel_y+115.0,MenuScreen::Settings=>260.0,_=>310.0};
    let w=menu_text_width(title,54);
    menu_label(title,960.0-w/2.0,title_y,54.0,ACCENT);
    match screen {
        MenuScreen::Home=>{
            // Keep title, buttons and bottom padding balanced on the fixed canvas.
            let first_y=panel_y+if has_game {195.0}else{225.0};
            let step=if has_game {82.0}else{85.0};
            let mut row=0.0;
            if has_game {
                if menu_button("RESUME GAME",first_y+step*row){return Some(MenuScreen::Playing);}
                row+=1.0;
            }
            if menu_button("NEW GAME",first_y+step*row){return Some(if has_game {MenuScreen::ConfirmNew} else {MenuScreen::Playing});}
            row+=1.0;
            if menu_button("HOW TO PLAY",first_y+step*row){return Some(MenuScreen::HowToPlay);}
            row+=1.0;
            if menu_button("GAME RULES",first_y+step*row){return Some(MenuScreen::Rules);}
            row+=1.0;
            if menu_button("SETTINGS",first_y+step*row){return Some(MenuScreen::Settings);}
            row+=1.0;
            if menu_button("QUIT",first_y+step*row){std::process::exit(0);}
            row+=1.0;
            if menu_button(&format!("CPU DIFFICULTY: {}",cpu_difficulty.name()),first_y+step*row){
                *cpu_difficulty=cpu_difficulty.next();
                save_cpu_difficulty(*cpu_difficulty);
            }
            if has_game {
                menu_label("Difficulty applies to new games",794.0,first_y+step*(row+1.0)+12.0,18.0,MUTED);
            }
        }
        MenuScreen::Settings=>{
            menu_label("WINDOW",750.0,323.0,22.0,MUTED);
            let (current_w,current_h)=(screen_width().round() as i32,screen_height().round() as i32);
            // Always identify a chosen resolution by its preset, even when
            // Windows fits its actual client size to the available desktop.
            let selected=read_selected_resolution().unwrap_or((current_w,current_h));
            let window_text=format!("PRESET: {} x {}  {}",selected.0,selected.1,if *resolution_open {"▲"}else{"▼"});
            let preset_button_clicked=menu_button(&window_text,365.0);
            if preset_button_clicked {
                *resolution_open=!*resolution_open;
            }
            if !*resolution_open && menu_button("TOGGLE FULLSCREEN",455.0){*fullscreen=!*fullscreen;set_fullscreen(*fullscreen);}
            menu_label("INTERFACE",750.0,558.0,22.0,MUTED);
            if !*resolution_open && menu_button(if *show_moves {"RECENT MOVES: ON"}else{"RECENT MOVES: OFF"},600.0){*show_moves=!*show_moves;}
            menu_label("APPEARANCE",750.0,705.0,22.0,MUTED);
            if !*resolution_open && menu_button(&format!("THEME: {}",active_theme().name()),745.0){
                let next=active_theme().next();set_theme(next);save_theme(next);
            }
            if !*resolution_open && menu_button("DEVELOPER TOOLS",835.0){return Some(MenuScreen::DeveloperTools);}
            if !*resolution_open && menu_button("BACK",915.0){return Some(MenuScreen::Home);}
            if has_game && !*resolution_open && menu_button("RESUME GAME",995.0){return Some(MenuScreen::Resume);}
            // Draw the expanded list last, above the other settings controls.
            if *resolution_open {
                if resolution_picker(834.0,426.0) {*resolution_open=false;}
                // Do not interpret the click that opened the picker as an outside click.
                if !preset_button_clicked && is_mouse_button_pressed(MouseButton::Left) {
                    let (mx,my)=game_mouse();
                    let on_preset_button=mx>=730.0&&mx<=1190.0&&my>=365.0&&my<=429.0;
                    let on_picker=mx>=834.0&&mx<=1086.0&&my>=426.0&&my<=752.0;
                    if !on_preset_button && !on_picker {*resolution_open=false;}
                }
            }
        }
        MenuScreen::DeveloperTools=>{
            menu_label("Visual previews do not affect gameplay.",720.0,365.0,22.0,MUTED);
            if menu_button("UI WORKSHOP",475.0){return Some(MenuScreen::UiWorkshop);}
            if menu_button("BACK TO SETTINGS",575.0){return Some(MenuScreen::Settings);}
        }
        MenuScreen::UiWorkshop=>{
            // Preview rendering happens in the main loop so it can use the workshop state.
            if menu_button("BACK TO DEVELOPER TOOLS",950.0){return Some(MenuScreen::DeveloperTools);}
        }
        MenuScreen::ConfirmNew=>{
            label("Start over? Current game progress will be lost.",675.0,310.0,22.0,WHITEISH);
            if menu_button("YES, NEW GAME",400.0){return Some(MenuScreen::Playing);}
            if menu_button("CANCEL",500.0){return Some(MenuScreen::Home);}
            if has_game && menu_button("RESUME GAME",600.0){return Some(MenuScreen::Resume);}
        }
        MenuScreen::HowToPlay|MenuScreen::Rules|MenuScreen::Playing|MenuScreen::Resume=>{}
    }
    None
}
// 16:10 resolution presets for the fixed 1920x1200 design canvas.
fn resolution_picker(x:f32,y:f32)->bool {
    let presets=[(1280,800),(1440,900),(1600,1000),(1920,1200),(2560,1600)];
    draw_rectangle(x,y,252.0,326.0,active_theme().background());
    draw_rectangle_lines(x,y,252.0,326.0,2.0,ACCENT);
    menu_label("16:10 RESOLUTIONS",x+19.0,y+27.0,19.0,ACCENT);
    let (mx,my)=game_mouse();
    for (index,(w,h)) in presets.iter().enumerate() {
        let bx=x+12.0;
        let by=y+39.0+index as f32*54.0;
        let hover=mx>=bx&&mx<=bx+228.0&&my>=by&&my<=by+48.0;
        themed_button(bx,by,228.0,48.0,hover);
        let text=format!("{} x {}",w,h);
        let tw=menu_text_width(&text,19);
        menu_label(&text,bx+(228.0-tw)/2.0,by+31.0,19.0,WHITEISH);
        if hover&&is_mouse_button_pressed(MouseButton::Left) {
            let fitted=fit_window_to_desktop(*w,*h);
            request_new_screen_size(fitted.0 as f32,fitted.1 as f32);
            save_window_size(fitted.0,fitted.1);
            save_selected_resolution(*w,*h);
            return true;
        }
    }
    false
}
// Windows window placement is independent of Macroquad's client-area sizing.
fn window_conf()->Conf {Conf{window_title:"PhaseUP - Offline Card Game".to_string(),window_width:read_window_size().0,window_height:read_window_size().1,window_resizable:true,..Default::default()}}
#[macroquad::main(window_conf)]
async fn main(){
    // Macroquad's PRNG starts with a deterministic seed unless explicitly
    // initialized. Seed once per launch before Game::new() deals the first hand.
    let seed=std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x50484153455550);
    macroquad::rand::srand(seed);
    set_theme(read_theme());
    layout::reload();
    load_menu_fonts().await;
    load_neon_cards().await;
    load_neon_ui().await;
    let mut game=Game::new();
    let mut show_debug=false;
    let (mut show_moves,mut fullscreen)=read_preferences();
    if fullscreen {set_fullscreen(true);}
    let mut saved_preferences=(show_moves,fullscreen);
    let mut menu=MenuScreen::Home;
    let mut cpu_difficulty=read_cpu_difficulty();
    let mut help_scroll=0.0f32;
    let mut resolution_open=false;
    let mut workshop_cpu=0usize;
    let mut layout_editor=layout_editor::Editor::new();
    let mut workshop_board=false;
    let mut workshop_preview=Game::new();
    workshop_preview.stage=Stage::Cpu;
    workshop_preview.turn=1;
    workshop_preview.turn_count=4;
    workshop_preview.players[1].laid=true;
    workshop_preview.players[1].table=vec![vec![Card{id:9001,kind:Kind::Number(5,0)},Card{id:9002,kind:Kind::Number(5,1)},Card{id:9003,kind:Kind::Number(5,2)}],vec![Card{id:9004,kind:Kind::Number(8,3)},Card{id:9005,kind:Kind::Number(8,0)},Card{id:9006,kind:Kind::Wild}]];
    let mut show_bounds=false;
    let mut has_game=false;
    let mut last_size=(screen_width().round() as i32,screen_height().round() as i32);
    let mut size_changed_at=get_time();
    let mut saved_size=last_size;
    let mut windowed_size=read_window_size();
    let mut previous_fullscreen=fullscreen;
    #[cfg(target_os="windows")]
    let mut center_after_resize=false;
    #[cfg(target_os="windows")]
    {
        window_placement::restore();
    }
    #[cfg(target_os="windows")]
    let mut last_position=window_placement::current();
    #[cfg(target_os="windows")]
    let mut saved_position=last_position;
    #[cfg(target_os="windows")]
    let mut position_changed_at=get_time();
    loop {
        if previous_fullscreen!=fullscreen {
            if !fullscreen {
                let (w,h)=windowed_size;
                request_new_screen_size(w as f32,h as f32);
                last_size=(w,h);
                saved_size=(w,h);
                size_changed_at=get_time();
                #[cfg(target_os="windows")]
                {center_after_resize=true;}
            }
            previous_fullscreen=fullscreen;
        }
        if is_key_pressed(KeyCode::F5) {layout::reload();}
        if is_key_pressed(KeyCode::F4) {show_bounds=!show_bounds;}
        if menu==MenuScreen::Playing && is_key_pressed(KeyCode::F3){show_debug=!show_debug;}
        if is_key_pressed(KeyCode::Escape){layout_editor.close();menu=if menu==MenuScreen::Playing {MenuScreen::Home} else if matches!(menu,MenuScreen::HowToPlay|MenuScreen::Rules|MenuScreen::Settings|MenuScreen::DeveloperTools|MenuScreen::UiWorkshop|MenuScreen::ConfirmNew) {MenuScreen::Home} else if has_game {MenuScreen::Playing} else {MenuScreen::Home};}
        // Right-click the in-game PHASEUP header or press Ctrl+0 to restore
        // the default window dimensions. The existing persistence loop saves it.
        let (mx,my)=game_mouse();
        let header_reset=is_mouse_button_pressed(MouseButton::Right)
            && mx>=12.0 && mx<=310.0 && my>=0.0 && my<=54.0;
        let shortcut_reset=is_key_pressed(KeyCode::Key0)
            && (is_key_down(KeyCode::LeftControl)||is_key_down(KeyCode::RightControl));
        if header_reset||shortcut_reset {
            let fitted=fit_window_to_desktop(1920,1200);
            request_new_screen_size(fitted.0 as f32,fitted.1 as f32);
            save_window_size(fitted.0,fitted.1);
            saved_size=fitted;
            #[cfg(target_os="windows")]
            {center_after_resize=true;}
        }
        #[cfg(target_os="windows")]
        if !fullscreen {
            let position=window_placement::current();
            if position!=last_position {
                last_position=position;
                position_changed_at=get_time();
            }
            if let Some(pos)=position {
                if Some(pos)!=saved_position && get_time()-position_changed_at>0.8 {
                    window_placement::save(pos.0,pos.1);
                    saved_position=Some(pos);
                }
            }
        }
        if (show_moves,fullscreen)!=saved_preferences {
            save_preferences(show_moves,fullscreen);
            saved_preferences=(show_moves,fullscreen);
        }
        let size=(screen_width().round() as i32,screen_height().round() as i32);
        if size!=last_size {
            last_size=size;
            size_changed_at=get_time();
            #[cfg(target_os="windows")]
            if !fullscreen {center_after_resize=true;}
        }
        // Center only after the window manager has applied the new client size.
        #[cfg(target_os="windows")]
        if center_after_resize && !fullscreen && get_time()-size_changed_at>0.25 {
            if let Some((x,y))=window_placement::center(){
                window_placement::save(x,y);
                last_position=Some((x,y));
                saved_position=Some((x,y));
                position_changed_at=get_time();
            }
            center_after_resize=false;
        }
        // Wait until resizing stops before writing, to avoid disk writes every frame.
        if size!=saved_size && get_time()-size_changed_at>1.0 && size.0>=640 && size.1>=480 {
            if !fullscreen {
                // Preserve the user's chosen window resolution and aspect ratio.
                save_window_size(size.0,size.1);
                windowed_size=size;
                saved_size=size;
            }
        }
        if menu==MenuScreen::Playing {
            has_game=true;
            game.update_cpu();
            draw_game(&mut game,show_debug,show_moves,show_bounds);
            draw_rectangle(1710.0,8.0,100.0,40.0,PANEL);
            menu_label("MENU",1724.0,35.0,22.0,ACCENT);
            let (mx,my)=game_mouse();
            if is_mouse_button_pressed(MouseButton::Left)&&mx>=1710.0&&mx<=1810.0&&my>=8.0&&my<=48.0 {menu=MenuScreen::Home;}
            if game.stage==Stage::GameOver && is_mouse_button_pressed(MouseButton::Left) && mx>=815.0 && mx<=1005.0 && my>=613.0 && my<=653.0 {menu=MenuScreen::Home;}
        } else {
            if menu==MenuScreen::UiWorkshop {
                // Tab is reserved for selecting layout elements in edit mode.
                if is_key_pressed(KeyCode::R) {
                    load_neon_cards().await;
                    load_neon_ui().await;
                }
            }
            let action=if menu==MenuScreen::UiWorkshop && workshop_board {
                draw_game(&mut workshop_preview,false,false,show_bounds);
                // Board preview is intentionally clean: testing controls live in the gallery.
                if button("BACK TO TOOLS",24.0,1120.0,240.0,54.0,true){
                    workshop_board=false;
                    layout_editor.close();
                }
                layout_editor.draw(&workshop_preview);
                None
            } else {
                draw_menu(menu,has_game,&mut fullscreen,&mut show_moves,&mut help_scroll,&mut resolution_open,&mut cpu_difficulty)
            };
            if menu==MenuScreen::UiWorkshop && !workshop_board {
                // Same 1920x1200 logical coordinates and camera as the menus.
                menu_label("CARD GALLERY",710.0,350.0,24.0,ACCENT);
                for (i,kind) in [Kind::Number(1,0),Kind::Number(2,1),Kind::Number(3,2),Kind::Number(4,3),Kind::Wild,Kind::Skip].iter().enumerate() {
                    card_ui(Card{id:i as u32,kind:*kind},704.0+i as f32*87.0,380.0,76.0,112.0,false);
                }
                cpu_card_back(704.0,520.0,76.0,112.0);
                menu_label("CPU BACK",704.0,660.0,18.0,MUTED);
                menu_label("CPU PANEL: CLICK TO CHANGE ACTIVE PLAYER",704.0,710.0,20.0,ACCENT);
                for i in 0..3 {
                    let x=700.0+i as f32*175.0;
                    neon_cpu_panel(x,730.0,165.0,140.0,if i==workshop_cpu{1.0}else{0.0},i==0);
                    menu_label(&format!("CPU {}",i+1),x+40.0,805.0,20.0,WHITEISH);
                    if inside(x,730.0,165.0,140.0)&&is_mouse_button_pressed(MouseButton::Left){workshop_cpu=i;}
                }
                // Dedicated card-test controls on the tools screen, not over the board.
                menu_label("CPU PHASE STACK TESTING",1300.0,90.0,25.0,ACCENT);
                if button(&format!("TARGET: CPU {}",workshop_cpu+1),1300.0,115.0,330.0,48.0,true){
                    workshop_cpu=(workshop_cpu+1)%3;
                }
                let target=workshop_cpu+1;
                for group_index in 0..2 {
                    let y=180.0+group_index as f32*68.0;
                    if button(&format!("GROUP {} + CARD",group_index+1),1300.0,y,245.0,54.0,true){
                        let p=&mut workshop_preview.players[target];
                        p.laid=true;
                        while p.table.len()<=group_index {p.table.push(Vec::new());}
                        let n=p.table.iter().flatten().map(|c|c.id).max().unwrap_or(40000)+1;
                        if p.table[group_index].len()<40 {
                            p.table[group_index].push(Card{id:n,kind:Kind::Number((n%12+1) as u8,(n%4) as u8)});
                        }
                    }
                    if button(&format!("GROUP {} - CARD",group_index+1),1570.0,y,245.0,54.0,true){
                        if let Some(group)=workshop_preview.players[target].table.get_mut(group_index){group.pop();}
                    }
                }
                let p=&workshop_preview.players[target];
                menu_label(&format!("CPU {} | GROUP 1: {} | GROUP 2: {}",target,p.table.first().map_or(0,Vec::len),p.table.get(1).map_or(0,Vec::len)),1300.0,348.0,20.0,WHITEISH);
                menu_label("Changes appear in FULL BOARD PREVIEW.",1300.0,380.0,18.0,MUTED);
                // Human staging and laid-phase stress tests. The mock game is isolated
                // from real gameplay; staged cards are also present in the hand.
                menu_label("HUMAN CARD TESTING",1300.0,440.0,25.0,ACCENT);
                for group_index in 0..2 {
                    let y=480.0+group_index as f32*68.0;
                    if button(&format!("STAGE {} +",group_index+1),1300.0,y,245.0,54.0,true){
                        let p=&mut workshop_preview.players[0];
                        let next=p.hand.iter().chain(p.table.iter().flatten()).map(|c|c.id).max().unwrap_or(60000).max(60000)+1;
                        if workshop_preview.staged[group_index].len()<24 {
                            p.hand.push(Card{id:next,kind:Kind::Number((next%12+1) as u8,(next%4) as u8)});
                            workshop_preview.staged[group_index].push(next);
                        }
                    }
                    if button(&format!("STAGE {} -",group_index+1),1570.0,y,245.0,54.0,true){
                        if let Some(id)=workshop_preview.staged[group_index].pop(){
                            workshop_preview.players[0].hand.retain(|c|c.id!=id);
                        }
                    }
                    let y=625.0+group_index as f32*68.0;
                    if button(&format!("LAID {} +",group_index+1),1300.0,y,245.0,54.0,true){
                        let p=&mut workshop_preview.players[0];
                        p.laid=true;
                        while p.table.len()<=group_index {p.table.push(Vec::new());}
                        let next=p.hand.iter().chain(p.table.iter().flatten()).map(|c|c.id).max().unwrap_or(70000).max(70000)+1;
                        if p.table[group_index].len()<40 {
                            p.table[group_index].push(Card{id:next,kind:Kind::Number((next%12+1) as u8,(next%4) as u8)});
                        }
                    }
                    if button(&format!("LAID {} -",group_index+1),1570.0,y,245.0,54.0,true){
                        if let Some(group)=workshop_preview.players[0].table.get_mut(group_index){group.pop();}
                    }
                }
                let human=&workshop_preview.players[0];
                menu_label(&format!("STAGED: {} / {}  |  LAID: {} / {}",
                    workshop_preview.staged[0].len(),workshop_preview.staged[1].len(),
                    human.table.first().map_or(0,Vec::len),human.table.get(1).map_or(0,Vec::len)),
                    1300.0,785.0,19.0,WHITEISH);
                menu_label("PRESS R TO RELOAD ARTWORK FROM DISK",710.0,900.0,20.0,MUTED);
                if button("FULL BOARD PREVIEW",710.0,915.0,500.0,55.0,true){workshop_board=true;}
            }
            if let Some(next)=action {
                if matches!(next,MenuScreen::HowToPlay|MenuScreen::Rules) {help_scroll=0.0;}
                if next==MenuScreen::Resume {menu=MenuScreen::Playing;}
                else {
                if next==MenuScreen::Playing && (menu==MenuScreen::ConfirmNew || !has_game) {
                    game=Game::new();
                    has_game=true;
                }
                menu=next;
                }
            }
        }
        next_frame().await;
    }
}
