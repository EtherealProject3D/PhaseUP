use macroquad::prelude::*;
use macroquad::rand::gen_range;

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
enum Stage { Draw, Play, RoundOver, GameOver }
#[derive(Clone, Copy, PartialEq, Eq)]
enum SortMode { Runs, Colors, Sets }
fn sort_key(card: Card, mode: SortMode) -> (u8,u8,u32) {
    match card.kind {
        Kind::Number(n,color) => match mode {
            SortMode::Runs => (n,color,card.id),
            SortMode::Colors => (color,n,card.id),
            SortMode::Sets => (n,color,card.id),
        },
        Kind::Wild => (250,0,card.id),
        Kind::Skip => (251,0,card.id),
    }
}
fn inside(x:f32,y:f32,w:f32,h:f32)->bool {
    let (mx,my)=game_mouse();mx>=x&&mx<=x+w&&my>=y&&my<=y+h
}
struct Game { players: Vec<Player>, deck: Vec<Card>, discard: Vec<Card>, turn: usize, stage: Stage, selected: Vec<u32>, message: String, round: u32, sort: SortMode, staged: [Vec<u32>;2], drag: Option<(u32,f32,f32)>, pile_drag: Option<(bool,f32,f32)>, drawn_card: Option<u32> }

const PHASES: [&[usize]; 10] = [
    &[3,3], &[3,4], &[4,4], &[7], &[8], &[9], &[4,4], &[7], &[5,2], &[5,3],
];
fn requirement(phase: usize, group: usize) -> &'static str {
    match (phase,group) {
        (0,_)|(6,_)|(8,_)|(9,_) => "set",
        (1,0)|(2,0) => "set",
        (1,1)|(2,1)|(3,_)|(4,_)|(5,_) => "run",
        (7,_) => "color",
        _ => "set"
    }
}
fn phase_name(p: usize) -> &'static str {
    const N: [&str;10] = ["2 sets of 3","set of 3 + run of 4","set of 4 + run of 4","run of 7","run of 8","run of 9","2 sets of 4","7 of one color","set of 5 + set of 2","set of 5 + set of 3"];
    N[p.min(9)]
}
fn card_points(c: Card) -> u32 { match c.kind { Kind::Number(n,_) if n<=9 => 5, Kind::Number(_,_) => 10, Kind::Skip => 15, Kind::Wild => 25 } }
fn color_for(k: Kind) -> Color { match k { Kind::Number(_,0) => Color::new(0.96,0.34,0.39,1.), Kind::Number(_,1) => Color::new(0.25,0.65,0.98,1.), Kind::Number(_,2) => Color::new(0.99,0.78,0.31,1.), Kind::Number(_,_) => Color::new(0.37,0.83,0.54,1.), Kind::Wild => Color::new(0.66,0.44,0.97,1.), Kind::Skip => Color::new(0.62,0.67,0.75,1.) } }
fn card_text(k: Kind) -> String { match k { Kind::Number(n,_) => n.to_string(), Kind::Wild => "W".into(), Kind::Skip => "S".into() } }
fn make_deck() -> Vec<Card> {
    let mut deck = Vec::new(); let mut id=0;
    for color in 0..4 { for n in 1..=12 { for _ in 0..2 { deck.push(Card{id,kind:Kind::Number(n,color)}); id+=1; } } }
    for _ in 0..8 { deck.push(Card{id,kind:Kind::Wild}); id+=1; }
    for _ in 0..4 { deck.push(Card{id,kind:Kind::Skip}); id+=1; }
    shuffle(&mut deck); deck
}
fn shuffle<T>(v: &mut [T]) { for i in (1..v.len()).rev() { let j=gen_range(0,i+1); v.swap(i,j); } }
fn valid_group(cards: &[Card], kind: &str) -> bool {
    if cards.is_empty() || cards.iter().any(|c| c.kind==Kind::Skip) { return false; }
    let normals: Vec<(u8,u8)> = cards.iter().filter_map(|c| if let Kind::Number(n,col)=c.kind {Some((n,col))} else {None}).collect();
    if normals.is_empty() { return false; } // require at least one natural card
    match kind {
        "set" => normals.iter().all(|(n,_)| *n==normals[0].0),
        "color" => normals.iter().all(|(_,c)| *c==normals[0].1),
        "run" => {
            let mut nums: Vec<u8> = normals.iter().map(|(n,_)|*n).collect(); nums.sort_unstable();
            if nums.windows(2).any(|w|w[0]==w[1]) { return false; }
            let length=cards.len() as u8;
            (1..=13-length).any(|start|nums.iter().all(|n|*n>=start && *n<start+length))
        }, _=>false
    }
}
fn partition_phase(cards: &[Card], phase: usize) -> Option<Vec<Vec<Card>>> {
    let sizes=PHASES[phase]; if cards.len()!=sizes.iter().sum::<usize>() {return None;}
    fn recurse(cards: &[Card], phase: usize, g: usize) -> Option<Vec<Vec<Card>>> {
        if g==PHASES[phase].len() {return if cards.is_empty(){Some(vec![])}else{None};}
        let n=cards.len(); let count=PHASES[phase][g];
        if count>n {return None;}
        // Enumerate subsets (at most 18 cards for a phase; the full hand is normally 10).
        fn choose(cards:&[Card], phase:usize,g:usize,start:usize,need:usize,indices:&mut Vec<usize>) -> Option<Vec<Vec<Card>>> {
            if need==0 {
                let mut chosen=Vec::new(); let mut rest=Vec::new();
                for (i,c) in cards.iter().enumerate() {if indices.contains(&i){chosen.push(*c)}else{rest.push(*c)}}
                if valid_group(&chosen,requirement(phase,g)) {if let Some(mut tail)=recurse(&rest,phase,g+1){let mut result=vec![chosen];result.append(&mut tail);return Some(result);}}
                return None;
            }
            if start+need>cards.len(){return None;}
            for i in start..=cards.len()-need {indices.push(i);if let Some(x)=choose(cards,phase,g,i+1,need-1,indices){return Some(x)}indices.pop();}
            None
        }
        choose(cards,phase,g,0,count,&mut Vec::new())
    }
    recurse(cards,phase,0)
}
fn can_hit(group: &[Card], card: Card, phase:usize, group_idx:usize) -> bool {
    if card.kind==Kind::Skip {return false;}
    let mut c=group.to_vec();c.push(card);
    valid_group(&c,requirement(phase,group_idx))
}
impl Game {
    fn new()->Self {
        let players=(0..4).map(|_|Player{hand:vec![],phase:0,laid:false,score:0,table:vec![]}).collect();
        let mut g=Self{players,deck:vec![],discard:vec![],turn:0,stage:Stage::Draw,selected:vec![],message:String::new(),round:0,sort:SortMode::Runs,staged:[vec![],vec![]],drag:None,pile_drag:None,drawn_card:None};g.deal();g
    }
    fn deal(&mut self) {
        self.deck=make_deck();self.discard.clear();self.selected.clear();self.staged=[vec![],vec![]];self.drag=None;self.pile_drag=None;self.drawn_card=None;self.turn=0;self.stage=Stage::Draw;self.round+=1;
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
        self.selected.retain(|v|*v!=id);
    }
    fn unstage_card(&mut self,id:u32) {
        for bucket in &mut self.staged {bucket.retain(|v|*v!=id);}
    }
    fn stage_selected(&mut self,group:usize) {
        let ids=self.selected.clone();
        for id in ids {self.stage_card(id,group);}
    }
    fn draw(&mut self,discard:bool) {
        if self.stage!=Stage::Draw||self.turn!=0{return;}
        if discard && self.discard.last().is_some_and(|c|c.kind==Kind::Skip) {self.message="Skip cards cannot be taken from discard".into();return;}
        let c=if discard {self.discard.pop()} else {self.draw_deck()};
        if let Some(c)=c {self.drawn_card=Some(c.id);self.players[0].hand.push(c);self.stage=Stage::Play;self.message="Select cards to lay your phase, or select one to discard".into();}
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
        if let Some(groups)=partition_phase(&cards,phase) {
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
            Self::remove_ids(&mut self.players[0].hand,&[card.id]);self.players[target].table[group].push(card);self.unstage_card(card.id);self.selected.clear();
            self.message=format!("Added {} to {}'s {}!",card_text(card.kind),if target==0 {"your".to_string()}else{format!("CPU {}",target)},requirement(phase,group));
            if self.players[0].hand.is_empty(){self.end_round(0);}
        }else{self.message="That card does not fit this group. For runs, add a number at either end.".into();}
    }
    fn discard_selected(&mut self) {
        if self.stage!=Stage::Play||self.selected.len()!=1{return;}
        let id=self.selected[0];let card=Self::remove_ids(&mut self.players[0].hand,&[id]);
        if card.is_empty(){return;}
        let c=card[0];self.discard.push(c);self.unstage_card(c.id);self.selected.clear();
        if self.players[0].hand.is_empty(){self.end_round(0);return;}
        if c.kind==Kind::Skip {let target=(1..4).find(|i|self.players[*i].hand.len()>0).unwrap_or(1);self.message=format!("Skip played on CPU {}",target);self.ai_cycle(Some(target));}
        else {self.ai_cycle(None);}
    }
    fn ai_cycle(&mut self,skip:Option<usize>) {
        for i in 1..4 {
            if skip==Some(i){continue;}
            self.turn=i;
            let use_discard=self.discard.last().is_some_and(|c|c.kind!=Kind::Skip && matches!(c.kind,Kind::Wild));
            let card=if use_discard{self.discard.pop()}else{self.draw_deck()};
            if let Some(c)=card{self.players[i].hand.push(c);}
            if !self.players[i].laid {
                let n=self.players[i].hand.len();let needed:usize=PHASES[self.players[i].phase].iter().sum();
                // Find any exact subset satisfying this phase.
                if needed<=n {
                    let hand=self.players[i].hand.clone();
                    let mut found=None;
                    fn search(hand:&[Card],needed:usize,start:usize,idx:&mut Vec<usize>,phase:usize)->Option<Vec<Vec<Card>>>{
                        if idx.len()==needed {let subset:Vec<Card>=idx.iter().map(|i|hand[*i]).collect();return partition_phase(&subset,phase);}
                        for j in start..hand.len() {idx.push(j);if let Some(g)=search(hand,needed,j+1,idx,phase){return Some(g);}idx.pop();}None
                    }
                    found=search(&hand,needed,0,&mut Vec::new(),self.players[i].phase);
                    if let Some(groups)=found {let ids:Vec<u32>=groups.iter().flatten().map(|c|c.id).collect();Self::remove_ids(&mut self.players[i].hand,&ids);self.players[i].table=groups;self.players[i].laid=true;}
                }
            }
            if self.players[i].laid {
                let mut j=0;
                while j<self.players[i].hand.len() {
                    let c=self.players[i].hand[j];let mut target_hit=None;
                    'outer: for target in 0..4 {if self.players[target].laid {for g in 0..self.players[target].table.len(){if can_hit(&self.players[target].table[g],c,self.players[target].phase,g){target_hit=Some((target,g));break 'outer;}}}}
                    if let Some((target,g))=target_hit {self.players[i].hand.remove(j);self.players[target].table[g].push(c);}else{j+=1;}
                }
            }
            if self.players[i].hand.is_empty(){self.end_round(i);return;}
            let idx=self.players[i].hand.iter().enumerate().max_by_key(|(_,c)|card_points(**c)).map(|(j,_)|j).unwrap();
            let c=self.players[i].hand.remove(idx);self.discard.push(c);
            if self.players[i].hand.is_empty(){self.end_round(i);return;}
        }
        self.turn=0;self.stage=Stage::Draw;self.drawn_card=None;self.message=String::new();
    }
    fn end_round(&mut self,winner:usize) {
        for p in &mut self.players {p.score+=p.hand.iter().map(|c|card_points(*c)).sum::<u32>();if p.laid{p.phase+=1;}}
        self.turn=winner;
        if self.players.iter().any(|p|p.phase>=10){self.stage=Stage::GameOver;self.message=format!("Game complete! Player {} finished all 10 phases",winner+1);}
        else{self.stage=Stage::RoundOver;self.message=format!("Player {} went out! Scores updated. Continue for next round.",winner+1);}
    }
}
// Fixed logical canvas: preserve card proportions and letterbox on narrow windows.
fn game_mouse()->(f32,f32) {
    let (pw,ph)=(screen_width(),screen_height());
    let scale=(pw/1600.0).min(ph/1000.0);
    let (mx,my)=mouse_position();
    ((mx-(pw-1600.0*scale)/2.0)/scale,(my-(ph-1000.0*scale)/2.0)/scale)
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
fn draw_game(g:&mut Game,show_debug:bool){
    clear_background(BG);
    let sw=1600.0;let sh=1000.0;
    let (pw,ph)=(screen_width(),screen_height());
    let scale=(pw/sw).min(ph/sh);
    let camera=Camera2D {
        target:vec2(sw/2.0,sh/2.0),
        zoom:vec2(2.0*scale/pw,2.0*scale/ph),
        ..Default::default()
    };
    set_camera(&camera);
    label("PHASEUP",24.,42.,32.,ACCENT);
    label("Right-click logo: reset window  |  Ctrl+0",310.,37.,16.,MUTED);
    label("v0.3.2-dev",184.,40.,19.,MUTED);
    label(&format!("ROUND {}",g.round),sw-150.,38.,22.,MUTED);
    let (turn_text,turn_color)=match g.stage {
        Stage::Draw=>("YOUR TURN  -  DRAW 1 CARD TO BEGIN",ACCENT),
        Stage::Play if g.players[0].laid=>("YOUR TURN  -  HIT OR DISCARD",YELLOW),
        Stage::Play=>("YOUR TURN  -  LAY PHASE OR DISCARD",YELLOW),
        Stage::RoundOver=>("ROUND COMPLETE",MUTED),
        Stage::GameOver=>("GAME COMPLETE",MUTED),
    };
    draw_rectangle(16.,55.,sw-32.,62.,PANEL);
    for i in 1..4 {let x=28.+(i-1) as f32*(sw-50.)/3.;let p=&g.players[i];label(&format!("CPU {}  |  Phase {}  |  {} cards",i,p.phase.min(9)+1,p.hand.len()),x,79.,19.,WHITEISH);label(&format!("Score {}{}",p.score,if p.laid{"  - LAID"}else{""}),x,104.,17.,MUTED);}
    let table_y=135.;label("TABLE",24.,table_y+18.,19.,MUTED);
    let mut hit_click=None;
    for i in 0..4 {
        let p=&g.players[i];
        if !p.laid {continue;}
        let column_w=(sw-50.0)/4.0;
        let x=24.0+i as f32*column_w;
        let y=table_y+35.0;
        label(if i==0 {"YOU"} else {match i {1=>"CPU 1",2=>"CPU 2",_=>"CPU 3"}},x,y,17.0,ACCENT);
        for (j,group) in p.table.iter().enumerate() {
            let yy=y+12.0+j as f32*108.0;
            let width=(column_w-8.0).max(95.0);
            let selected_card=if g.selected.len()==1 {g.players[0].hand.iter().find(|c|c.id==g.selected[0]).copied()}else{None};
            let can_place=g.stage==Stage::Play&&g.players[0].laid&&selected_card.is_some_and(|c|can_hit(group,c,p.phase,j));
            draw_rectangle(x,yy,width,102.0,if can_place {Color::new(0.13,0.34,0.29,1.0)}else{PANEL});
            draw_rectangle_lines(x,yy,width,102.0,if can_place {3.0}else{1.0},if can_place {ACCENT}else{MUTED});
            let desc=format!("{}  -  {} cards",requirement(p.phase,j).to_uppercase(),group.len());
            label(&desc,x+6.0,yy+17.0,15.0,WHITEISH);
            let card_w=39.0;
            let available=(width-12.0-card_w).max(0.0);
            let step=if group.len()>1 {(available/(group.len()-1) as f32).min(42.0)} else {0.0};
            for (k,card) in group.iter().enumerate() {
                let cx=x+6.0+k as f32*step;
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
    let center=sh*0.49;
    label("DRAW",sw*0.39-55.,center-48.,19.,MUTED);
    let deck_x=sw*0.39-55.;draw_rectangle(deck_x,center-36.,82.,110.,Color::new(0.19,0.39,0.52,1.));draw_rectangle_lines(deck_x,center-36.,82.,110.,if g.stage==Stage::Draw {4.0}else{2.0},if g.stage==Stage::Draw {YELLOW}else{ACCENT});label("DRAW",deck_x+12.,center+9.,19.,WHITEISH);label(&format!("{} left",g.deck.len()),deck_x+11.,center+36.,17.,WHITEISH);
    let m=game_mouse();if g.stage==Stage::Draw&&is_mouse_button_pressed(MouseButton::Left)&&inside(deck_x,center-36.,82.,110.){g.pile_drag=Some((false,m.0,m.1));}
    label("DISCARD",sw*0.56-50.,center-48.,19.,MUTED);
    let discard_x=sw*0.56-40.;
    let discard_hover=g.stage==Stage::Play&&g.drag.is_some()&&inside(discard_x,center-36.,82.,110.);
    draw_rectangle(discard_x,center-36.,82.,110.,Color::new(0.10,0.18,0.23,1.));
    draw_rectangle_lines(discard_x,center-36.,82.,110.,if discard_hover||g.stage==Stage::Draw {3.0}else{2.0},if discard_hover {GREEN}else if g.stage==Stage::Draw {YELLOW}else{ACCENT});
    if let Some(c)=g.discard.last().copied(){let _=card_ui(c,discard_x,center-36.,82.,110.,false);if g.stage==Stage::Draw&&is_mouse_button_pressed(MouseButton::Left)&&inside(discard_x,center-36.,82.,110.){g.pile_drag=Some((true,m.0,m.1));}}
    else {label("DISCARD",discard_x+5.,center+12.,17.,MUTED);label("EMPTY",discard_x+14.,center+36.,16.,MUTED);}
    // Phase staging: cards remain in the player's actual hand until played.
    let stage_y=sh-365.0;
    let groups=PHASES[g.players[0].phase];
    let stage_w=((sw-42.0)/groups.len() as f32).min(610.0);
    label(if g.stage==Stage::Draw {"PHASE STAGING  -  DRAW FIRST TO UNLOCK"}else{"PHASE STAGING  -  drag cards here, or select and press STAGE"},24.0,stage_y-10.0,18.0,if g.stage==Stage::Draw {MUTED}else{ACCENT});
    let mut staged_click=None;
    let mut stage_drop=None;
    for group in 0..groups.len() {
        let x=18.0+group as f32*(stage_w+6.0);
        let y=stage_y;
        draw_rectangle(x,y,stage_w,117.0,PANEL);
        let hovering=inside(x,y,stage_w,117.0);
        let dragging=g.drag.is_some_and(|(_,sx,sy)| {
            let (mx,my)=game_mouse();
            is_mouse_button_down(MouseButton::Left)&&(mx-sx).abs()+(my-sy).abs()>10.0
        });
        if hovering&&dragging&&g.stage==Stage::Play {draw_rectangle(x+2.0,y+2.0,stage_w-4.0,113.0,Color::new(0.12,0.43,0.39,0.65));}
        draw_rectangle_lines(x,y,stage_w,117.0,if hovering&&dragging&&g.stage==Stage::Play {3.0}else{1.5},if g.stage==Stage::Draw {MUTED}else if hovering&&dragging {YELLOW}else{ACCENT});
        let ids=g.staged[group].clone();
        let cards:Vec<Card>=ids.iter().filter_map(|id|g.players[0].hand.iter().find(|c|c.id==*id).copied()).collect();
        let ready=cards.len()==groups[group] && valid_group(&cards,requirement(g.players[0].phase,group));
        label(&format!("{} {} / {}{}",requirement(g.players[0].phase,group).to_uppercase(),cards.len(),groups[group],if ready{"  READY"}else{""}),x+10.0,y+20.0,16.0,if ready{GREEN}else{WHITEISH});
        for (i,card) in cards.iter().enumerate() {
            let cx=x+10.0+i as f32*52.0;
            if cx+46.0<x+stage_w && card_ui(*card,cx,y+29.0,45.0,76.0,false) {staged_click=Some(card.id);}
        }
        if g.stage==Stage::Play&&is_mouse_button_released(MouseButton::Left) && inside(x,y,stage_w,117.0) {stage_drop=Some(group);}
        if button(&format!("STAGE {}",group+1),x+stage_w-99.0,y-27.0,96.0,24.0,g.stage==Stage::Play&&!g.selected.is_empty()) {g.stage_selected(group);}
    }
    if let Some(id)=staged_click {g.unstage_card(id);}
    if g.stage==Stage::Draw {for group in 0..groups.len(){let x=18.0+group as f32*(stage_w+6.0);draw_rectangle(x,stage_y,stage_w,117.0,Color::new(0.04,0.07,0.10,0.72));let t="DRAW FIRST";let tw=measure_text(t,None,24,1.0).width;label(t,x+(stage_w-tw)/2.0,stage_y+67.0,24.0,MUTED);}}
    let py=sh-175.0;
    draw_rectangle(12.0,py-72.0,sw-24.0,247.0,PANEL);
    // Draw inside the hand panel, after its background and before the cards.
    // Taller, high-contrast banner; hand cards start lower to clear it.
    let guidance_y=791.0;
    draw_rectangle(415.0,guidance_y,770.0,43.0,BG);
    draw_rectangle_lines(415.0,guidance_y,770.0,43.0,2.0,turn_color);
    let tw=measure_text(turn_text,None,25,1.0).width;
    label(turn_text,800.0-tw/2.0,guidance_y+30.0,25.0,turn_color);
    let p=&g.players[0];
    label(&format!("YOUR HAND  |  Phase {}: {}  |  Score {}",p.phase.min(9)+1,phase_name(p.phase),p.score),24.0,py-46.0,21.0,WHITEISH);
    let sorts=[(SortMode::Runs,"RUNS"),(SortMode::Colors,"COLORS"),(SortMode::Sets,"SETS")];
    for (i,(mode,name)) in sorts.iter().enumerate() {
        let x=sw-338.0+i as f32*109.0;
        if button(name,x,py-65.0,103.0,28.0,true) {g.sort=*mode;}
        if g.sort==*mode {draw_rectangle_lines(x,py-65.0,103.0,28.0,2.0,ACCENT);}
    }
    let mut hand:Vec<Card>=p.hand.iter().copied().filter(|c|!g.staged.iter().any(|v|v.contains(&c.id))).collect();
    hand.sort_by_key(|c|sort_key(*c,g.sort));
    let n=hand.len();let w=((sw-48.0)/(n.max(1) as f32)).min(75.0);
    let spacing=if n>1{((sw-48.0-w)/(n-1) as f32).min(81.0)}else{0.0};
    let start=(sw-(spacing*(n.saturating_sub(1) as f32)+w))/2.0;
    let card_y=py+13.0;
    let mut toggle=None;
    for (i,c) in hand.iter().enumerate() {
        let x=start+i as f32*spacing;
        let is_dragging=g.drag.is_some_and(|(id,sx,sy)| {
            let (mx,my)=game_mouse();
            id==c.id&&is_mouse_button_down(MouseButton::Left)&&(mx-sx).abs()+(my-sy).abs()>10.0
        });
        if is_dragging {
            draw_rectangle(x,card_y,w,105.0,Color::new(0.24,0.32,0.40,0.45));
            draw_rectangle_lines(x,card_y,w,105.0,1.0,MUTED);
        } else {if card_ui(*c,x,card_y,w,105.0,g.selected.contains(&c.id)){toggle=Some(c.id);}if g.drawn_card==Some(c.id){let y=if g.selected.contains(&c.id){card_y-15.0}else{card_y};draw_rectangle_lines(x-3.0,y-3.0,w+6.0,111.0,4.0,YELLOW);label("NEW",x+5.0,y-8.0,17.0,YELLOW);}}
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
                } else if let Some(group)=stage_drop {g.stage_card(id,group);}
            }
        }
        g.drag=None;
        if let Some((from_discard,sx,sy))=g.pile_drag {
            let moved=(mx-sx).abs()+(my-sy).abs()>12.0;
            let source=if from_discard {inside(sw*0.56-40.0,center-36.0,82.0,110.0)}else{inside(deck_x,center-36.0,82.0,110.0)};
            if g.stage==Stage::Draw&&((!moved&&source)||(moved&&inside(12.0,py-72.0,sw-24.0,247.0))) {g.draw(from_discard);}
        }
        g.pile_drag=None;
    }
    if button("SELECT STAGED",24.0,sh-99.0,155.0,32.0,g.stage==Stage::Play&&g.staged.iter().any(|v|!v.is_empty())) {
        g.selected=g.staged.iter().flatten().copied().collect();
        g.message="Staged cards selected. Add other cards or lay your phase.".into();
    }
    if button("RETURN ALL",190.0,sh-99.0,138.0,32.0,g.staged.iter().any(|v|!v.is_empty())) {
        g.staged=[vec![],vec![]];
    }
    if button("LAY PHASE",24.0,sh-51.0,142.0,40.0,g.stage==Stage::Play&&!g.players[0].laid){g.lay();}
    if button("DISCARD",179.0,sh-51.0,122.0,40.0,g.stage==Stage::Play&&g.selected.len()==1){g.discard_selected();}
    if button("CLEAR",314.0,sh-51.0,104.0,40.0,!g.selected.is_empty()){g.selected.clear();}
    label(&g.message,435.0,sh-23.0,18.0,ACCENT);
    if let Some((target,group))=hit_click {g.hit(target,group);}
    if g.stage==Stage::RoundOver||g.stage==Stage::GameOver {
        draw_rectangle(0.,0.,sw,sh,Color::new(0.,0.,0.,0.77));let x=sw/2.-230.;let y=sh/2.-120.;draw_rectangle(x,y,460.,240.,PANEL);
        label(if g.stage==Stage::GameOver{"GAME COMPLETE"}else{"ROUND COMPLETE"},x+45.,y+53.,32.,ACCENT);
        for i in 0..4 {let p=&g.players[i];label(&format!("{}: phase {}  |  {} pts",if i==0{"You".into()}else{format!("CPU {}",i)},p.phase.min(10),p.score),x+40.,y+83.+i as f32*27.,20.,WHITEISH);}
        if button(if g.stage==Stage::GameOver{"NEW GAME"}else{"NEXT ROUND"},x+135.,y+191.,190.,39.,true){if g.stage==Stage::GameOver{*g=Game::new();}else{g.deal();}}
    }
    if show_debug {debug_overlay(g);}
}

// Desktop window dimensions are stored outside the repository, in the user's
// app-data directory. Invalid/old files safely fall back to the defaults.
fn window_size_path() -> std::path::PathBuf {
    let base=std::env::var_os("APPDATA").map(std::path::PathBuf::from)
        .or_else(||std::env::var_os("HOME").map(|h|std::path::PathBuf::from(h).join(".config")))
        .unwrap_or_else(||std::path::PathBuf::from("."));
    base.join("PhaseUP").join("window_size.txt")
}
fn read_window_size() -> (i32,i32) {
    if let Ok(s)=std::fs::read_to_string(window_size_path()) {
        let mut parts=s.split_whitespace();
        if let (Some(w),Some(h))=(parts.next().and_then(|v|v.parse::<i32>().ok()),parts.next().and_then(|v|v.parse::<i32>().ok())) {
            if (640..=7680).contains(&w)&&(480..=4320).contains(&h) {return (w,h);}
        }
    }
    (1600,1000)
}
fn save_window_size(w:i32,h:i32) {
    let path=window_size_path();
    if let Some(parent)=path.parent() {
        if std::fs::create_dir_all(parent).is_err(){return;}
    }
    let _=std::fs::write(path,format!("{w} {h}\n"));
}
fn debug_overlay(g:&Game) {
    let (pw,ph)=(screen_width(),screen_height());
    let (mx,my)=game_mouse();
    let lines=[
        format!("FPS: {}",get_fps()),
        format!("Window: {} x {}",pw.round() as i32,ph.round() as i32),
        format!("Canvas: 1600 x 1000 ({:.2}x)",(pw/1600.0).min(ph/1000.0)),
        format!("Mouse (canvas): {:.0}, {:.0}",mx,my),
        format!("Round: {}  Turn: {}",g.round,if g.turn==0{"You".to_string()}else{format!("CPU {}",g.turn)}),
        format!("Stage: {}",match g.stage {Stage::Draw=>"Draw",Stage::Play=>"Play",Stage::RoundOver=>"RoundOver",Stage::GameOver=>"GameOver"}),
        format!("Selected: {}  Dragging: {}",g.selected.len(),g.drag.is_some()),
        "F3: hide debug info".to_string(),
    ];
    let x=1170.0;let y=65.0;
    draw_rectangle(x,y,410.0,lines.len() as f32*23.0+20.0,Color::new(0.02,0.04,0.08,0.93));
    draw_rectangle_lines(x,y,410.0,lines.len() as f32*23.0+20.0,1.5,ACCENT);
    for (i,line) in lines.iter().enumerate(){draw_text(line,x+12.0,y+29.0+i as f32*23.0,18.0,WHITEISH);}
}

#[derive(Clone,Copy,PartialEq,Eq)]
enum MenuScreen { Home, Settings, ConfirmNew, Playing }
fn menu_button(title:&str,y:f32)->bool {
    let (mx,my)=game_mouse();
    let hovered=mx>=570.0&&mx<=1030.0&&my>=y&&my<=y+64.0;
    draw_rectangle(570.0,y,460.0,64.0,if hovered {ACCENT} else {PANEL});
    draw_rectangle_lines(570.0,y,460.0,64.0,2.0,ACCENT);
    let w=measure_text(title,None,28,1.0).width;
    label(title,800.0-w/2.0,y+42.0,28.0,if hovered {BG} else {WHITEISH});
    hovered&&is_mouse_button_pressed(MouseButton::Left)
}
fn draw_menu(screen:MenuScreen,has_game:bool,fullscreen:&mut bool)->Option<MenuScreen> {
    clear_background(BG);
    let (pw,ph)=(screen_width(),screen_height());
    let scale=(pw/1600.0).min(ph/1000.0);
    set_camera(&Camera2D{target:vec2(800.0,500.0),zoom:vec2(2.0*scale/pw,2.0*scale/ph),..Default::default()});
    draw_rectangle(460.0,125.0,680.0,760.0,PANEL);
    draw_rectangle_lines(460.0,125.0,680.0,760.0,2.0,ACCENT);
    let title=match screen {MenuScreen::Home=>"PHASEUP",MenuScreen::Settings=>"SETTINGS",MenuScreen::ConfirmNew=>"NEW GAME?",MenuScreen::Playing=>""};
    let w=measure_text(title,None,54,1.0).width;
    label(title,800.0-w/2.0,230.0,54.0,ACCENT);
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
            if menu_button("RESET SIZE (1600 x 1000)",355.0){
                request_new_screen_size(1600.0,1000.0);
                save_window_size(1600,1000);
            }
            if menu_button("TOGGLE FULLSCREEN",450.0){*fullscreen=!*fullscreen;set_fullscreen(*fullscreen);}
            if menu_button("BACK",575.0){return Some(MenuScreen::Home);}
        }
        MenuScreen::ConfirmNew=>{
            label("Start over? Current game progress will be lost.",515.0,310.0,22.0,WHITEISH);
            if menu_button("YES, NEW GAME",400.0){return Some(MenuScreen::Playing);}
            if menu_button("CANCEL",500.0){return Some(MenuScreen::Home);}
        }
        MenuScreen::Playing=>{}
    }
    None
}
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
    let mut menu=MenuScreen::Home;
    let mut has_game=false;
    let mut fullscreen=false;
    let mut last_size=(screen_width().round() as i32,screen_height().round() as i32);
    let mut size_changed_at=get_time();
    let mut saved_size=last_size;
    loop {
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
            request_new_screen_size(1600.0,1000.0);
            save_window_size(1600,1000);
            saved_size=(1600,1000);
        }
        let size=(screen_width().round() as i32,screen_height().round() as i32);
        if size!=last_size {last_size=size;size_changed_at=get_time();}
        // Wait until resizing stops before writing, to avoid disk writes every frame.
        if size!=saved_size && get_time()-size_changed_at>1.0 && size.0>=640 && size.1>=480 {
            save_window_size(size.0,size.1);
            saved_size=size;
        }
        if menu==MenuScreen::Playing {
            has_game=true;
            draw_game(&mut game,show_debug);
            draw_rectangle(1390.0,8.0,100.0,40.0,PANEL);
            label("MENU",1404.0,35.0,22.0,ACCENT);
            let (mx,my)=game_mouse();
            if is_mouse_button_pressed(MouseButton::Left)&&mx>=1390.0&&mx<=1490.0&&my>=8.0&&my<=48.0 {menu=MenuScreen::Home;}
        } else {
            let action=draw_menu(menu,has_game,&mut fullscreen);
            if let Some(next)=action {
                if next==MenuScreen::Playing && (menu==MenuScreen::ConfirmNew || !has_game) {
                    game=Game::new();
                    has_game=true;
                }
                menu=next;
            }
        }
        next_frame().await;
    }
}
