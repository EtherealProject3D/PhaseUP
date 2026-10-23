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
            items.push((name,Rect{x:32.+(i-1) as f32*632.+j as f32*312.,y:263.,w:305.,h:116.}));
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
