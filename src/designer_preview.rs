//! Plain, deterministic interaction sandbox. No live game or user save is mutated.
use super::{phases::*, Box2, Card, Kind, Snapshot};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use std::collections::BTreeSet;

// One aspect ratio for hand, phase, CPU and dragged cards.
const CARD_ASPECT:f32=0.68;
#[derive(Clone,Copy,Debug,Default)]
pub(super) struct HandStyle {pub height:Option<f32>,pub dx:f32,pub dy:f32,pub gap:Option<f32>}
pub(super) type HandStyles=std::collections::BTreeMap<String,HandStyle>;

pub(super) fn card_offset(styles:&HandStyles,key:&str,index:usize)->Vec2 {
    let name=format!("{key}_card_{index}");
    let style=styles.get(&name).copied().unwrap_or_default();
    Vec2::new(style.dx,style.dy)
}
pub(super) fn resize_hand(style:&mut HandStyle,old_height:f32,height:f32,anchor:usize){
    let delta=height-old_height;
    match anchor {1=>style.dx+=delta*CARD_ASPECT/2.,2=>style.dx-=delta*CARD_ASPECT/2.,3=>style.dy+=delta/2.,4=>style.dy-=delta/2.,_=>{}}
    style.height=Some(height);
}
fn styled_row(cards:&[Card],r:Rect,z:f32,selected:&BTreeSet<u32>,style:HandStyle)->Vec<(Card,Rect)>{
    if cards.is_empty(){return vec![];}
    let default_height=card_size(Vec2::new(r.width(),r.height().min(136.*z))).y;
    let height=style.height.map(|h|h*z).unwrap_or(default_height);
    let width=height*CARD_ASPECT;
    let count=cards.len();
    let gap=style.gap.unwrap_or(12.).clamp(0.,200.)*z;
    let ideal_step=width+gap;
    let available_step=if count>1 {(r.width()-width).max(0.)/(count-1) as f32}else{0.};
    let step=if count>1 {ideal_step.min(available_step)}else{0.};
    let total=width+step*(count-1) as f32;
    let left=r.center().x-total/2.+style.dx*z;
    cards.iter().enumerate().map(|(i,c)|{
        let y=r.center().y-height/2.+style.dy*z-if selected.contains(&c.id){12.*z}else{0.};
        (*c,Rect::from_min_size(Pos2::new(left+i as f32*step,y),Vec2::new(width,height)))
    }).collect()
}
fn card_size(bounds:Vec2)->Vec2 {
    let height=bounds.y.max(0.).min(bounds.x.max(0.)/CARD_ASPECT);
    Vec2::new(height*CARD_ASPECT,height)
}

pub(super) fn add_children(elements:&mut Snapshot) {
    for parent in ["phase","staging"] {
        let Some(p)=elements.get(parent).copied() else {continue};
        for count in 1..=2 {
            let gap=8.; let w=(p.w-24.-gap*(count-1) as f32)/count as f32;
            for g in 0..count {elements.entry(format!("{parent}_{count}_group_{}",g+1)).or_insert(Box2{x:p.x+12.+g as f32*(w+gap),y:p.y+36.,w,h:(p.h-44.).max(40.)});}
        }
    }
    for i in 1..=3 {
        let p=elements[&format!("cpu_{i}")];
        elements.entry(format!("cpu_{i}_hand")).or_insert(Box2{x:p.x+12.,y:p.y+40.,w:p.w-24.,h:90.});
        for g in 0..2 {elements.entry(format!("cpu_{i}_laid_{}",g+1)).or_insert(Box2{x:p.x+12.+g as f32*(p.w-16.)/2.,y:p.y+142.,w:(p.w-32.)/2.,h:p.h-154.});}
    }
}
pub(super) fn parent(name:&str)->Option<String> {
    if name.starts_with("phase_")&&name.contains("_group_"){Some("phase".into())}
    else if name.starts_with("staging_"){Some("staging".into())}
    else if name.starts_with("cpu_")&&name.matches('_').count()>1 {Some(name[..5].into())}else{None}
}
// Remove the hand-row band without shrinking the playable laid-phase cards.
pub(super) fn collapse_cpu_rows(elements:&mut Snapshot) {
    for i in 1..=3 {
        let panel_key=format!("cpu_{i}");
        let panel=elements[&panel_key];
        let top=(1..=2).map(|g|elements[&format!("cpu_{i}_laid_{g}")].y).fold(f32::INFINITY,f32::min);
        let removed=(top-(panel.y+40.)).max(0.);
        elements.get_mut(&panel_key).unwrap().h=(panel.h-removed).max(40.);
        for g in 1..=2 {elements.get_mut(&format!("cpu_{i}_laid_{g}")).unwrap().y-=removed;}
    }
}

pub(super) fn layout(separate: bool) -> Snapshot {
    let mut result = Snapshot::new();
    for (name,x,y,w,h) in [
        ("header",24.,24.,1872.,64.),
        ("cpu_1",24.,112.,608.,280.),("cpu_2",656.,112.,608.,280.),("cpu_3",1288.,112.,608.,280.),
        ("draw",800.,420.,120.,155.),("discard",1000.,420.,120.,155.),
        ("phase",280.,610.,1360.,220.),
        ("staging",280.,840.,1360.,150.),
        ("phase_action",760.,if separate{780.}else{845.},400.,54.),
        ("hand",24.,if separate{1000.}else{925.},1872.,if separate{130.}else{205.}),
        ("sort_runs",650.,1140.,180.,40.),("sort_colors",850.,1140.,180.,40.),("sort_sets",1050.,1140.,180.,40.),
        ("discard_action",1460.,1140.,300.,40.),
    ] {result.insert(name.into(),Box2{x,y,w,h});}
    if !separate {result.remove("staging");}
    else {result.get_mut("phase").unwrap().h=155.;}
    add_children(&mut result);result
}

pub(super) struct Preview {
    phase: usize, hand: Vec<Card>, groups: [Vec<Card>;2], selected: BTreeSet<u32>,
    pinned: BTreeSet<u32>, last_sort: Option<usize>, laid: bool, ended: bool,
    dragging: Option<u32>, discard: Option<Card>, message: String,
}
impl Preview {
    pub(super) fn hand_items(&self,elements:&Snapshot,styles:&HandStyles,show_cpu:bool)->Vec<(String,usize,Rect)> {
        let mut items=vec![];
        let r=elements["hand"].rect();
        let content=Rect::from_min_max(r.min+Vec2::new(20.,42.),r.max-Vec2::splat(12.));
        for (i,(_,r)) in styled_row(&self.hand,content,1.,&self.selected,styles.get("hand").copied().unwrap_or_default()).into_iter().enumerate(){items.push(("hand".into(),i,r.translate(card_offset(styles,"hand",i))));}
        if show_cpu {for cpu in 1..=3 {
            let key=format!("cpu_{cpu}_hand");let cards:Vec<_>=(0..10).map(|id|Card{id,kind:Kind::Skip}).collect();
            for (i,(_,r)) in styled_row(&cards,elements[&key].rect().shrink(6.),1.,&BTreeSet::new(),styles.get(&key).copied().unwrap_or_default()).into_iter().enumerate(){items.push((key.clone(),i,r));}
        }}
        items
    }
    pub(super) fn visible(&self,name:&str,show_cpu:bool,separate:bool)->bool {
        if name.starts_with("cpu_")&&name.ends_with("_hand")&&!show_cpu{return false;}
        if name.contains("_group_") {
            let count=PHASES[self.phase].len();
            if !name.contains(&format!("_{count}_group_")){return false;}
            if name.starts_with("staging_"){return separate&&!self.laid;}
            if name.starts_with("phase_"){return !separate||self.laid;}
        }
        true
    }
    pub(super) fn new(phase: usize) -> Self {
        let mut hand=Vec::new();
        for (g,&count) in PHASES[phase].iter().enumerate() {
            for i in 0..count {
                let kind=match requirement(phase,g) {
                    "run"=>Kind::Number((i+2) as u8,(i%4) as u8),
                    "color"=>Kind::Number((i+1) as u8,1),
                    _=>Kind::Number((g+4) as u8,(i%4) as u8),
                };
                hand.push(Card{id:hand.len() as u32,kind});
            }
        }
        while hand.len()<11 {hand.push(Card{id:hand.len() as u32,kind:Kind::Number(12,2)});}
        Self{phase,hand,groups:[vec![],vec![]],selected:BTreeSet::new(),pinned:BTreeSet::new(),last_sort:None,laid:false,ended:false,dragging:None,discard:None,message:"Preview starts after drawing. Build your phase, play it, then discard.".into()}
    }
    fn ready(&self)->bool {
        !self.laid && PHASES[self.phase].iter().enumerate().all(|(g,&n)|
            self.groups[g].len()>=n && self.groups[g].len()<=12 && valid_group(&self.groups[g],requirement(self.phase,g)))
    }
    fn transfer(&mut self,id:u32,target:Option<usize>,index:usize) {
        if self.ended{return;}
        let card=self.hand.iter().chain(self.groups.iter().flatten()).find(|c|c.id==id).copied();
        let Some(card)=card else{return;};
        if self.laid {
            if !self.hand.iter().any(|c|c.id==id){return;}
            let Some(g)=target else {self.reorder(id,index);return;};
            if self.groups[g].len()>=12 || !can_hit(&self.groups[g],card,self.phase,g){self.message="That card cannot extend this group.".into();return;}
        }
        self.hand.retain(|c|c.id!=id);
        for group in &mut self.groups {group.retain(|c|c.id!=id);}
        if let Some(g)=target {self.groups[g].push(card);if requirement(self.phase,g)=="run" && self.groups[g].len()<=12 && valid_group(&self.groups[g],"run"){order_run(&mut self.groups[g]);}}
        else {self.hand.insert(index.min(self.hand.len()),card);self.pinned.insert(id);}
        self.selected.remove(&id);self.last_sort=None;
    }
    fn reorder(&mut self,id:u32,index:usize) {
        if let Some(i)=self.hand.iter().position(|c|c.id==id) {
            let c=self.hand.remove(i);self.hand.insert(index.min(self.hand.len()),c);
            self.pinned.insert(id);self.last_sort=None;
        }
    }
    fn sort(&mut self,mode:usize) {
        if self.last_sort==Some(mode){self.pinned.clear();self.last_sort=None;}else{self.last_sort=Some(mode);}
        let mut movable:Vec<Card>=self.hand.iter().filter(|c|!self.pinned.contains(&c.id)).copied().collect();
        movable.sort_by_key(|c| match c.kind {Kind::Number(n,col) if mode==1=>(col,n,c.id),Kind::Number(n,col)=>(n,col,c.id),Kind::Wild=>(250,0,c.id),Kind::Skip=>(251,0,c.id)});
        let mut it=movable.into_iter();
        for c in &mut self.hand {if !self.pinned.contains(&c.id){*c=it.next().unwrap();}}
    }
    fn play_phase(&mut self) {
        if self.ready(){self.laid=true;self.selected.clear();self.last_sort=None;self.message="PHASE IN PLAY — valid hits are now allowed. Discard to finish.".into();}
    }
    fn discard_selected(&mut self) {
        if self.ended{return;}
        if self.ready(){self.message="Your phase is ready. Play it before discarding in this preview.".into();return;}
        if self.selected.len()!=1 {self.message="Select exactly one hand card to discard.".into();return;}
        let id=*self.selected.first().unwrap();
        if let Some(i)=self.hand.iter().position(|c|c.id==id){self.discard=Some(self.hand.remove(i));self.selected.clear();self.ended=true;self.message="TURN COMPLETE — reset the preview to try another arrangement.".into();}
    }
    pub(super) fn controls(&mut self,ui:&mut egui::Ui) {
        ui.heading("Plain prototype");
        ui.label("Interaction sandbox, not a live game.");
        let mut p=self.phase;
        egui::ComboBox::from_label("Phase").selected_text(format!("{}: {}",p+1,phase_name(p))).show_ui(ui,|ui|{for n in 0..10 {ui.selectable_value(&mut p,n,format!("{}: {}",n+1,phase_name(n)));}});
        if p!=self.phase {*self=Self::new(p);}
        if ui.button("Reset test turn").clicked(){*self=Self::new(self.phase);}
        if ui.button("Fill valid phase").clicked(){
            *self=Self::new(self.phase);
            for g in 0..PHASES[self.phase].len(){for _ in 0..PHASES[self.phase][g]{let id=self.hand[0].id;self.transfer(id,Some(g),0);}}
        }
        ui.label("Click cards to raise/select. Drag cards into a group or back to the hand. Drag within the hand to pin a position (dot).");
        ui.label("Sort once: preserve pinned positions. Same sort again: sort all. Moving/selecting a card resets the second click.");
        ui.label("Uncheck Play preview to move or resize layout elements. Width/height are in Properties.");
        ui.separator();ui.label(&self.message);
    }
    pub(super) fn draw(&mut self,ui:&mut egui::Ui,board:Rect,z:f32,elements:&Snapshot,separate:bool,interactive:bool,texts:&std::collections::BTreeMap<String,super::LabelStyle>,show_cpu:bool,cpu_laid:bool,styles:&HandStyles,cpu_counts:&std::collections::BTreeMap<String,usize>) {
        let screen=|b:Box2|Rect::from_min_size(board.min+Vec2::new(b.x,b.y)*z,Vec2::new(b.w,b.h)*z);
        let area=|name:&str|screen(elements[name]);
        let text=|ui:&egui::Ui,r:Rect,s:&str,size:f32|{ui.painter().text(r.center(),Align2::CENTER_CENTER,s,FontId::proportional((size*z).max(9.)),Color32::WHITE);};
        let label=|ui:&egui::Ui,key:&str,r:Rect,s:&str,size:f32|{
            let style=texts.get(key).cloned().unwrap_or_default();
            let content=if style.text.is_empty(){s.to_owned()}else{style.text.replace("{phase_number}",&(self.phase+1).to_string()).replace("{phase_name}",phase_name(self.phase)).replace("{value}",s)};
            text(ui,r.translate(Vec2::new(style.dx,style.dy)*z),&content,style.size.unwrap_or(size));
        };
        let mut panels:Vec<_>=elements.iter().filter(|(n,_)|self.visible(n,show_cpu,separate)).collect();
        panels.sort_by_key(|(n,_)|usize::from(parent(n).is_some()));
        for (name,b) in panels {
            let bounds=screen(*b);
            let r=if name=="draw"||name=="discard"{Rect::from_center_size(bounds.center(),card_size(bounds.size()))}else{bounds};
            ui.painter().rect_filled(r,3.,Color32::from_rgb(32,39,48));
            ui.painter().rect_stroke(r,3.,Stroke::new(1.0_f32,Color32::from_gray(110)));
            if name.starts_with("cpu_") && parent(name).is_none(){label(ui,name,Rect::from_min_size(r.min,Vec2::new(r.width(),32.*z)),&format!("{}  |  Phase 1  |  10 cards",name.to_uppercase()),22.);}
        }
        for i in 1..=3 {
            let key=format!("cpu_{i}_hand");let r=area(&key);
            if show_cpu {
                let cards:Vec<_>=(0..10).map(|id|Card{id,kind:Kind::Skip}).collect();
                for (_,r) in styled_row(&cards,r.shrink(6.*z),z,&BTreeSet::new(),styles.get(&key).copied().unwrap_or_default()) {ui.painter().rect_filled(r,2.,Color32::from_rgb(45,70,95));ui.painter().rect_stroke(r,2.,Stroke::new(1.0_f32,Color32::LIGHT_BLUE));}
                label(ui,&key,r,"HAND",16.);
            }
            for g in 1..=2 {
                let key=format!("cpu_{i}_laid_{g}");let r=area(&key);
                let count=*cpu_counts.get(&key).unwrap_or(&3);
                label(ui,&key,Rect::from_min_size(r.min,Vec2::new(r.width(),24.*z)),&format!("SET — {count} cards"),18.);
                if cpu_laid {
                    let cards:Vec<_>=(0..count as u32).map(|id|Card{id,kind:Kind::Number((i+g) as u8,(id%4) as u8)}).collect();
                    let content=Rect::from_min_max(
    r.min+Vec2::new(6.*z,28.*z),
    r.max-Vec2::splat(6.*z)
);
let style=styles.get(&key).copied().unwrap_or_default();
// Lay every CPU preview group out with the same centering and spacing
// rules. Preserve its configured card height and offset, but only
// overlap cards when the available width makes it unavoidable.
let natural_height=content.height().max(0.).min(content.width().max(0.)/CARD_ASPECT);
let card_h=style.height.unwrap_or(natural_height/z)*z;
let card_w=card_h*CARD_ASPECT;
let n=cards.len();
let gap=style.gap.unwrap_or(16.).clamp(0.,200.)*z;
let available=(content.width()-card_w).max(0.);
let step=if n>1 {
    (available/(n-1) as f32).min(card_w+gap)
}else{0.};
let total=if n>0 {card_w+step*(n-1) as f32}else{0.};
let left=content.center().x-total/2.+style.dx*z;
let top=content.center().y-card_h/2.+style.dy*z;
let laid_cards:Vec<_>=cards.into_iter().enumerate().map(|(index,card)|{
    (card,Rect::from_min_size(
        Pos2::new(left+index as f32*step,top),
        Vec2::new(card_w,card_h)
    ))
}).collect();
for (card,card_rect) in laid_cards {
    paint_card(ui,card,card_rect,z,false);
}
                }else{text(ui,r.translate(Vec2::new(0.,12.*z)),"NOT LAID",18.);}
            }
        }
        if elements.contains_key("header") { label(ui,"header",area("header"),"PHASE-UP  |  PLAIN LAYOUT PROTOTYPE",30.); }
        label(ui,"draw",area("draw"),"DRAW",22.);
        label(ui,"discard",area("discard"),&self.discard.map(|c|card_label(c)).unwrap_or("DISCARD".into()),28.);
        let phase_area=area("phase");
        let active=if separate && !self.laid{area("staging")}else{phase_area};
        if separate && !self.laid {text(ui,phase_area,"YOUR LAID PHASE — NOT YET PLAYED",24.);}
        let status=if self.laid{"PHASE IN PLAY"}else if self.ready(){"READY — NOT IN PLAY"}else{"BUILD YOUR PHASE — NOT IN PLAY"};
        let heading=Rect::from_min_size(active.min,Vec2::new(active.width(),32.*z));
        label(ui,if separate&&!self.laid{"staging"}else{"phase"},heading,status,22.);
        let count=PHASES[self.phase].len();
        let group_keys:Vec<String>=(0..count).map(|g|format!("{}_{count}_group_{}",if separate&&!self.laid{"staging"}else{"phase"},g+1)).collect();
        let groups:Vec<Rect>=group_keys.iter().map(|key|area(key)).collect();
        let mut slots:Vec<(Card,Rect)>=vec![];
        for (g,r) in groups.iter().enumerate(){
            ui.painter().rect_stroke(*r,2.,Stroke::new(1.0_f32,if self.laid{Color32::LIGHT_GREEN}else{Color32::GRAY}));
            label(ui,&group_keys[g],Rect::from_min_size(r.min,Vec2::new(r.width(),24.*z)),&format!("{}  {} / {}",requirement(self.phase,g).to_uppercase(),self.groups[g].len(),PHASES[self.phase][g]),18.);
            slots.extend(styled_row(&self.groups[g],Rect::from_min_max(r.min+Vec2::new(8.*z,28.*z),r.max-Vec2::splat(6.*z)),z,&self.selected,styles.get(&group_keys[g]).copied().unwrap_or_default()));
        }
        let hand=area("hand");
        label(ui,"hand",Rect::from_min_size(hand.min,Vec2::new(hand.width(),28.*z)),&format!("YOUR HAND — PHASE {}: {}",self.phase+1,phase_name(self.phase)),22.);
        let hand_content=Rect::from_min_max(hand.min+Vec2::new(20.*z,42.*z),hand.max-Vec2::splat(12.*z));
        let hand_slots:Vec<_>=styled_row(&self.hand,hand_content,z,&self.selected,styles.get("hand").copied().unwrap_or_default())
    .into_iter().enumerate()
    .map(|(i,(card,r))|(card,r.translate(card_offset(styles,"hand",i)*z)))
    .collect();
        slots.extend(hand_slots.iter().copied());
        let mut dropped=None;
        for (card,r) in &slots {
            paint_card(ui,*card,*r,z,self.pinned.contains(&card.id));
            let movable=!self.ended && (!self.laid || self.hand.iter().any(|c|c.id==card.id));
            if interactive && movable {
                let response=ui.interact(*r,ui.id().with(("card",card.id)),Sense::click_and_drag());
                if response.clicked(){if !self.selected.insert(card.id){self.selected.remove(&card.id);}self.last_sort=None;}
                if response.drag_started(){self.dragging=Some(card.id);}
                if response.drag_stopped(){dropped=self.dragging.take();}
            }
        }
        if let Some(id)=dropped {
            if let Some(p)=ui.input(|i|i.pointer.interact_pos()) {
                if let Some(g)=groups.iter().position(|r|r.contains(p)){self.transfer(id,Some(g),0);}
                else if hand.contains(p){let index=hand_slots.iter().position(|(_,r)|p.x<r.center().x).unwrap_or(self.hand.len());self.transfer(id,None,index);}
            }
        }
        let enabled=self.ready()&&!self.ended;
        let r=area("phase_action");
        if enabled{ui.painter().rect_filled(r,3.,Color32::from_rgb(36,95,65));}
        label(ui,"phase_action",r,if self.laid{"PHASE IN PLAY"}else{"PLAY PHASE"},24.);
        if interactive && enabled && ui.interact(r,ui.id().with("play_phase"),Sense::click()).clicked(){self.play_phase();}
        for (mode,name) in ["sort_runs","sort_colors","sort_sets"].iter().enumerate(){
            let r=area(name);label(ui,name,r,["RUNS","COLORS","SETS"][mode],22.);
            if interactive && !self.ended && ui.interact(r,ui.id().with(name),Sense::click()).clicked(){self.sort(mode);}
        }
        let r=area("discard_action");label(ui,"discard_action",r,if self.ended{"TURN COMPLETE"}else{"DISCARD SELECTED"},22.);
        if interactive && ui.interact(r,ui.id().with("discard_action"),Sense::click()).clicked(){self.discard_selected();}
        if let Some(id)=self.dragging {
            if let (Some(p),Some(card))=(ui.input(|i|i.pointer.interact_pos()),slots.iter().find(|(c,_)|c.id==id).map(|(c,_)|*c)) {
                let size=slots.iter().find(|(c,_)|c.id==id).map(|(_,r)|r.size()).unwrap_or(Vec2::new(68.,100.)*z);
                paint_card(ui,card,Rect::from_center_size(p,size),z,false);
            }
        }
    }
}
fn row(cards:&[Card],r:Rect,z:f32,selected:&BTreeSet<u32>)->Vec<(Card,Rect)> {
    if cards.is_empty(){return vec![];}
    let size=card_size(Vec2::new(r.width(),r.height().min(136.*z)));
    let h=size.y;let w=size.x;
    let step=((r.width()-w)/(cards.len().saturating_sub(1).max(1) as f32)).min(w+12.*z).max(1.);
    let start=r.center().x-(w+step*(cards.len()-1) as f32)/2.;
    cards.iter().enumerate().map(|(i,c)|(*c,Rect::from_min_size(Pos2::new(start+i as f32*step,r.center().y-h/2.-if selected.contains(&c.id){12.*z}else{0.}),Vec2::new(w,h)))).collect()
}
fn card_label(c:Card)->String {match c.kind {Kind::Number(n,_)=>n.to_string(),Kind::Wild=>"W".into(),Kind::Skip=>"S".into()}}
fn paint_card(ui:&egui::Ui,c:Card,r:Rect,z:f32,pinned:bool) {
    let r=Rect::from_center_size(r.center(),card_size(r.size()));
    let color=match c.kind {Kind::Number(_,0)=>Color32::from_rgb(135,48,50),Kind::Number(_,1)=>Color32::from_rgb(40,85,140),Kind::Number(_,2)=>Color32::from_rgb(130,105,35),Kind::Number(_,_)=>Color32::from_rgb(40,115,70),Kind::Wild=>Color32::from_rgb(110,55,140),Kind::Skip=>Color32::GRAY};
    ui.painter().rect_filled(r,3.,color);ui.painter().rect_stroke(r,3.,Stroke::new(1.0_f32,Color32::WHITE));
    ui.painter().text(r.center(),Align2::CENTER_CENTER,card_label(c),FontId::proportional((32.*z).max(10.)),Color32::WHITE);
    if pinned {ui.painter().circle_filled(r.center_bottom()-Vec2::new(0.,6.*z),3.*z,Color32::WHITE);}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn resizing_one_card_resizes_its_hand_and_preserves_the_chosen_anchor(){
        let p=Preview::new(0);let e=layout(false);
        let before=p.hand_items(&e,&Default::default(),true);
        for anchor in 0..5 {
            let old=before.iter().find(|(key,_,_)|key=="hand").unwrap().2.height();
            let mut style=HandStyle::default();resize_hand(&mut style,old,old+50.,anchor);
            let styles=HandStyles::from([("hand".into(),style)]);
            let after=p.hand_items(&e,&styles,true);
            for ((key,_,a),(_,_,b)) in before.iter().zip(after.iter()) {
                if key!="hand"{assert_eq!(a,b);continue;}
                assert!((b.height()-old-50.).abs()<0.001);
                assert!((b.width()/b.height()-CARD_ASPECT).abs()<0.001);
                let (a,b)=match anchor {1=>(a.left(),b.left()),2=>(a.right(),b.right()),3=>(a.top(),b.top()),4=>(a.bottom(),b.bottom()),_=>(a.center().x,b.center().x)};
                assert!((a-b).abs()<0.001);
            }
        }
    }
    #[test] fn cards_keep_proportions_in_tall_wide_and_small_areas(){
        let cards=vec![Card{id:0,kind:Kind::Wild}];
        for size in [Vec2::new(30.,300.),Vec2::new(300.,30.),Vec2::new(2.,2.),Vec2::new(800.,220.)] {
            let r=Rect::from_min_size(Pos2::ZERO,size);
            let bounds=row(&cards,r,1.,&BTreeSet::new())[0].1;
            assert!((bounds.width()/bounds.height()-CARD_ASPECT).abs()<0.0001);
            assert!(bounds.width()<=r.width()+0.001&&bounds.height()<=r.height()+0.001);
        }
    }
    #[test] fn pointer_selects_and_drags_card_into_phase() {
        let ctx=egui::Context::default();let mut p=Preview::new(0);let layout=layout(false);
        let board=Rect::from_min_size(Pos2::ZERO,Vec2::new(1920.,1200.));
        let frame=|p:&mut Preview,events:Vec<egui::Event>| {
            let input=egui::RawInput{screen_rect:Some(board),events,..Default::default()};
            let _=ctx.run(input,|ctx|{egui::CentralPanel::default().show(ctx,|ui|{
                ui.allocate_rect(board,Sense::click_and_drag());p.draw(ui,board,1.,&layout,false,true,&Default::default(),true,false,&Default::default(),&Default::default());
            });});
        };
        let press=|pos,pressed|egui::Event::PointerButton{pos,button:egui::PointerButton::Primary,pressed,modifiers:egui::Modifiers::default()};
        let r=layout["hand"].rect();
        let cards=row(&p.hand,Rect::from_min_max(r.min+Vec2::new(20.,42.),r.max-Vec2::splat(12.)),1.,&p.selected);
        let origin=cards[0].1.center();let id=cards[0].0.id;
        frame(&mut p,vec![]);
        frame(&mut p,vec![egui::Event::PointerMoved(origin),press(origin,true)]);
        frame(&mut p,vec![press(origin,false)]);
        assert!(p.selected.contains(&id));
        frame(&mut p,vec![press(origin,true)]);
        let target=Pos2::new(450.,720.);
        frame(&mut p,vec![egui::Event::PointerMoved(target)]);
        frame(&mut p,vec![press(target,false)]);
        assert!(p.groups[0].iter().any(|c|c.id==id));
        assert!(!p.hand.iter().any(|c|c.id==id));
    }
    #[test] fn complete_turn_all_phases(){for phase in 0..10 {let mut p=Preview::new(phase);for g in 0..PHASES[phase].len(){for _ in 0..PHASES[phase][g]{let id=p.hand[0].id;p.transfer(id,Some(g),0);}}assert!(p.ready());p.play_phase();assert!(p.laid);let id=p.groups[0][0].id;p.transfer(id,None,0);assert!(!p.hand.iter().any(|c|c.id==id));p.selected.insert(p.hand[0].id);p.discard_selected();assert!(p.ended);}}
    #[test] fn unplayed_cards_return_and_sort_pins(){let mut p=Preview::new(0);let id=p.hand[0].id;p.transfer(id,Some(0),0);p.transfer(id,None,3);assert_eq!(p.hand[3].id,id);p.sort(1);assert_eq!(p.hand[3].id,id);assert!(p.pinned.contains(&id));p.sort(1);assert!(p.pinned.is_empty());}
    #[test] fn profile_isolation(){assert!(!layout(false).contains_key("staging"));assert!(layout(true).contains_key("staging"));assert_ne!(layout(false)["hand"].y,layout(true)["hand"].y);}
    #[test] fn collapsing_cpu_rows_reclaims_space_without_shrinking_cards(){
        let mut e=layout(false);let old=e.clone();collapse_cpu_rows(&mut e);
        for i in 1..=3 {
            let key=format!("cpu_{i}");assert_eq!(old[&key].h-e[&key].h,102.);
            for g in 1..=2 {let key=format!("cpu_{i}_laid_{g}");assert_eq!(e[&key].h,old[&key].h);assert_eq!(e[&key].w,old[&key].w);assert_eq!(old[&key].y-e[&key].y,102.);}
        }
        let once=e.clone();collapse_cpu_rows(&mut e);assert_eq!(e["cpu_1"].h,once["cpu_1"].h);
        assert_eq!(e["hand"].y,old["hand"].y);
    }
    #[test] fn group_boxes_are_centered_and_visibility_matches_state(){
        let e=layout(false);let p=e["phase"];let a=e["phase_2_group_1"];let b=e["phase_2_group_2"];
        assert_eq!(a.x-p.x,p.x+p.w-b.x-b.w);
        assert_eq!(b.x-a.x-a.w,8.);
        let preview=Preview::new(0);
        assert!(preview.visible("phase_2_group_1",true,false));
        assert!(!preview.visible("phase_1_group_1",true,false));
        assert!(!preview.visible("cpu_1_hand",false,false));
        assert!(preview.visible("cpu_1_laid_1",false,false));
        let single=Preview::new(3);assert!(single.visible("phase_1_group_1",true,false));
    }
}
