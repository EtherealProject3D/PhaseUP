// Desktop reads editable layout overrides; Android embeds the saved build layout.
#[path = "designer_gameplay.rs"]
pub(super) mod designer;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone,Copy)]
pub(super) struct Rect { pub x:f32,pub y:f32,pub w:f32,pub h:f32 }
thread_local! { static OVERRIDES: RefCell<HashMap<String,Rect>> = RefCell::new(HashMap::new()); }

// Shared defaults used by the renderer and the visual editor.
pub(super) fn editor_items()->Vec<(&'static str,Rect)> {
    let mut items=vec![
        ("game_logo",Rect{x:24.,y:10.,w:230.,h:42.}),
        ("game_version",Rect{x:270.,y:21.,w:140.,h:25.}),
        ("round_turn_counter",Rect{x:1400.,y:16.,w:288.,h:28.}),
        ("menu_button",Rect{x:1710.,y:8.,w:100.,h:40.}),
        ("upper_banner",Rect{x:12.,y:8.,w:1896.,h:48.}),
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
pub(super) fn human_phase_panel(count:usize)->Rect {
    rect(if count==1 {"player_phase_single"}else{"player_phase_double"},Rect{x:544.0,y:622.0,w:832.0,h:181.0})
}
pub(super) fn human_phase_group(group:usize,count:usize)->Rect {
    let panel=human_phase_panel(count);
    let inset=16.0;
    let gap=6.0;
    let width=if count==1 {panel.w-2.0*inset}
        else {(panel.w-2.0*inset-gap)/2.0};
    rect(&format!("human_{}_group_{}",if count==1 {"single"}else{"double"},group+1),Rect{x:panel.x+inset+group as f32*(width+gap),
        y:panel.y+46.0,w:width,h:116.0})
}
pub(super) fn human_content_offset(label:bool,count:usize)->(f32,f32) {
    let key=match (count==1,label) {(true,true)=>"human_single_label_offset",(true,false)=>"human_single_cards_offset",(false,true)=>"human_double_label_offset",(false,false)=>"human_double_cards_offset"};
    let r=rect(key,Rect{x:0.0,y:0.0,w:1.0,h:1.0});
    (r.x,r.y)
}
pub(super) fn human_card_area(group:Rect,count:usize)->Rect {
    {let (dx,dy)=human_content_offset(false,count);
    Rect{x:group.x+16.0+dx,y:group.y+6.0+dy,w:group.w-32.0,h:84.0}}
}
pub(super) fn cpu_panel(player:usize)->Rect {
    let index=player.saturating_sub(1).min(2);
    // Fixed shared geometry: per-player saved overrides must not introduce
    // subtle differences in panel size or relative group placement.
    rect(&format!("cpu_{}_artwork",index+1),Rect{x:8.0+index as f32*636.0,y:72.0,w:632.0,h:376.0})
}
// Canonical two-slot CPU geometry. Both slots are identical, even when
// one slot is empty or the phase has a single requirement.
pub(super) fn cpu_group(player:usize,group:usize,_count:usize,_y:f32)->Rect {
    let panel=cpu_panel(player);
    let inset=8.0;
    let gap=6.0;
    let width=(panel.w-2.0*inset-gap)/2.0;
    rect(cpu_group_key(player,group),Rect {
        x:panel.x+inset+group as f32*(width+gap),
        y:panel.y+191.0,
        w:width,
        h:116.0,
    })
}
// Shared live Workshop offsets, stored as tiny layout rectangles so the
// existing F5 reload and layout::save persistence work unchanged.
pub(super) fn cpu_content_offset(label:bool)->(f32,f32) {
    let key=if label {"cpu_label_offset"}else{"cpu_cards_offset"};
    let r=rect(key,Rect{x:0.0,y:0.0,w:1.0,h:1.0});
    (r.x,r.y)
}
// Optical correction for the asymmetric neon frame: the outer rails are
// heavier than the shared center rail. Keep logical boxes untouched.
pub(super) fn cpu_group_visual_x(group:usize,neon:bool)->f32 {
    if !neon {0.0} else if group==0 {4.0} else {-4.0}
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
    let drawn=scene_items();
    if !drawn.is_empty(){return drawn;}
    let mut items:Vec<(String,Rect)>=editor_items().into_iter().map(|(n,r)|(n.to_owned(),r)).collect();
    let sh=crate::game_canvas_height();
    let groups=crate::PHASES[g.players[0].phase.min(crate::PHASES.len()-1)];
    let stage_w=if groups.len()==1 {800.0}else{680.0};
    let stage_left=(1920.0-(groups.len() as f32*stage_w+(groups.len().saturating_sub(1)) as f32*6.0))/2.0;
    for group in 0..groups.len().min(2){
        let base=rect(&stage_key(group,groups.len(),""),Rect{x:stage_left+group as f32*(stage_w+6.0),y:sh-347.0,w:stage_w,h:117.0});
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
    let default=designer::mapped(name).unwrap_or(default); record(name,default);
    OVERRIDES.with(|m|m.borrow().get(name).copied().unwrap_or(default))
}

pub(super) fn set(name:&str,value:Rect){set_many(&[(name.to_owned(),value)]);}

// Staging card overrides use world coordinates. Carry existing individual
// adjustments when their containing area moves, without moving them twice
// when parent and child are both explicitly selected in one operation.
pub(super) fn set_many(changes:&[(String,Rect)]){
    let defaults=scene_items();
    OVERRIDES.with(|m|{
        let mut values=m.borrow_mut();
        let moves:Vec<_>=changes.iter().filter_map(|(name,value)|{
            let previous=values.get(name).copied()
                .or_else(||defaults.iter().find(|(n,_)|n==name).map(|(_,r)|*r))?;
            Some((name,value.x-previous.x,value.y-previous.y))
        }).collect();
        for (child,r) in values.iter_mut(){
            if changes.iter().any(|(name,_)|name==child){continue;}
            // The longest matching name is the nearest moved ancestor.
            if let Some((_,dx,dy))=moves.iter().filter(|(parent,_,_)|staging_descendant(parent,child))
                .max_by_key(|(parent,_,_)|parent.len()) {
                r.x+=dx;r.y+=dy;
            }
        }
        for (name,value) in changes {values.insert(name.clone(),*value);}
    });
}
fn staging_descendant(parent:&str,child:&str)->bool {
    for panel in ["stage_1","stage_2","stage_single_1","stage_double_1","stage_double_2"] {
        let area=format!("{panel}_cards");
        let card_prefix=format!("{panel}_card_");
        let row=format!("{panel}_row_offset");
        if parent==panel {return child==area || child==row || child.starts_with(&card_prefix);}
        if parent==area {return child==row || child.starts_with(&card_prefix);}
        if parent==row {return child.starts_with(&card_prefix);}
    }
    false
}

#[cfg(not(target_os="android"))]
pub(super) fn save()->Result<(),String>{ if designer::active(){return Err("Save this layout in the standalone UI Designer".into());}
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
pub(super) fn clear_all()->Result<(),String>{ if designer::active(){return Err("Reset this layout in the standalone UI Designer".into());}
    let path="assets/layouts/game_16_10.json";
    let root=serde_json::json!({"schema_version":1,"canvas":{"width":1920,"height":1200},"elements":{}});
    std::fs::create_dir_all("assets/layouts").map_err(|e|e.to_string())?;
    std::fs::write(path,format!("{}\n",serde_json::to_string_pretty(&root).map_err(|e|e.to_string())?))
        .map_err(|e|e.to_string())?;
    OVERRIDES.with(|m|m.borrow_mut().clear());
    Ok(())
}

// Accept legacy files that ended in a literal backslash-n due to an older save bug.
fn parse_layout(data:&str)->Option<serde_json::Value>{
    serde_json::from_str(data).ok().or_else(||{
        let trimmed=data.trim_end();
        let repaired=trimmed.strip_suffix("\\n")?;
        serde_json::from_str(repaired).ok()
    })
}

pub(super) fn reload(){
    let mut values=HashMap::new(); designer::load(None);
    #[cfg(not(target_os="android"))]
    let data=std::fs::read_to_string(crate::selected_layout_file()).ok()
        .or_else(||std::fs::read_to_string("assets/layouts/game_16_10.json").ok());
    #[cfg(target_os="android")]
    let data=Some(match crate::selected_layout_file().as_str(){
"assets/layouts/prototype_mobile_compact_20_9.json"=>include_str!("../assets/layouts/prototype_mobile_compact_20_9.json"),
"assets/layouts/prototype_mobile_compact_19_5_9.json"=>include_str!("../assets/layouts/prototype_mobile_compact_19_5_9.json"),
_=>include_str!("../assets/layouts/game_16_10.json")}.to_owned());
    if let Some(data)=data {
        if let Some(json)=parse_layout(&data) { designer::load(Some(&json));
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
    if !values.contains_key("draw_count_offset") {
        if let Some(offset)=values.get("draw_text_offset").copied() {
            values.insert("draw_count_offset".to_owned(),offset);
        }
    }
    migrate_mode_layouts(&mut values);
    OVERRIDES.with(|m|*m.borrow_mut()=values);
}

pub(super) fn pile_offset(discard:bool,label:bool)->(f32,f32) {
    let key=match (discard,label) {(false,false)=>"draw_pile_offset",(false,true)=>"draw_text_offset",(true,false)=>"discard_pile_offset",(true,true)=>"discard_text_offset"};
    let r=rect(key,Rect{x:0.,y:0.,w:1.,h:1.});
    (r.x,r.y)
}
// Shared by rendering, click/drag hitboxes, and card animation endpoints.
pub(super) fn pile_position(discard:bool)->(f32,f32) { if let Some(r)=designer::element(if discard{"discard"}else{"draw"}){let h=r.h.min(r.w/0.68);return (r.x+(r.w-h*0.68)/2.,r.y+(r.h-h)/2.);}
    let (dx,dy)=pile_offset(discard,false);
    let y=if crate::active_theme()==crate::Theme::Futuristic {460.}else{485.};
    let r=rect(if discard {"discard_pile"}else{"draw_pile"},Rect{x:if discard {1048.+dx}else{760.+dx},y:y+dy,w:112.,h:150.});
    (r.x,r.y)
}
// Preserve the previously shared text adjustment when loading older layouts.
// Once loaded, the count has its own offset independent of the draw title.
pub(super) fn draw_count_offset()->(f32,f32) {
    let r=rect("draw_count_offset",Rect{x:0.,y:0.,w:1.,h:1.});
    (r.x,r.y)
}
// Actual rendered elements, refreshed each board frame for the Workshop.
thread_local! {
    static SCENE: RefCell<Vec<(String,Rect)>> = const {RefCell::new(Vec::new())};
    static CAPTURE: std::cell::Cell<bool> = const {std::cell::Cell::new(false)};
    static COUNTERS: RefCell<HashMap<String,usize>> = RefCell::new(HashMap::new());
}
pub(super) fn begin_scene(){SCENE.with(|v|v.borrow_mut().clear());COUNTERS.with(|v|v.borrow_mut().clear());CAPTURE.with(|v|v.set(true));}
pub(super) fn end_scene(){CAPTURE.with(|v|v.set(false));}
pub(super) fn capturing()->bool {CAPTURE.with(|v|v.get())}
pub(super) fn scene_items()->Vec<(String,Rect)> {SCENE.with(|v|v.borrow().clone())}
pub(super) fn scene_key(base:&str)->String {
    COUNTERS.with(|v|{let mut v=v.borrow_mut();let count=v.entry(base.to_owned()).or_insert(0);let key=format!("{base}_{}",*count+1);*count+=1;key})
}
fn record(name:&str,default:Rect){
    if capturing() && !name.ends_with("_offset") {
        SCENE.with(|v|{let mut v=v.borrow_mut();if !v.iter().any(|(n,_)|n==name){v.push((name.to_owned(),default));}});
    }
}

pub(super) fn adjusted(mut r:Rect,dx:f32,dy:f32,resize:bool)->Rect {
    if resize {r.w=(r.w+dx).max(8.);r.h=(r.h+dy).max(8.);}
    else {r.x+=dx;r.y+=dy;}
    r
}

// The hand box is available space, not a request to distribute cards edge to
// edge. Keep the normal 81px maximum step and center the resulting compact row.
pub(super) fn compact_hand(count:usize,area:Rect)->(f32,f32,f32){
    let width=(area.h*75./105.).min(75.).min(area.w/count.max(1) as f32).max(0.);
    let step=if count>1 {((area.w-width).max(0.)/(count-1) as f32).min(81.)}else{0.};
    let row_width=if count==0 {0.}else{width+step*(count-1) as f32};
    (area.x+(area.w-row_width)/2.,width,step)
}
pub(super) fn hand_layout(count:usize)->(Rect,f32){ if let Some(row)=designer::hand(count){return row;}
    let panel=rect("hand_panel",Rect{x:12.,y:971.,w:1896.,h:217.});
    let width=(1872./count.max(1) as f32).min(75.);
    let step=if count>1 {((1872.-width)/(count-1) as f32).min(81.)}else{0.};
    let row_width=width+step*count.saturating_sub(1) as f32;
    let area=rect("hand_cards",Rect{x:(1920.-row_width)/2.,y:panel.y+63.,w:row_width,h:105.});
    let (x,w,step)=compact_hand(count,area);
    (Rect{x,y:area.y,w,h:105.},step)
}

pub(super) fn centered_staging(count:usize,area:Rect)->(f32,f32){
    let step=if count>1 {((area.w-50.).max(0.)/(count-1) as f32).min(57.)}else{57.};
    let row_width=if count==0 {0.}else{50.+step*(count-1) as f32};
    (area.x+(area.w-row_width).max(0.)/2.,step)
}
pub(super) fn staging_row(group:usize,count:usize,area:Rect,requirements:usize)->(f32,f32){
    if designer::active(){let key=format!("{}_{}_group_{}",if designer::separate(){"staging"}else{"phase"},requirements,group+1);let (r,step)=designer::row(&key,area,count);return (r.x,step);} let (start,step)=centered_staging(count,area);
    let key=stage_key(group,requirements,"_row_offset");
    // Older saved individual cards were anchored to the left of the area.
    let old=rect(&key,Rect{x:area.x,y:area.y,w:57.,h:1.});
    set(&key,old);
    set(&key,Rect{x:start,y:area.y,w:step.max(0.001),h:1.});
    if (step-old.w).abs()>0.001 {
        let prefix=stage_key(group,requirements,"_card_");
        OVERRIDES.with(|values|for (name,r) in values.borrow_mut().iter_mut(){
            if let Some(index)=name.strip_prefix(&prefix).and_then(|v|v.parse::<usize>().ok()){
                r.x+=index.saturating_sub(1) as f32*(step-old.w);
            }
        });
    }
    (start,step)
}
pub(super) fn staging_card(group:usize,index:usize,count:usize,amount:usize)->Rect {
    if designer::active(){let key=format!("{}_{}_group_{}",if designer::separate(){"staging"}else{"phase"},count,group+1); if let Some(area)=designer::element(&key){let (r,step)=designer::row(&key,designer::content(area),amount);return Rect{x:r.x+index as f32*step,..r};}} let width=if count==1 {800.}else{680.};
    let left=(1920.-(count as f32*width+count.saturating_sub(1) as f32*6.))/2.;
    let panel=rect(&stage_key(group,count,""),Rect{x:left+group as f32*(width+6.),y:853.,w:width,h:117.});
    let area=rect(&stage_key(group,count,"_cards"),Rect{x:panel.x+10.,y:panel.y+29.,w:panel.w-20.,h:76.});
    let (start,step)=staging_row(group,amount,area,count);
    rect(&stage_key(group,count,&format!("_card_{}",index+1)),Rect{x:start+index as f32*step,y:area.y,w:50.,h:area.h})
}

#[cfg(test)]
mod staging_parent_tests {
    use super::*;
    fn setup()->(Rect,Rect,Rect){
        OVERRIDES.with(|v|v.borrow_mut().clear());
        begin_scene();
        let first=staging_card(0,0,2,3);
        let second=staging_card(0,1,2,3);
        let area=scene_items().into_iter().find(|(name,_)|name=="stage_double_1_cards").unwrap().1;
        end_scene();
        (area,first,second)
    }
    #[test]
    fn staging_parent_move_carries_adjusted_and_unadjusted_cards(){
        let (area,first,second)=setup();
        set("stage_double_1_card_1",adjusted(first,12.,7.,false));
        set("stage_double_1_cards",adjusted(area,20.,10.,false));
        let moved=staging_card(0,0,2,3);let untouched=staging_card(0,1,2,3);
        assert_eq!((moved.x,moved.y),(first.x+32.,first.y+17.));
        assert_eq!((untouched.x,untouched.y),(second.x+20.,second.y+10.));
        set("stage_double_1_cards",area);
        let restored=staging_card(0,0,2,3);
        assert_eq!((restored.x,restored.y),(first.x+12.,first.y+7.));
    }
    #[test]
    fn staging_parent_child_batch_moves_once_in_either_order(){
        for child_first in [false,true] {
            let (area,first,_)=setup();
            let custom=adjusted(first,12.,7.,false);
            set("stage_double_1_card_1",custom);
            let mut changes=vec![("stage_double_1_cards".to_owned(),adjusted(area,20.,10.,false)),
                ("stage_double_1_card_1".to_owned(),adjusted(custom,20.,10.,false))];
            if child_first {changes.reverse();}
            set_many(&changes);
            let result=staging_card(0,0,2,3);
            assert_eq!((result.x,result.y),(custom.x+20.,custom.y+10.));
            set_many(&[("stage_double_1_cards".to_owned(),area),("stage_double_1_card_1".to_owned(),custom)]);
            let restored=staging_card(0,0,2,3);
            assert_eq!((restored.x,restored.y),(custom.x,custom.y));
        }
    }
    #[test]
    fn staging_resize_recenters_while_preserving_individual_offsets(){
        let (area,first,_)=setup();
        let custom=adjusted(first,12.,7.,false);
        set("stage_double_1_card_1",custom);
        set("stage_double_1_cards",adjusted(area,30.,20.,true));
        let result=staging_card(0,0,2,3);
        assert_eq!((result.x,result.y,result.w,result.h),(custom.x+15.,custom.y,custom.w,custom.h));
    }
}

#[cfg(test)]
mod centered_staging_tests {
    use super::*;
    #[test]
    fn staging_rows_stay_centered_for_different_counts(){
        let area=Rect{x:100.,y:800.,w:660.,h:76.};
        for count in [0,1,3,7,12,24,40] {
            let (start,step)=centered_staging(count,area);
            let width=if count==0 {0.}else{50.+step*(count-1) as f32};
            assert!((start+width/2.-(area.x+area.w/2.)).abs()<0.01);
            assert!(step<=57. && width<=area.w+0.01);
        }
    }
    #[test]
    fn changing_staged_count_carries_saved_individual_adjustments(){
        OVERRIDES.with(|v|v.borrow_mut().clear());begin_scene();
        let original=staging_card(0,0,2,3);end_scene();
        set("stage_double_1_card_1",adjusted(original,12.,7.,false));
        let four=staging_card(0,0,2,4);
        assert_eq!((four.x,four.y),(original.x+12.-28.5,original.y+7.));
        let three=staging_card(0,0,2,3);
        assert_eq!((three.x,three.y),(original.x+12.,original.y+7.));
    }
}

pub(super) fn stage_key(group:usize,requirements:usize,suffix:&str)->String {
    format!("stage_{}_{}{suffix}",if requirements==1 {"single"}else{"double"},group+1)
}
pub(super) fn staging_label_key(group:usize,requirements:usize,ready:bool)->String {
    format!("staging_{}_{}label_{}",if requirements==1 {"single"}else{"double"},if ready {"ready_"}else{""},group+1)
}

fn migrate_mode_layouts(values:&mut HashMap<String,Rect>){
    let old=values.clone();
    for (name,r) in &old {
        for group in 1..=2 {
            let prefix=format!("stage_{group}");
            if name==&prefix || name.starts_with(&format!("{prefix}_")) {
                let new=format!("stage_double_{group}{}",&name[prefix.len()..]);
                values.entry(new).or_insert(*r);
            }
        }
        if let Some(index)=name.strip_prefix("staging_label_"){
            values.entry(format!("staging_double_label_{index}")).or_insert(*r);
        }
    }
    if let Some(panel)=old.get("player_phase") {
        values.entry("player_phase_single".to_owned()).or_insert(*panel);
        values.entry("player_phase_double".to_owned()).or_insert(*panel);
    }
}

#[cfg(test)]
mod staging_mode_tests {
    use super::*;
    #[test]
    fn single_and_double_staging_moves_are_independent(){
        OVERRIDES.with(|v|v.borrow_mut().clear());begin_scene();
        let single=staging_card(0,0,1,3);
        let double=staging_card(0,0,2,3);end_scene();
        let area=rect("stage_single_1_cards",Rect{x:570.,y:882.,w:780.,h:76.});
        set("stage_single_1_card_1",single);
        set("stage_double_1_card_1",double);
        set("stage_single_1_cards",adjusted(area,50.,10.,false));
        let moved=staging_card(0,0,1,3);
        let unchanged=staging_card(0,0,2,3);
        assert_eq!((moved.x,moved.y),(single.x+50.,single.y+10.));
        assert_eq!((unchanged.x,unchanged.y),(double.x,double.y));
    }
    #[test]
    fn legacy_layout_migrates_only_to_double_and_preserves_explicit_layouts(){
        let original=Rect{x:279.,y:839.,w:680.,h:134.};
        let restored=Rect{x:280.,..original};
        let single=Rect{x:620.,..original};
        let mut values=HashMap::from([("stage_1".to_owned(),original),
            ("stage_double_1".to_owned(),restored),("stage_single_1".to_owned(),single)]);
        migrate_mode_layouts(&mut values);
        assert_eq!(values["stage_double_1"].x,280.);
        assert_eq!(values["stage_single_1"].x,620.);
        values.remove("stage_single_1");migrate_mode_layouts(&mut values);
        assert!(!values.contains_key("stage_single_1"));
    }
}
