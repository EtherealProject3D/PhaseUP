// Developer layout overrides. Desktop reads an editable JSON file; Android uses defaults.
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone,Copy)]
pub(super) struct Rect { pub x:f32,pub y:f32,pub w:f32,pub h:f32 }
thread_local! { static OVERRIDES: RefCell<HashMap<String,Rect>> = RefCell::new(HashMap::new()); }

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
        if let Ok(json)=parse_layout(&data) {
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
