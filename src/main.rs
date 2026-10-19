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
    let (mx,my)=mouse_position();mx>=x&&mx<=x+w&&my>=y&&my<=y+h
}
struct Game { players: Vec<Player>, deck: Vec<Card>, discard: Vec<Card>, turn: usize, stage: Stage, selected: Vec<u32>, message: String, round: u32, sort: SortMode, staged: [Vec<u32>;2], drag: Option<(u32,f32,f32)> }

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
        let mut g=Self{players,deck:vec![],discard:vec![],turn:0,stage:Stage::Draw,selected:vec![],message:String::new(),round:0,sort:SortMode::Runs,staged:[vec![],vec![]],drag:None};g.deal();g
    }
    fn deal(&mut self) {
        self.deck=make_deck();self.discard.clear();self.selected.clear();self.staged=[vec![],vec![]];self.drag=None;self.turn=0;self.stage=Stage::Draw;self.round+=1;
        for p in &mut self.players {p.hand.clear();p.laid=false;p.table.clear();}
        for _ in 0..10 {for i in 0..4 {let c=self.deck.pop().unwrap();self.players[i].hand.push(c);}}
        self.discard.push(self.deck.pop().unwrap());
        self.message=format!("Round {} — draw from deck or discard pile",self.round);
    }
    fn stage_card(&mut self,id:u32,group:usize) {
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
        if let Some(c)=c {self.players[0].hand.push(c);self.stage=Stage::Play;self.message="Select cards to lay your phase, or select one to discard".into();}
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
        if self.stage!=Stage::Play||!self.players[0].laid||self.selected.len()!=1{return;}
        let Some(card)=self.players[0].hand.iter().find(|c|c.id==self.selected[0]).copied() else{return;};
        if !self.players[target].laid||group>=self.players[target].table.len(){return;}
        let phase=self.players[target].phase;
        if can_hit(&self.players[target].table[group],card,phase,group) {
            Self::remove_ids(&mut self.players[0].hand,&[card.id]);self.players[target].table[group].push(card);self.unstage_card(card.id);self.selected.clear();self.message="Card hit successfully".into();
        }else{self.message="That card cannot be added to this group".into();}
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
        self.turn=0;self.stage=Stage::Draw;self.message="Your turn — draw a card".into();
    }
    fn end_round(&mut self,winner:usize) {
        for p in &mut self.players {p.score+=p.hand.iter().map(|c|card_points(*c)).sum::<u32>();if p.laid{p.phase+=1;}}
        self.turn=winner;
        if self.players.iter().any(|p|p.phase>=10){self.stage=Stage::GameOver;self.message=format!("Game complete! Player {} finished all 10 phases",winner+1);}
        else{self.stage=Stage::RoundOver;self.message=format!("Player {} went out! Scores updated. Continue for next round.",winner+1);}
    }
}
fn label(text:&str,x:f32,y:f32,size:f32,color:Color){draw_text(text,x,y,size,color);}
fn button(text:&str,x:f32,y:f32,w:f32,h:f32,enabled:bool)->bool{
    let mouse=mouse_position();let hover=mouse.0>=x&&mouse.0<=x+w&&mouse.1>=y&&mouse.1<=y+h;
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
    let m=mouse_position();m.0>=x&&m.0<=x+w&&m.1>=yy&&m.1<=yy+h&&is_mouse_button_pressed(MouseButton::Left)
}
fn draw_game(g:&mut Game){
    clear_background(BG);
    let sw=screen_width();let sh=screen_height();
    label("PHASEUP",24.,42.,32.,ACCENT);
    label(&format!("ROUND {}",g.round),sw-150.,38.,22.,MUTED);
    draw_rectangle(16.,55.,sw-32.,62.,PANEL);
    for i in 1..4 {let x=28.+(i-1) as f32*(sw-50.)/3.;let p=&g.players[i];label(&format!("CPU {}  |  Phase {}  |  {} cards",i,p.phase.min(9)+1,p.hand.len()),x,79.,19.,WHITEISH);label(&format!("Score {}{}",p.score,if p.laid{"  - LAID"}else{""}),x,104.,17.,MUTED);}
    let table_y=135.;label("TABLE",24.,table_y+18.,19.,MUTED);
    let mut hit_click=None;
    for i in 0..4 {let p=&g.players[i];if !p.laid{continue;}let x=30.+(i as f32)*((sw-50.)/4.);let y=table_y+35.;label(&format!("{}",if i==0{"YOU".into()}else{format!("CPU {}",i)}),x,y,17.,ACCENT);
        for (j,group) in p.table.iter().enumerate(){let yy=y+12.+j as f32*43.;draw_rectangle(x,yy,((sw-70.)/4.).min(275.),39.,PANEL);let desc=format!("{}: {} cards",requirement(p.phase,j),group.len());label(&desc,x+9.,yy+25.,17.,WHITEISH);let m=mouse_position();if m.0>=x&&m.0<=x+((sw-70.)/4.).min(275.)&&m.1>=yy&&m.1<=yy+39.&&is_mouse_button_pressed(MouseButton::Left){hit_click=Some((i,j));}}
    }
    let center=sh*0.49;
    label("DRAW",sw*0.39-55.,center-48.,19.,MUTED);
    let deck_x=sw*0.39-55.;draw_rectangle(deck_x,center-36.,82.,110.,Color::new(0.19,0.39,0.52,1.));draw_rectangle_lines(deck_x+6.,center-30.,70.,98.,2.,ACCENT);label(&format!("{}",g.deck.len()),deck_x+27.,center+22.,25.,WHITEISH);
    let m=mouse_position();if g.stage==Stage::Draw&&m.0>=deck_x&&m.0<=deck_x+82.&&m.1>=center-36.&&m.1<=center+74.&&is_mouse_button_pressed(MouseButton::Left){g.draw(false);}
    label("DISCARD",sw*0.56-50.,center-48.,19.,MUTED);
    if let Some(c)=g.discard.last().copied(){let x=sw*0.56-40.;if card_ui(c,x,center-36.,82.,110.,false)&&g.stage==Stage::Draw{g.draw(true);}}
    // Phase staging: cards remain in the player's actual hand until played.
    let stage_y=sh-365.0;
    let groups=PHASES[g.players[0].phase];
    let stage_w=((sw-42.0)/groups.len() as f32).min(610.0);
    label("PHASE STAGING  -  drag cards here, or select and press STAGE",24.0,stage_y-10.0,18.0,ACCENT);
    let mut staged_click=None;
    let mut stage_drop=None;
    for group in 0..groups.len() {
        let x=18.0+group as f32*(stage_w+6.0);
        let y=stage_y;
        draw_rectangle(x,y,stage_w,117.0,PANEL);
        draw_rectangle_lines(x,y,stage_w,117.0,1.5,ACCENT);
        let ids=g.staged[group].clone();
        let cards:Vec<Card>=ids.iter().filter_map(|id|g.players[0].hand.iter().find(|c|c.id==*id).copied()).collect();
        let ready=cards.len()==groups[group] && valid_group(&cards,requirement(g.players[0].phase,group));
        label(&format!("{} {} / {}{}",requirement(g.players[0].phase,group).to_uppercase(),cards.len(),groups[group],if ready{"  READY"}else{""}),x+10.0,y+20.0,16.0,if ready{GREEN}else{WHITEISH});
        for (i,card) in cards.iter().enumerate() {
            let cx=x+10.0+i as f32*52.0;
            if cx+46.0<x+stage_w && card_ui(*card,cx,y+29.0,45.0,76.0,false) {staged_click=Some(card.id);}
        }
        if is_mouse_button_released(MouseButton::Left) && inside(x,y,stage_w,117.0) {stage_drop=Some(group);}
        if button(&format!("STAGE {}",group+1),x+stage_w-99.0,y-27.0,96.0,24.0,!g.selected.is_empty()) {g.stage_selected(group);}
    }
    if let Some(id)=staged_click {g.unstage_card(id);}
    let py=sh-175.0;
    draw_rectangle(12.0,py-72.0,sw-24.0,247.0,PANEL);
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
    let mut toggle=None;
    for (i,c) in hand.iter().enumerate() {
        let x=start+i as f32*spacing;
        if card_ui(*c,x,py,w,105.0,g.selected.contains(&c.id)){toggle=Some(c.id);}
        if is_mouse_button_pressed(MouseButton::Left)&&inside(x,py-15.0,w,120.0){g.drag=Some((c.id,mouse_position().0,mouse_position().1));}
    }
    if let Some(id)=toggle {if g.selected.contains(&id){g.selected.retain(|x|*x!=id);}else{g.selected.push(id);}}
    if is_mouse_button_released(MouseButton::Left) {
        if let (Some((id,ox,oy)),Some(group))=(g.drag,stage_drop) {
            let (mx,my)=mouse_position();
            if (mx-ox).abs()+(my-oy).abs()>12.0 {g.stage_card(id,group);}
        }
        g.drag=None;
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
}
fn window_conf()->Conf {Conf{window_title:"PhaseUP - Offline Card Game".to_string(),window_width:1280,window_height:800,window_resizable:true,..Default::default()}}
#[macroquad::main(window_conf)]
async fn main(){let mut game=Game::new();loop{draw_game(&mut game);next_frame().await;}}
