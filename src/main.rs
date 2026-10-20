// Launch as a GUI application on Windows, without a separate console window.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

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
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage { Draw, Play, Cpu, RoundOver, GameOver }
#[derive(Clone, Copy, PartialEq, Eq)]
enum SortMode { Runs, Colors, Sets }
fn inside(x:f32,y:f32,w:f32,h:f32)->bool {
    let (mx,my)=game_mouse();mx>=x&&mx<=x+w&&my>=y&&my<=y+h
}
struct Game { players: Vec<Player>, deck: Vec<Card>, discard: Vec<Card>, turn: usize, stage: Stage, selected: Vec<u32>, message: String, round: u32, sort: SortMode, staged: [Vec<u32>;2], drag: Option<(u32,f32,f32)>, pile_drag: Option<(bool,f32,f32)>, drawn_card: Option<u32>, cpu_names: [String;3], move_log: Vec<String>, turn_count:u32, staged_drag: Option<(u32,f32,f32)>, cpu_step:u8, cpu_skip:Option<usize>, cpu_next_at:f64, cpu_speed:u8, cpu_draw_flight:Option<CpuDrawFlight>, human_draw_flight:Option<HumanDrawFlight>, cpu_discard_flight:Option<CpuDiscardFlight>, human_discard_flight:Option<HumanDiscardFlight> }

mod phases;
use phases::*;
mod cards;
use cards::*;
mod ai;
use ai::*;
mod cpu;
mod animations;
use animations::*;
impl Game {
    fn new()->Self {
        let players=(0..4).map(|_|Player{hand:vec![],phase:0,laid:false,score:0,table:vec![]}).collect();
        let names=["Alex","Blake","Casey","Dakota","Emery","Finley","Harper","Jamie","Jordan","Kai","Morgan","Parker","Quinn","Reese","Riley","Rowan","Sage","Taylor"];let mut chosen=Vec::new();while chosen.len()<3 {let name=names[macroquad::rand::gen_range(0,names.len() as i32) as usize];if !chosen.contains(&name){chosen.push(name);}}let cpu_names=[chosen[0].to_string(),chosen[1].to_string(),chosen[2].to_string()];
        let mut g=Self{players,deck:vec![],discard:vec![],turn:0,stage:Stage::Draw,selected:vec![],message:String::new(),round:0,sort:SortMode::Runs,staged:[vec![],vec![]],drag:None,pile_drag:None,drawn_card:None,cpu_names,move_log:Vec::new(),turn_count:1,staged_drag:None,cpu_step:0,cpu_skip:None,cpu_next_at:0.0,cpu_speed:2,cpu_draw_flight:None,human_draw_flight:None,cpu_discard_flight:None,human_discard_flight:None};g.deal();g
    }
    fn cpu_name(&self,i:usize)->String {if i==0 {"You".to_string()}else{self.cpu_names[i-1].clone()}}
    fn log_move(&mut self,event:String){self.move_log.push(event);if self.move_log.len()>8{self.move_log.remove(0);}}
    fn deal(&mut self) {
        self.move_log.clear();
        self.deck=make_deck();self.discard.clear();self.selected.clear();self.staged=[vec![],vec![]];self.drag=None;self.staged_drag=None;self.pile_drag=None;self.drawn_card=None;self.human_draw_flight=None;self.cpu_discard_flight=None;self.human_discard_flight=None;self.cpu_draw_flight=None;self.turn=0;self.stage=Stage::Draw;self.cpu_step=0;self.cpu_skip=None;self.turn_count=1;self.round+=1;
        for p in &mut self.players {p.hand.clear();p.laid=false;p.table.clear();}
        for _ in 0..10 {for i in 0..4 {let c=self.deck.pop().unwrap();self.players[i].hand.push(c);}}
        self.discard.push(self.deck.pop().unwrap());
        self.message=String::new(); // Turn guidance is displayed in the banner above the hand.
    }
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
        if let Some(c)=c {self.drawn_card=Some(c.id);self.players[0].hand.push(c);self.human_draw_flight=if animate {Some(HumanDrawFlight{card_id:c.id,from_discard:discard,started:get_time()})}else{None};self.stage=Stage::Play;self.message="Select cards to lay your phase, or select one to discard".into();}
    }
    fn draw_deck(&mut self)->Option<Card> {
        if self.deck.is_empty() && self.discard.len()>1 {let top=self.discard.pop().unwrap();self.deck.append(&mut self.discard);shuffle(&mut self.deck);self.discard.push(top);}
        self.deck.pop()
    }
    fn remove_ids(hand:&mut Vec<Card>, ids:&[u32])->Vec<Card> {
        let mut out=Vec::new();let mut i=0;
        while i<hand.len() {if ids.contains(&hand[i].id){out.push(hand.remove(i));}else{i+=1;}}
        out
    }
    fn lay(&mut self) {
        if self.stage!=Stage::Play||self.players[0].laid{return;}
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
            Self::remove_ids(&mut self.players[0].hand,&self.selected);
            self.players[0].table=groups;self.players[0].laid=true;self.selected.clear();self.staged=[vec![],vec![]];
            self.message="Phase completed! Hit any laid phase or discard to finish.".into();
        }else{self.message=format!("Selected cards do not match phase {} ({})",phase+1,phase_name(phase));}
    }
    fn hit(&mut self, target:usize,group:usize) {
        if self.stage!=Stage::Play {self.message="Draw a card before playing onto a phase.".into();return;}
        if !self.players[0].laid {self.message="Complete your own phase before adding cards to the table.".into();return;}
        if self.selected.len()!=1 {self.message="Select exactly one card, then click a table group to add it.".into();return;}
        let Some(card)=self.players[0].hand.iter().find(|c|c.id==self.selected[0]).copied() else{return;};
        if !self.players[target].laid||group>=self.players[target].table.len(){return;}
        let phase=self.players[target].phase;
        if can_hit(&self.players[target].table[group],card,phase,group) {
            Self::remove_ids(&mut self.players[0].hand,&[card.id]);self.players[target].table[group].push(card);if requirement(phase,group)=="run"{order_run(&mut self.players[target].table[group]);}self.unstage_card(card.id);self.selected.clear();
            self.message=format!("Added {} to {}'s {}!",card_text(card.kind),if target==0 {"your".to_string()}else{format!("CPU {}",target)},requirement(phase,group));
            if self.players[0].hand.is_empty(){self.end_round(0);}
        }else{self.message="That card does not fit this group. For runs, add a number at either end.".into();}
    }
    fn discard_selected(&mut self) {
        if self.stage!=Stage::Play||self.selected.len()!=1{return;}
        let id=self.selected[0];
        // Capture the card's actual sorted hand slot before removing it.
        let mut sorted:Vec<Card>=self.players[0].hand.iter().copied()
            .filter(|c|!self.staged.iter().any(|bucket|bucket.contains(&c.id))).collect();
        sorted.sort_by_key(|c|sort_key(*c,self.sort));
        let n=sorted.len();
        let width=((1600.0-48.0)/(n.max(1) as f32)).min(75.0);
        let spacing=if n>1 {((1600.0-48.0-width)/(n-1) as f32).min(81.0)}else{0.0};
        let left=(1600.0-(spacing*(n.saturating_sub(1) as f32)+width))/2.0;
        let index=sorted.iter().position(|c|c.id==id).unwrap_or(n.saturating_sub(1));
        let from_x=left+index as f32*spacing;
        let from_y=1200.0-166.0;
        let card=Self::remove_ids(&mut self.players[0].hand,&[id]);
        if card.is_empty(){return;}
        let c=card[0];
        self.human_draw_flight=None;
        self.human_discard_flight=Some(HumanDiscardFlight{card:c,from_x,from_y,started:get_time()});
        self.discard.push(c);self.unstage_card(c.id);self.selected.clear();
        if self.players[0].hand.is_empty(){self.end_round(0);return;}
        if c.kind==Kind::Skip {let target=(1..4).find(|i|self.players[*i].hand.len()>0).unwrap_or(1);self.message=format!("Skip played on CPU {}",target);self.ai_cycle(Some(target));}
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
    fn end_round(&mut self,winner:usize) {
        for p in &mut self.players {p.score+=p.hand.iter().map(|c|card_points(*c)).sum::<u32>();if p.laid{p.phase+=1;}}
        self.turn=winner;
        self.log_move(format!("{} went out!",self.cpu_name(winner)));
        if self.players.iter().any(|p|p.phase>=10){self.stage=Stage::GameOver;self.message=format!("Game complete! {} finished all 10 phases",self.cpu_name(winner));}
        else{self.stage=Stage::RoundOver;self.message=format!("{} went out! Scores updated. Continue for next round.",self.cpu_name(winner));}
    }
}
// Fixed logical canvas: preserve card proportions and letterbox on narrow windows.
fn game_mouse()->(f32,f32) {
    let (pw,ph)=(screen_width(),screen_height());
    let scale=(pw/1600.0).min(ph/1200.0);
    let (mx,my)=mouse_position();
    ((mx-(pw-1600.0*scale)/2.0)/scale,(my-(ph-1200.0*scale)/2.0)/scale)
}
fn label(text:&str,x:f32,y:f32,size:f32,color:Color){draw_text(text,x,y,size,color);}
fn button(text:&str,x:f32,y:f32,w:f32,h:f32,enabled:bool)->bool{
    let mouse=game_mouse();let hover=mouse.0>=x&&mouse.0<=x+w&&mouse.1>=y&&mouse.1<=y+h;
    draw_rectangle(x,y,w,h,if !enabled{Color::new(0.18,0.22,0.28,1.)}else if hover{Color::new(0.22,0.66,0.61,1.)}else{Color::new(0.15,0.48,0.46,1.)});
    let t=measure_text(text,None,19,1.);label(text,x+(w-t.width)/2.,y+h/2.+7.,19.,if enabled{WHITEISH}else{MUTED});
    enabled&&hover&&is_mouse_button_pressed(MouseButton::Left)
}
fn card_ui(card:Card,x:f32,y:f32,w:f32,h:f32,selected:bool)->bool {
    let yy=if selected{y-15.}else{y};let c=color_for(card.kind);
    draw_rectangle(x+2.,yy+3.,w,h,Color::new(0.,0.,0.,0.3));draw_rectangle(x,yy,w,h,WHITEISH);
    draw_rectangle(x+5.,yy+5.,w-10.,h-10.,c);
    let t=card_text(card.kind);let size: f32=if w<45.0{23.0}else{32.0};let dims=measure_text(&t,None,size as u16,1.0);
    label(&t,x+(w-dims.width)/2.,yy+h/2.+dims.height/2.,size,if matches!(card.kind,Kind::Number(_,2)){BG}else{WHITE});
    let m=game_mouse();m.0>=x&&m.0<=x+w&&m.1>=yy&&m.1<=yy+h&&is_mouse_button_pressed(MouseButton::Left)
}
fn column_hand_width(count:usize,card_w:f32)->f32 {
    if count<=1 {0.0}else{(count-1) as f32*24.0_f32.min((450.0-card_w)/(count-1) as f32)}
}
fn cpu_card_back(x:f32,y:f32,w:f32,h:f32) {
    draw_rectangle(x+2.0,y+3.0,w,h,Color::new(0.0,0.0,0.0,0.28));
    draw_rectangle(x,y,w,h,WHITEISH);
    draw_rectangle(x+3.0,y+3.0,w-6.0,h-6.0,Color::new(0.12,0.39,0.48,1.0));
    draw_rectangle_lines(x+6.0,y+6.0,w-12.0,h-12.0,1.5,ACCENT);
    draw_line(x+8.0,y+8.0,x+w-8.0,y+h-8.0,1.0,ACCENT);
    draw_line(x+w-8.0,y+8.0,x+8.0,y+h-8.0,1.0,ACCENT);
}
fn draw_game(g:&mut Game,show_debug:bool,show_moves:bool){
    set_default_camera();
    clear_background(BG);
    let sw=1600.0;let sh=1200.0;
    let (pw,ph)=(screen_width(),screen_height());
    let scale=(pw/sw).min(ph/sh);
    let camera=Camera2D {
        target:vec2(sw/2.0,sh/2.0),
        zoom:vec2(2.0*scale/pw,2.0*scale/ph),
        ..Default::default()
    };
    set_camera(&camera);
    expire_human_flights(g);
    label("PHASEUP",24.,42.,32.,ACCENT);
    label("Right-click logo: reset window  |  Ctrl+0",310.,37.,16.,MUTED);
    label("v0.3.2-dev",184.,40.,19.,MUTED);
    label(&format!("ROUND {}  |  TURN {}",g.round,g.turn_count),sw-300.,38.,22.,MUTED);
    let (turn_text,turn_color)=match g.stage {
        Stage::Draw=>("YOUR TURN  -  DRAW 1 CARD TO BEGIN",ACCENT),
        Stage::Cpu=>("CPU TURN  -  WATCH THE ACTIONS",YELLOW),
        Stage::Play if g.players[0].laid=>("YOUR TURN  -  HIT OR DISCARD",YELLOW),
        Stage::Play=>("YOUR TURN  -  LAY PHASE OR DISCARD",YELLOW),
        Stage::RoundOver=>("ROUND COMPLETE",MUTED),
        Stage::GameOver=>("GAME COMPLETE",MUTED),
    };
    draw_rectangle(16.,55.,sw-32.,62.,PANEL);
    for i in 1..4 {let x=28.+(i-1) as f32*(sw-50.)/3.;let p=&g.players[i];if g.stage==Stage::Cpu&&g.turn==i {draw_rectangle_lines(x-5.0,53.0,470.0,172.0,3.0,YELLOW);}label(&format!("{}  |  Phase {}  |  {} cards",g.cpu_name(i),p.phase.min(9)+1,p.hand.len()),x,79.,22.,WHITEISH);label(&format!("Score {}{}",p.score,if p.laid{"  - LAID"}else{""}),x,104.,19.,MUTED);
        // Face-down fanned hand: the visible backs represent the actual hand count.
        let count=p.hand.len();
        let card_w=65.0;let card_h=87.0;
        let step=if count>1 {column_hand_width(count,card_w)/(count-1) as f32}else{0.0};
        for k in 0..count {
            if g.cpu_draw_flight.as_ref().is_some_and(|a|a.player==i && a.card_id==p.hand[k].id) {continue;}
            cpu_card_back(x+k as f32*step,119.0,card_w,card_h);
        }
    }
    let table_y=207.;label("CPU PHASES",24.,table_y+18.,19.,MUTED);
    let mut hit_click=None;
    for i in 0..4 {
        let p=&g.players[i];
        if !p.laid {continue;}
        let x=if i==0 {sw/2.0-350.0}else{24.0+(i-1) as f32*((sw-50.0)/3.0)};
        let y=if i==0 {680.0}else{table_y+35.0};
        let phase_label=if i==0 {"YOUR LAID PHASE".to_string()}else{g.cpu_name(i).to_uppercase()};
        label(&phase_label,x,y,17.0,ACCENT);
        for (j,group) in p.table.iter().enumerate() {
            let yy=y+12.0;
            let width=if i==0 {if p.table.len()==1 {700.0}else{347.0}}else{if p.table.len()==1 {((sw-50.0)/3.0-8.0).max(95.0)}else{245.0}};
            let x=if i==0 {x+j as f32*353.0}else{x+j as f32*252.0};
            let selected_card=if g.selected.len()==1 {g.players[0].hand.iter().find(|c|c.id==g.selected[0]).copied()}else{None};
            let can_place=g.stage==Stage::Play&&g.players[0].laid&&selected_card.is_some_and(|c|can_hit(group,c,p.phase,j));
            draw_rectangle(x,yy,width,102.0,if can_place {Color::new(0.13,0.34,0.29,1.0)}else{PANEL});
            draw_rectangle_lines(x,yy,width,102.0,if can_place {3.0}else{1.0},if can_place {ACCENT}else{MUTED});
            let desc=format!("{}  -  {} cards",requirement(p.phase.min(PHASES.len()-1),j).to_uppercase(),group.len());
            label(&desc,x+6.0,yy+17.0,15.0,WHITEISH);
            let card_w=39.0;
            // While dragging a valid card from the hand, preview the space it
            // will occupy and shift the existing table cards around that gap.
            let preview=g.drag.and_then(|(id,sx,sy)| {
                let (mx,my)=game_mouse();
                if !is_mouse_button_down(MouseButton::Left)
                    || (mx-sx).abs()+(my-sy).abs()<=12.0
                    || !inside(x,yy,width,102.0) || !g.players[0].laid
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
            let available=(width-12.0-card_w).max(0.0);
            let step=if shown_len>1 {(available/(shown_len-1) as f32).min(42.0)} else {0.0};
            if let Some(slot)=preview {
                let cx=x+6.0+slot as f32*step;
                draw_rectangle_lines(cx,yy+23.0,card_w,69.0,2.5,YELLOW);
            }
            for (k,card) in group.iter().enumerate() {
                let shown=k+usize::from(preview.is_some_and(|slot|k>=slot));
                let cx=x+6.0+shown as f32*step;
                let cy=yy+23.0;
                let color=color_for(card.kind);
                draw_rectangle(cx+1.0,cy+2.0,card_w,69.0,BG);
                draw_rectangle(cx,cy,card_w,69.0,WHITEISH);
                draw_rectangle(cx+3.0,cy+3.0,card_w-6.0,63.0,color);
                let value=card_text(card.kind);
                label(&value,cx+6.0,cy+27.0,23.0,if matches!(card.kind,Kind::Number(_,2)){BG}else{WHITE});
            }
            if can_place {label("CLICK TO ADD",x+7.0,yy+97.0,13.0,ACCENT);}
            if is_mouse_button_pressed(MouseButton::Left)&&inside(x,yy,width,102.0) {
                hit_click=Some((i,j));
            }
            // Dropping a dragged hand card on a laid group plays it directly.
            if is_mouse_button_released(MouseButton::Left)&&inside(x,yy,width,102.0) {
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
    // Reserve a separate central lane for draw/discard piles, away from laid groups.
    let center=535.0;
    label("DRAW",sw*0.39-55.,center-48.,19.,MUTED);
    let deck_x=sw*0.39-55.;draw_rectangle(deck_x,center-36.,82.,110.,Color::new(0.19,0.39,0.52,1.));draw_rectangle_lines(deck_x,center-36.,82.,110.,if g.stage==Stage::Draw {4.0}else{2.0},if g.stage==Stage::Draw {YELLOW}else{ACCENT});label("DRAW",deck_x+12.,center+9.,19.,WHITEISH);label(&format!("{} left",g.deck.len()),deck_x+11.,center+36.,17.,WHITEISH);
    let m=game_mouse();if g.stage==Stage::Draw&&is_mouse_button_pressed(MouseButton::Left)&&inside(deck_x,center-36.,82.,110.){g.pile_drag=Some((false,m.0,m.1));}
    label("DISCARD",sw*0.56-50.,center-48.,19.,MUTED);
    let discard_x=sw*0.56-40.;
    let discard_hover=g.stage==Stage::Play&&g.drag.is_some()&&inside(discard_x,center-36.,82.,110.);
    draw_rectangle(discard_x,center-36.,82.,110.,Color::new(0.10,0.18,0.23,1.));
    draw_rectangle_lines(discard_x,center-36.,82.,110.,if discard_hover||g.stage==Stage::Draw {3.0}else{2.0},if discard_hover {GREEN}else if g.stage==Stage::Draw {YELLOW}else{ACCENT});
    if let Some(c)=g.discard.last().copied(){if !g.cpu_discard_flight.as_ref().is_some_and(|a|a.card.id==c.id) && !g.human_discard_flight.as_ref().is_some_and(|a|a.card.id==c.id){let _=card_ui(c,discard_x,center-36.,82.,110.,false);}if g.stage==Stage::Draw&&is_mouse_button_pressed(MouseButton::Left)&&inside(discard_x,center-36.,82.,110.){g.pile_drag=Some((true,m.0,m.1));}}
    else {label("DISCARD",discard_x+5.,center+12.,17.,MUTED);label("EMPTY",discard_x+14.,center+36.,16.,MUTED);}
    // Recent CPU actions stay visible after their turns resolve instantly.
    if show_moves {
    let log_x=1198.0;let log_y=480.0;
    draw_rectangle(log_x,log_y,374.0,207.0,PANEL);
    draw_rectangle_lines(log_x,log_y,374.0,207.0,1.5,ACCENT);
    label("RECENT MOVES",log_x+12.0,log_y+25.0,19.0,ACCENT);
    // Put the live action *inside* the panel so laid phases cannot cover it.
    let events_y=if g.stage==Stage::Cpu {
        let action=match g.cpu_step {0=>"DRAWING",1=>"CHECKING PHASE",2=>"HITTING TABLE",_=>"DISCARDING"};
        let notice=format!("{} - {}",g.cpu_name(g.turn.min(3)).to_uppercase(),action);
        draw_rectangle(log_x+9.0,log_y+34.0,356.0,36.0,BG);
        draw_rectangle_lines(log_x+9.0,log_y+34.0,356.0,36.0,1.5,YELLOW);
        let width=measure_text(&notice,None,18,1.0).width;
        label(&notice,log_x+187.0-width/2.0,log_y+59.0,18.0,YELLOW);
        log_y+94.0
    }else{log_y+50.0};
    let max_events=if g.stage==Stage::Cpu {5}else{7};
    for (j,event) in g.move_log.iter().rev().take(max_events).enumerate(){
        label(event,log_x+12.0,events_y+j as f32*23.0,16.0,WHITEISH);
    }
    }
    // Phase staging: cards remain in the player's actual hand until played.
    let stage_y=sh-347.0;
    // The completed player advances to phase index 10, beyond the ten phase definitions.
    // Keep rendering the final phase while the GameOver overlay is displayed.
    let groups=PHASES[g.players[0].phase.min(PHASES.len()-1)];
    let stage_w=if groups.len()==1 {700.0}else{540.0};
    let stage_left=(sw-(groups.len() as f32*stage_w+(groups.len().saturating_sub(1)) as f32*6.0))/2.0;
    let mut staged_click=None;
    let mut stage_drop=None;
    let mut staged_reorder=None;
    for group in 0..groups.len() {
        let x=stage_left+group as f32*(stage_w+6.0);
        let y=stage_y;
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
        label(&format!("STAGE {} {} / {}{}",requirement(g.players[0].phase.min(PHASES.len()-1),group).to_uppercase(),cards.len(),groups[group],if ready{"  READY"}else{""}),x+10.0,y+20.0,16.0,if ready{GREEN}else{WHITEISH});
        let drag_preview=g.staged_drag.or(g.drag).and_then(|(id,sx,sy)|{
            let (mx,my)=game_mouse();
            if is_mouse_button_down(MouseButton::Left)
                && (mx-sx).abs()+(my-sy).abs()>10.0
                && inside(x,y,stage_w,117.0) {
                let slot=(((mx-(x+10.0)+26.0)/52.0).floor().max(0.0)) as usize;
                Some((id,slot))
            } else {None}
        });
        let visible_cards:Vec<Card>=cards.iter().copied()
            .filter(|card|drag_preview.is_none_or(|(id,_)|card.id!=id)).collect();
        let insert_at=drag_preview.map(|(_,slot)|slot.min(visible_cards.len()));
        if let Some(slot)=insert_at {
            let gap_x=x+10.0+slot as f32*52.0;
            draw_rectangle_lines(gap_x,y+29.0,45.0,76.0,2.0,YELLOW);
        }
        for (i,card) in visible_cards.iter().enumerate() {
            let shifted=i+usize::from(insert_at.is_some_and(|slot|i>=slot));
            let cx=x+10.0+shifted as f32*52.0;
            if cx+46.0<x+stage_w {
                let floating=g.staged_drag.is_some_and(|(id,sx,sy)|{
                    let (mx,my)=game_mouse();
                    id==card.id&&is_mouse_button_down(MouseButton::Left)&&(mx-sx).abs()+(my-sy).abs()>10.0
                });
                if floating {
                    draw_rectangle(cx,y+29.0,45.0,76.0,Color::new(0.24,0.32,0.40,0.45));
                    draw_rectangle_lines(cx,y+29.0,45.0,76.0,1.0,MUTED);
                } else {let _=card_ui(*card,cx,y+29.0,45.0,76.0,false);}
                if g.stage==Stage::Play&&is_mouse_button_pressed(MouseButton::Left)&&inside(cx,y+29.0,45.0,76.0){g.staged_drag=Some((card.id,game_mouse().0,game_mouse().1));}
            }
        }
        if g.stage==Stage::Play&&is_mouse_button_released(MouseButton::Left) && inside(x,y,stage_w,117.0) {
            stage_drop=Some(group);
            if let Some((id,sx,sy))=g.staged_drag {
                let (mx,my)=game_mouse();
                if (mx-sx).abs()+(my-sy).abs()>10.0 {
                    let slot=(((mx-(x+10.0)+26.0)/52.0).floor().max(0.0)) as usize;
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
            && !cards.iter().enumerate().any(|(j,_)|inside(x+10.0+j as f32*52.0,y+30.0,45.0,78.0))
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
    if g.stage==Stage::Draw && g.turn_count==1 {for group in 0..groups.len(){let x=stage_left+group as f32*(stage_w+6.0);draw_rectangle(x,stage_y,stage_w,117.0,Color::new(0.04,0.07,0.10,0.72));let t="DRAW FIRST";let tw=measure_text(t,None,24,1.0).width;label(t,x+(stage_w-tw)/2.0,stage_y+67.0,24.0,MUTED);}}
    let py=sh-157.0;
    draw_rectangle(12.0,py-72.0,sw-24.0,217.0,PANEL);
    // Draw inside the hand panel, after its background and before the cards.
    // Taller, high-contrast banner; hand cards start lower to clear it.
    let guidance_y=806.0;
    draw_rectangle(415.0,guidance_y,770.0,43.0,BG);
    draw_rectangle_lines(415.0,guidance_y,770.0,43.0,2.0,turn_color);
    let tw=measure_text(turn_text,None,25,1.0).width;
    label(turn_text,800.0-tw/2.0,guidance_y+30.0,25.0,turn_color);
    let p=&g.players[0];
    let hand_info=format!("YOUR HAND  |  Phase {}: {}  |  Score {}",p.phase.min(9)+1,phase_name(p.phase),p.score);
    let hand_info_width=measure_text(&hand_info,None,21,1.0).width;
    label(&hand_info,(sw-hand_info_width)/2.0,py-46.0,21.0,WHITEISH);
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
    let mut toggle=None;
    for (i,c) in hand.iter().enumerate() {
        let x=start+i as f32*spacing;
        if g.human_draw_flight.as_ref().is_some_and(|a|a.card_id==c.id) {continue;}
        let is_dragging=g.drag.is_some_and(|(id,sx,sy)| {
            let (mx,my)=game_mouse();
            id==c.id&&is_mouse_button_down(MouseButton::Left)&&(mx-sx).abs()+(my-sy).abs()>10.0
        });
        if is_dragging {
            draw_rectangle(x,card_y,w,105.0,Color::new(0.24,0.32,0.40,0.45));
            draw_rectangle_lines(x,card_y,w,105.0,1.0,MUTED);
        } else {if card_ui(*c,x,card_y,w,105.0,g.selected.contains(&c.id)){toggle=Some(c.id);}if g.drawn_card==Some(c.id){let y=if g.selected.contains(&c.id){card_y-15.0}else{card_y};draw_rectangle_lines(x-3.0,y-3.0,w+6.0,111.0,4.0,YELLOW);}}
        if is_mouse_button_pressed(MouseButton::Left)&&inside(x,if g.selected.contains(&c.id){card_y-15.0}else{card_y},w,105.0){g.drag=Some((c.id,game_mouse().0,game_mouse().1));}
    }
    if let Some(id)=toggle {if g.selected.contains(&id){g.selected.retain(|x|*x!=id);}else{g.selected.push(id);}}
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
                if g.stage==Stage::Play&&inside(sw*0.56-44.0,center-40.0,90.0,118.0) {
                    g.selected=vec![id];
                    g.discard_selected();
                    // Drag-and-drop is already a direct physical movement.
                    // Only button-triggered discards should fly.
                    g.human_discard_flight=None;
                } else if let Some(group)=stage_drop {
                    let x=stage_left+group as f32*(stage_w+6.0);
                    let index=(((mx-(x+10.0)+26.0)/52.0).floor().max(0.0)) as usize;
                    g.stage_card_at(id,group,index);
                }
            }
        }
        g.drag=None;
        if let Some((from_discard,sx,sy))=g.pile_drag {
            let moved=(mx-sx).abs()+(my-sy).abs()>12.0;
            let source=if from_discard {inside(sw*0.56-40.0,center-36.0,82.0,110.0)}else{inside(deck_x,center-36.0,82.0,110.0)};
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
        draw_rectangle(525.0,772.0,550.0,31.0,Color::new(0.08,0.28,0.22,1.0));
        draw_rectangle_lines(525.0,772.0,550.0,31.0,1.5,GREEN);
        label("PHASE COMPLETE",545.0,794.0,21.0,GREEN);
        if button("LAY PHASE",852.0,774.0,206.0,27.0,true){
            g.selected=g.staged.iter().flatten().copied().collect();
            g.lay();
        }
    }
    // Hand-area controls match the mockup: clears in the left margin,
    // discard at the right, and sorting buttons remain centered.
    // Position the clear controls immediately left of the phase staging slots.
    // Two-group phases have a 254px gutter; one-group phases have a 450px gutter.
    let control_w=154.0;
    let control_h=30.0;
    // Anchor the controls to the two-group layout so single-group phases
    // keep the same horizontal position even though their staging box is narrower.
    let two_group_left=(sw-(2.0*540.0+6.0))/2.0;
    let control_x=(two_group_left-control_w)/2.0;
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
    for discard_x in [1600.0-1050.0-discard_width,1050.0] {
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
        let title_w=measure_text(&title,None,32,1.0).width;
        label(&title,x+(530.0-title_w)/2.0,y+47.,32.,ACCENT);
        let winner_name=g.cpu_name(g.turn).to_uppercase();
        let announcement=if game_over {if g.turn==0 {"YOU FINISHED THE GAME!".to_string()}else{format!("{} FINISHED THE GAME!",winner_name)}}else if g.turn==0 {"YOU WIN THE ROUND!".to_string()}else{format!("{} WINS THE ROUND!",winner_name)};
        let announce_w=measure_text(&announcement,None,27,1.0).width;
        label(&announcement,x+(530.0-announce_w)/2.0,y+91.,27.,YELLOW);
        label("TOTAL SCORES",x+29.,y+128.,18.,MUTED);
        for i in 0..4 {
            let p=&g.players[i];
            let name=g.cpu_name(i);
            let line=format!("{}{}  |  Phase {}  |  {} pts",name,if i==g.turn {"  (OUT)"}else{""},p.phase.min(10),p.score);
            label(&line,x+29.,y+159.+i as f32*29.,20.,if i==g.turn {YELLOW}else{WHITEISH});
        }
        if game_over {
            if button("NEW GAME",x+60.,y+277.,190.,40.,true){*g=Game::new();}
            let _=button("MAIN MENU",x+280.,y+277.,190.,40.,true);
        } else if button("NEXT ROUND",x+170.,y+277.,190.,40.,true){g.deal();}
    }
    draw_card_flights(g,sw,sh);
    if show_debug {debug_overlay(g);}
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
        format!("Canvas: 1600 x 1000 ({:.2}x)",(pw/1600.0).min(ph/1200.0)),
        format!("Mouse (canvas): {:.0}, {:.0}",mx,my),
        format!("Round: {}  Turn: {}",g.round,if g.turn==0{"You".to_string()}else{format!("CPU {}",g.turn)}),
        format!("Stage: {}",match g.stage {Stage::Draw=>"Draw",Stage::Play=>"Play",Stage::Cpu=>"CPU",Stage::RoundOver=>"RoundOver",Stage::GameOver=>"GameOver"}),
        format!("Selected: {}  Dragging: {}",g.selected.len(),g.drag.is_some()),
        "F3: hide debug info".to_string(),
    ];
    let x=1170.0;let y=65.0;
    draw_rectangle(x,y,410.0,lines.len() as f32*23.0+20.0,Color::new(0.02,0.04,0.08,0.93));
    draw_rectangle_lines(x,y,410.0,lines.len() as f32*23.0+20.0,1.5,ACCENT);
    for (i,line) in lines.iter().enumerate(){draw_text(line,x+12.0,y+29.0+i as f32*23.0,18.0,WHITEISH);}
}

#[derive(Clone,Copy,PartialEq,Eq)]
enum MenuScreen { Home, Settings, ConfirmNew, Playing, Resume }
fn menu_button(title:&str,y:f32)->bool {
    let (mx,my)=game_mouse();
    let hovered=mx>=570.0&&mx<=1030.0&&my>=y&&my<=y+64.0;
    draw_rectangle(570.0,y,460.0,64.0,if hovered {ACCENT} else {PANEL});
    draw_rectangle_lines(570.0,y,460.0,64.0,2.0,ACCENT);
    let w=measure_text(title,None,28,1.0).width;
    label(title,800.0-w/2.0,y+42.0,28.0,if hovered {BG} else {WHITEISH});
    hovered&&is_mouse_button_pressed(MouseButton::Left)
}
fn draw_menu(screen:MenuScreen,has_game:bool,fullscreen:&mut bool,show_moves:&mut bool)->Option<MenuScreen> {
    set_default_camera();
    clear_background(BG);
    let (pw,ph)=(screen_width(),screen_height());
    let scale=(pw/1600.0).min(ph/1200.0);
    set_camera(&Camera2D{target:vec2(800.0,600.0),zoom:vec2(2.0*scale/pw,2.0*scale/ph),..Default::default()});
    draw_rectangle(460.0,225.0,680.0,760.0,PANEL);
    draw_rectangle_lines(460.0,225.0,680.0,760.0,2.0,ACCENT);
    let title=match screen {MenuScreen::Home=>"PHASEUP",MenuScreen::Settings=>"SETTINGS",MenuScreen::ConfirmNew=>"NEW GAME?",MenuScreen::Playing|MenuScreen::Resume=>""};
    let w=measure_text(title,None,54,1.0).width;
    label(title,800.0-w/2.0,310.0,54.0,ACCENT);
    match screen {
        MenuScreen::Home=>{
            if has_game && menu_button("RESUME GAME",310.0){return Some(MenuScreen::Playing);}
            if menu_button("NEW GAME",400.0){return Some(if has_game {MenuScreen::ConfirmNew} else {MenuScreen::Playing});}
            if menu_button("SETTINGS",490.0){return Some(MenuScreen::Settings);}
            if menu_button("QUIT",580.0){std::process::exit(0);}
            if has_game { label("ESC: resume game",690.0,750.0,20.0,MUTED); }
        }
        MenuScreen::Settings=>{
            label("WINDOW",590.0,300.0,26.0,WHITEISH);
            if menu_button("RESET SIZE (1600 x 1200)",355.0){
                request_new_screen_size(1600.0,1200.0);
                save_window_size(1600,1200);
            }
            if menu_button("TOGGLE FULLSCREEN",450.0){*fullscreen=!*fullscreen;set_fullscreen(*fullscreen);}
            label("INTERFACE",590.0,555.0,26.0,WHITEISH);
            if menu_button(if *show_moves {"RECENT MOVES: ON"}else{"RECENT MOVES: OFF"},595.0){*show_moves=!*show_moves;}
            if menu_button("BACK",705.0){return Some(MenuScreen::Home);}
            if has_game && menu_button("RESUME GAME",790.0){return Some(MenuScreen::Resume);}
        }
        MenuScreen::ConfirmNew=>{
            label("Start over? Current game progress will be lost.",515.0,310.0,22.0,WHITEISH);
            if menu_button("YES, NEW GAME",400.0){return Some(MenuScreen::Playing);}
            if menu_button("CANCEL",500.0){return Some(MenuScreen::Home);}
            if has_game && menu_button("RESUME GAME",600.0){return Some(MenuScreen::Resume);}
        }
        MenuScreen::Playing|MenuScreen::Resume=>{}
    }
    None
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
    let mut game=Game::new();
    let mut show_debug=false;
    let (mut show_moves,mut fullscreen)=read_preferences();
    if fullscreen {set_fullscreen(true);}
    let mut saved_preferences=(show_moves,fullscreen);
    let mut menu=MenuScreen::Home;
    let mut has_game=false;
    let mut last_size=(screen_width().round() as i32,screen_height().round() as i32);
    let mut size_changed_at=get_time();
    let mut saved_size=last_size;
    let mut windowed_size=read_window_size();
    let mut previous_fullscreen=fullscreen;
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
    // Normalize the drawable client area to 4:3 at launch, including when
    // Windows restores a window with a different client-area ratio.
    if !fullscreen && last_size.0>0 && last_size.1>0 {
        let target_h=((last_size.0 as f32)*12.0/16.0).round() as i32;
        if (last_size.1-target_h).abs()>2 {
            request_new_screen_size(last_size.0 as f32,target_h as f32);
            saved_size=(last_size.0,target_h);
        }
    }
    loop {
        if previous_fullscreen!=fullscreen {
            if !fullscreen {
                let (w,h)=windowed_size;
                request_new_screen_size(w as f32,h as f32);
                last_size=(w,h);
                saved_size=(w,h);
                size_changed_at=get_time();
            }
            previous_fullscreen=fullscreen;
        }
        if menu==MenuScreen::Playing && is_key_pressed(KeyCode::F3){show_debug=!show_debug;}
        if is_key_pressed(KeyCode::Escape){menu=if menu==MenuScreen::Playing {MenuScreen::Home} else if has_game {MenuScreen::Playing} else {MenuScreen::Home};}
        // Right-click the in-game PHASEUP header or press Ctrl+0 to restore
        // the default window dimensions. The existing persistence loop saves it.
        let (mx,my)=game_mouse();
        let header_reset=is_mouse_button_pressed(MouseButton::Right)
            && mx>=12.0 && mx<=310.0 && my>=0.0 && my<=54.0;
        let shortcut_reset=is_key_pressed(KeyCode::Key0)
            && (is_key_down(KeyCode::LeftControl)||is_key_down(KeyCode::RightControl));
        if header_reset||shortcut_reset {
            request_new_screen_size(1600.0,1200.0);
            save_window_size(1600,1200);
            saved_size=(1600,1200);
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
        if size!=last_size {last_size=size;size_changed_at=get_time();}
        // Wait until resizing stops before writing, to avoid disk writes every frame.
        if size!=saved_size && get_time()-size_changed_at>1.0 && size.0>=640 && size.1>=480 {
            if !fullscreen {
                let fitted=fit_window_to_desktop(size.0,size.1);
                let scale=(fitted.0 as f32/size.0 as f32).min(fitted.1 as f32/size.1 as f32);
                let fit_w=((size.0 as f32*scale).round() as i32).max(640);
                let fit_h=((fit_w as f32)*0.75).round() as i32;
                if (size.0-fit_w).abs()>2 || (size.1-fit_h).abs()>2 {
                    request_new_screen_size(fit_w as f32,fit_h as f32);
                    saved_size=(fit_w,fit_h);
                } else {
                    save_window_size(size.0,size.1);
                    windowed_size=size;
                    saved_size=size;
                }
            }
        }
        if menu==MenuScreen::Playing {
            has_game=true;
            game.update_cpu();
            draw_game(&mut game,show_debug,show_moves);
            draw_rectangle(1390.0,8.0,100.0,40.0,PANEL);
            label("MENU",1404.0,35.0,22.0,ACCENT);
            let (mx,my)=game_mouse();
            if is_mouse_button_pressed(MouseButton::Left)&&mx>=1390.0&&mx<=1490.0&&my>=8.0&&my<=48.0 {menu=MenuScreen::Home;}
            if game.stage==Stage::GameOver && is_mouse_button_pressed(MouseButton::Left) && mx>=815.0 && mx<=1005.0 && my>=613.0 && my<=653.0 {menu=MenuScreen::Home;}
        } else {
            let action=draw_menu(menu,has_game,&mut fullscreen,&mut show_moves);
            if let Some(next)=action {
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
