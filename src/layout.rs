// Developer layout overrides. Desktop reads an editable JSON file; Android uses defaults.
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone,Copy)]
pub(super) struct Rect { pub x:f32,pub y:f32,pub w:f32,pub h:f32 }
thread_local! { static OVERRIDES: RefCell<HashMap<String,Rect>> = RefCell::new(HashMap::new()); }

pub(super) fn rect(name:&str, default:Rect)->Rect {
    OVERRIDES.with(|m|m.borrow().get(name).copied().unwrap_or(default))
}

pub(super) fn reload(){
    let mut values=HashMap::new();
    #[cfg(not(target_os="android"))]
    if let Ok(data)=std::fs::read_to_string("assets/layouts/game_16_10.json") {
        if let Ok(json)=serde_json::from_str::<serde_json::Value>(&data) {
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
