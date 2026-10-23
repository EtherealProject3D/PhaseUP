// Developer layout overrides. Desktop reads an editable JSON file; Android uses defaults.
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone,Copy)]
pub(super) struct Rect { pub x:f32,pub y:f32,pub w:f32,pub h:f32 }
thread_local! { static OVERRIDES: RefCell<HashMap<String,Rect>> = RefCell::new(HashMap::new()); }

// Shared defaults used by the renderer and the visual editor.
pub(super) fn editor_items()->Vec<(&'static str,Rect)> {
    let mut items=vec![
        ("game_logo",Rect{x:24.,y:10.,w:230.,h:42.}),
        ("turn_banner",Rect{x:16.,y:55.,w:1888.,h:62.}),
        ("cpu_1_panel",Rect{x:16.,y:72.,w:626.,h:376.}),
        ("cpu_2_panel",Rect{x:648.,y:72.,w:626.,h:376.}),
        ("cpu_3_panel",Rect{x:1280.,y:72.,w:626.,h:376.}),
        ("player_phase",Rect{x:560.,y:656.,w:800.,h:140.}),
        ("stage_1",Rect{x:274.,y:853.,w:680.,h:117.}),
        ("stage_2",Rect{x:960.,y:853.,w:680.,h:117.}),
        ("hand_panel",Rect{x:12.,y:971.,w:1896.,h:217.}),
    ];
    for i in 1..=3 {
        for j in 0..2 {
            let name=match (i,j){
                (1,0)=>"cpu_1_group_1",(1,1)=>"cpu_1_group_2",
                (2,0)=>"cpu_2_group_1",(2,1)=>"cpu_2_group_2",
                (3,0)=>"cpu_3_group_1",_=>"cpu_3_group_2",
            };
            let panel_x=16.0+(i-1) as f32*632.0;
            let w=(626.0-16.0-6.0)/2.0;
            items.push((name,Rect{x:panel_x+8.0+j as f32*(w+6.0),y:263.,w,h:116.}));
        }
    }
    items
}
// Match the logical hit/layout region to the two lower bays of the
// rendered neon CPU panel, rather than an unrelated fixed 632px grid.
// Canonical human laid-phase artwork and hit regions. These do not depend
// on saved editor overrides or whether the phase is already laid.
pub(super) fn human_phase_panel()->Rect {
    Rect{x:544.0,y:622.0,w:832.0,h:181.0}
}
pub(super) fn human_phase_group(group:usize,count:usize)->Rect {
    let panel=human_phase_panel();
    let inset=16.0;
    let gap=6.0;
    let width=if count==1 {panel.w-2.0*inset}
        else {(panel.w-2.0*inset-gap)/2.0};
    Rect{x:panel.x+inset+group as f32*(width+gap),
        y:panel.y+46.0,w:width,h:116.0}
}
pub(super) fn human_card_area(group:Rect)->Rect {
    Rect{x:group.x+16.0,y:group.y+6.0,w:group.w-32.0,h:84.0}
}
pub(super) fn cpu_panel(player:usize)->Rect {
    let index=player.saturating_sub(1).min(2);
    // Fixed shared geometry: per-player saved overrides must not introduce
    // subtle differences in panel size or relative group placement.
    Rect{x:8.0+index as f32*636.0,y:72.0,w:632.0,h:376.0}
}
// Canonical two-slot CPU geometry. Both slots are identical, even when
// one slot is empty or the phase has a single requirement.
pub(super) fn cpu_group(player:usize,group:usize,_count:usize,_y:f32)->Rect {
    let panel=cpu_panel(player);
    let inset=8.0;
    let gap=6.0;
    let width=(panel.w-2.0*inset-gap)/2.0;
    Rect {
        x:panel.x+inset+group as f32*(width+gap),
        y:panel.y+191.0,
        w:width,
        h:116.0,
    }
}
// Shared live Workshop offsets, stored as tiny layout rectangles so the
// existing F5 reload and layout::save persistence work unchanged.
pub(super) fn cpu_content_offset(label:bool)->(f32,f32) {
    let key=if label {"cpu_label_offset"}else{"cpu_cards_offset"};
    let r=rect(key,Rect{x:0.0,y:0.0,w:1.0,h:1.0});
    (r.x,r.y)
}
pub(super) fn nudge_cpu_content(label:bool,dx:f32,dy:f32) {
    let key=if label {"cpu_label_offset"}else{"cpu_cards_offset"};
    let mut r=rect(key,Rect{x:0.0,y:0.0,w:1.0,h:1.0});
    r.x=(r.x+dx).clamp(-80.0,80.0);
    r.y=(r.y+dy).clamp(-40.0,40.0);
    set(key,r);
}
pub(super) fn cpu_card_area(group:Rect,neon:bool)->Rect {
    let inset=if neon {12.0}else{16.0};
    Rect {
        x:group.x+inset+cpu_content_offset(false).0,
        y:group.y+36.0+cpu_content_offset(false).1,
        w:group.w-2.0*inset,
        h:84.0,
    }
}
pub(super) fn cpu_group_key(player:usize,group:usize)->&'static str{
    match (player,group){(1,0)=>"cpu_1_group_1",(1,1)=>"cpu_1_group_2",(2,0)=>"cpu_2_group_1",(2,1)=>"cpu_2_group_2",(3,0)=>"cpu_3_group_1",_=>"cpu_3_group_2"}
}
pub(super) fn editor_items_for(g:&crate::Game)->Vec<(String,Rect)>{
    let mut items:Vec<(String,Rect)>=editor_items().into_iter().map(|(n,r)|(n.to_owned(),r)).collect();
    let sh=crate::game_canvas_height();
    let groups=crate::PHASES[g.players[0].phase.min(crate::PHASES.len()-1)];
    let stage_w=if groups.len()==1 {800.0}else{680.0};
    let stage_left=(1920.0-(groups.len() as f32*stage_w+(groups.len().saturating_sub(1)) as f32*6.0))/2.0;
    for group in 0..groups.len().min(2){
        let base=rect(if group==0 {"stage_1"}else{"stage_2"},Rect{x:stage_left+group as f32*(stage_w+6.0),y:sh-347.0,w:stage_w,h:117.0});
        for (index,id) in g.staged[group].iter().enumerate(){
            items.push((format!("stage_card_{id}"),Rect{x:base.x+10.0+index as f32*57.0,y:base.y+29.0,w:50.0,h:76.0}));
        }
    }
    for player in 0..4 {
        let p=&g.players[player];
        for (group_index,group) in p.table.iter().enumerate(){
            if group_index>1 {continue;}
            let default=if player==0 {
                Rect{x:560.0+group_index as f32*403.0,y:sh-532.0+12.0,w:if p.table.len()==1 {800.0}else{397.0},h:116.0}
            }else{
                Rect{x:32.0+(player-1) as f32*632.0+group_index as f32*312.0,y:260.0,w:if p.table.len()==1 {615.0}else{305.0},h:116.0}
            };
            let base=rect(if player==0 {"player_phase"}else{cpu_group_key(player,group_index)},default);
            let step=if group.len()>1 {((base.w-67.0).max(0.0)/(group.len()-1) as f32).min(56.0)}else{0.0};
            for (index,card) in group.iter().enumerate(){
                items.push((format!("laid_card_{}",card.id),Rect{x:base.x+if player==0 {16.0}else{if group_index==0 {12.0}else{4.0}}+index as f32*step,y:base.y+if player==0 {6.0}else{36.0},w:55.0,h:84.0}));
            }
        }
    }
    items
}
pub(super) fn default_rect(name:&str)->Option<Rect>{
    editor_items().into_iter().find(|(key,_)|*key==name).map(|(_,r)|r)
}
pub(super) fn at(name:&str,default:Rect)->Rect{rect(name,default)}

pub(super) fn rect(name:&str, default:Rect)->Rect {
    OVERRIDES.with(|m|m.borrow().get(name).copied().unwrap_or(default))
}

pub(super) fn set(name:&str,value:Rect){OVERRIDES.with(|m|{m.borrow_mut().insert(name.to_owned(),value);});}

#[cfg(not(target_os="android"))]
pub(super) fn save()->Result<(),String>{
    let path="assets/layouts/game_16_10.json";
    let mut root=std::fs::read_to_string(path).ok()
        .and_then(|s|parse_layout(&s))
        .unwrap_or_else(||serde_json::json!({"schema_version":1,"canvas":{"width":1920,"height":1200},"elements":{}}));
    if !root.get("elements").is_some_and(|v|v.is_object()) {root["elements"]=serde_json::json!({});}
    OVERRIDES.with(|m|for (name,r) in m.borrow().iter(){
        root["elements"][name]=serde_json::json!({"x":r.x,"y":r.y,"width":r.w,"height":r.h});
    });
    std::fs::create_dir_all("assets/layouts").map_err(|e|e.to_string())?;
    std::fs::write(path,serde_json::to_string_pretty(&root).map_err(|e|e.to_string())?+"\n").map_err(|e|e.to_string())
}

// Remove every saved and in-memory layout override, retaining only renderer defaults.
#[cfg(not(target_os="android"))]
pub(super) fn clear_all()->Result<(),String>{
    let path="assets/layouts/game_16_10.json";
    let root=serde_json::json!({"schema_version":1,"canvas":{"width":1920,"height":1200},"elements":{}});
    std::fs::create_dir_all("assets/layouts").map_err(|e|e.to_string())?;
    std::fs::write(path,format!("{}\n",serde_json::to_string_pretty(&root).map_err(|e|e.to_string())?))
        .map_err(|e|e.to_string())?;
    OVERRIDES.with(|m|m.borrow_mut().clear());
    Ok(())
}

// Accept legacy files that ended in a literal backslash-n due to an older save bug.
#[cfg(not(target_os="android"))]
fn parse_layout(data:&str)->Option<serde_json::Value>{
    serde_json::from_str(data).ok().or_else(||{
        let trimmed=data.trim_end();
        let repaired=trimmed.strip_suffix("\\n")?;
        serde_json::from_str(repaired).ok()
    })
}

pub(super) fn reload(){
    let mut values=HashMap::new();
    #[cfg(not(target_os="android"))]
    if let Ok(data)=std::fs::read_to_string("assets/layouts/game_16_10.json") {
        if let Some(json)=parse_layout(&data) {
            if let Some(elements)=json.get("elements").and_then(|v|v.as_object()) {
                for (name,element) in elements {
                    let read=|field:&str| element.get(field).and_then(|v|v.as_f64()).map(|n|n as f32);
                    if let (Some(x),Some(y),Some(w),Some(h))=(read("x"),read("y"),read("width"),read("height")) {
                        if [x,y,w,h].iter().all(|n|n.is_finite()) && w>0.0 && h>0.0 {
                            values.insert(name.clone(),Rect{x,y,w,h});
                        }
                    }
                }
            }
        }
    }
    OVERRIDES.with(|m|*m.borrow_mut()=values);
}
