#[path = "../card_model.rs"]
mod card_model;
use card_model::{Card, Kind};
#[path = "../phases.rs"]
#[allow(dead_code)]
mod phases;
#[path = "../designer_preview.rs"]
mod designer_preview;
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use std::{collections::{BTreeMap,BTreeSet},fs,path::PathBuf};

const BOARD:Vec2=Vec2::new(1920.0,1200.0);
const PATH:&str="assets/layouts/prototype_mobile_compact_20_9.json";
const ASPECTS:[(&str,&str,f32);4]=[("20:9","20_9",20./9.),("19.5:9","19_5_9",19.5/9.),("16:9","16_9",16./9.),("16:10","16_10",16./10.)];
fn device_aspect(desktop:bool,aspect:usize)->usize {
 if desktop {if aspect>=2&&aspect<4{aspect}else{3}}else{if aspect<2{aspect}else{0}}
}
fn canvas(desktop:bool,aspect:usize)->Vec2 {Vec2::new(1200.*ASPECTS[device_aspect(desktop,aspect)].2,1200.)}
fn rescale(map:&mut Snapshot,from:Vec2,to:Vec2){for b in map.values_mut(){b.x*=to.x/from.x;b.w*=to.x/from.x;b.y*=to.y/from.y;b.h*=to.y/from.y;}}
#[derive(Clone, Default)]
struct LabelStyle {text:String,size:Option<f32>,dx:f32,dy:f32}
fn profile_path(desktop:bool,separate:bool,aspect:usize)->PathBuf {
 format!("assets/layouts/prototype_{}_{}_{}.json",if desktop{"desktop"}else{"mobile"},if desktop&&separate{"staging"}else{"compact"},ASPECTS[device_aspect(desktop,aspect)].1).into()
}
#[derive(Clone,Copy,Debug)]
struct Box2{x:f32,y:f32,w:f32,h:f32}
impl Box2 {
 fn from_json(v:&serde_json::Value)->Option<Self>{Some(Self{x:v["x"].as_f64()? as f32,y:v["y"].as_f64()? as f32,w:v["width"].as_f64()? as f32,h:v["height"].as_f64()? as f32})}
 fn json(self)->serde_json::Value{serde_json::json!({"x":self.x.round(),"y":self.y.round(),"width":self.w.round(),"height":self.h.round()})}
 fn rect(self)->Rect{Rect::from_min_size(Pos2::new(self.x,self.y),Vec2::new(self.w,self.h))}
}
fn wheel_drag_value(
    ui: &mut egui::Ui,
    value: &mut f32,
    min: f32,
    max: f32,
) -> bool {
    // Preserve the original value unless the user actually edits it.
    let original = *value;
    let mut displayed = original.round().clamp(min, max);

    let response = ui.add(
        egui::DragValue::new(&mut displayed)
            .speed(1.0)
            .range(min..=max)
            .fixed_decimals(0)
    );

    // Record the numeric control's screen rectangle for the Properties
    // ScrollArea to test against the pointer on the next frame.
    ui.ctx().data_mut(|data| {
        let id = egui::Id::new("phaseup_numeric_rects");
        let mut rects = data.get_temp::<Vec<Rect>>(id).unwrap_or_default();
        rects.push(response.rect);
        data.insert_temp(id, rects);
    });

    // A wheel over this control is an intentional one-pixel adjustment.
    // Merely displaying a rounded number must not modify the stored value.
    let wheel = if response.hovered() {
        ui.input(|i| i.raw_scroll_delta.y)
    } else {
        0.0
    };
    if wheel.abs() > 0.0 {
        let step = if wheel > 0.0 { 1.0 } else { -1.0 };
        *value = (original.round() + step).clamp(min, max);
        (*value - original).abs() > 0.001
    } else if response.changed() {
        *value = displayed;
        (*value - original).abs() > 0.001
    } else {
        false
    }
}
fn anchored_resize(old:Box2,w:f32,h:f32,anchor:usize)->Box2 {
 let ax=match anchor{1=>0.,2=>1.,_=>0.5};
 let ay=match anchor{3=>0.,4=>1.,_=>0.5};
 Box2{x:old.x+(old.w-w)*ax,y:old.y+(old.h-h)*ay,w,h}
}
fn defaults()->BTreeMap<String,Box2>{
 designer_preview::layout(false)
}

type Snapshot=BTreeMap<String,Box2>;
#[derive(Clone)]
enum SizeSync {
 Elements(Vec<(String,f32,f32)>),
 Hand(String,f32),
}

struct Designer {
 elements:Snapshot, selected:BTreeSet<String>,undo:Vec<Snapshot>,redo:Vec<Snapshot>,
 zoom:f32,pan:Vec2,grid:f32,snap_grid:bool,snap_edges:bool,snap_objects:bool,
 drag_start:Option<(Pos2,Snapshot)>,marquee:Option<Pos2>,status:String,
 path:PathBuf, preview:designer_preview::Preview, play:bool, separate:bool, center_guides:bool, desktop:bool,show_cpu:bool,cpu_laid:bool,texts:BTreeMap<String,LabelStyle>,cpu_expanded:Snapshot,cpu_collapsed:Snapshot,aspect:usize,hand_styles:designer_preview::HandStyles,selected_card:Option<(String,usize)>,resize_anchor:usize,hand_undo:Vec<designer_preview::HandStyles>,hand_redo:Vec<designer_preview::HandStyles>,card_resize:Option<(f32,designer_preview::HandStyle)>,card_move:Option<(Pos2,designer_preview::HandStyles)>,cpu_counts:BTreeMap<String,usize>,grid_vertical:bool,grid_horizontal:bool,grid_columns:usize,grid_rows:usize,
 sync_draft:Option<SizeSync>,sync_last_change:f64,sync_prompt:Option<SizeSync>,sync_owner:Option<String>
}
impl Designer {
 fn new()->Self{
  let mut app=Self{elements:defaults(),selected:BTreeSet::new(),undo:vec![],redo:vec![],
   zoom:0.0,pan:Vec2::ZERO,grid:8.,snap_grid:true,snap_edges:true,snap_objects:true,
   drag_start:None,marquee:None,status:String::new(),path:PATH.into(),preview:designer_preview::Preview::new(0),play:true,separate:false,center_guides:true,desktop:false,show_cpu:true,cpu_laid:false,texts:BTreeMap::new(),cpu_expanded:Snapshot::new(),cpu_collapsed:Snapshot::new(),aspect:0,hand_styles:Default::default(),selected_card:None,resize_anchor:0,hand_undo:vec![],hand_redo:vec![],card_resize:None,card_move:None,cpu_counts:BTreeMap::new(),grid_vertical:false,grid_horizontal:false,grid_columns:4,grid_rows:2,sync_draft:None,sync_last_change:0.,sync_prompt:None,sync_owner:None};
  app.load();app
 }
 fn board_size(&self)->Vec2{canvas(self.desktop,self.aspect)}
 fn queue_size_sync(&mut self,change:SizeSync,now:f64){
  if self.desktop{return;}
  self.sync_last_change=now;
  self.sync_owner=self.selected_card.as_ref().map(|(key,_)|key.clone())
   .or_else(||self.selected.iter().next().cloned());
  match (&mut self.sync_draft,change){
   (Some(SizeSync::Elements(existing)),SizeSync::Elements(changes))=>{
    for (name,w,h) in changes{
     if let Some(item)=existing.iter_mut().find(|item|item.0==name){item.1=w;item.2=h;}
     else{existing.push((name,w,h));}
    }
   }
   (_,change)=>self.sync_draft=Some(change),
  }
 }
 fn apply_size_sync(&mut self,change:SizeSync)->Result<String,String>{
  if self.desktop{return Err("Size sync is only available for mobile profiles".into());}
  let target_aspect=if self.aspect==0{1}else{0};
  let path=profile_path(false,false,target_aspect);
  if !path.exists(){return Err(format!("Other mobile profile does not exist: {}",path.display()));}
  let data=fs::read_to_string(&path).map_err(|e|e.to_string())?;
  let mut root:serde_json::Value=serde_json::from_str(&data).map_err(|e|e.to_string())?;
  match change{
   SizeSync::Elements(items)=>{
    let elements=root["elements"].as_object_mut().ok_or("Other profile has no elements object")?;
    for (name,w,h) in items{
     let entry=elements.get_mut(&name).ok_or_else(||format!("Element {name} missing from other profile"))?;
     if !entry.is_object(){return Err(format!("Element {name} is not an object"));}
     entry["width"]=serde_json::json!(w.round());
     entry["height"]=serde_json::json!(h.round());
    }
   }
   SizeSync::Hand(key,height)=>{
    if !root["hand_card_styles"].is_object(){root["hand_card_styles"]=serde_json::json!({});}
    if !root["hand_card_styles"][&key].is_object(){root["hand_card_styles"][&key]=serde_json::json!({"dx":0,"dy":0});}
    root["hand_card_styles"][&key]["height"]=serde_json::json!(height.round());
   }
  }
  // Keep a recoverable copy of the destination profile before replacing it.
  let backup=path.with_extension("json.size-sync-backup");
  fs::copy(&path,&backup).map_err(|e|format!("Backup failed: {e}"))?;
  let output=format!("{}\n",serde_json::to_string_pretty(&root).map_err(|e|e.to_string())?);
  fs::write(&path,output).map_err(|e|e.to_string())?;
  Ok(format!("Applied size to {} (backup: {})",ASPECTS[target_aspect].0,backup.display()))
 }

 fn load(&mut self){
  if !self.desktop {self.separate=false;self.path=profile_path(false,false,self.aspect);}
  self.sync_draft=None;self.sync_prompt=None;self.sync_last_change=0.0;self.sync_owner=None;
  self.hand_styles.clear();self.selected_card=None;self.card_move=None;self.hand_undo.clear();self.hand_redo.clear();self.cpu_counts.clear();
   for cpu in 1..=3 {
    for group in 1..=2 {
     let key=format!("cpu_{cpu}_laid_{group}");
     self.cpu_counts.insert(key,3);
    }
   }
  let target=self.board_size();let mut source_size=BOARD;
  let legacy=PathBuf::from(if self.separate{"assets/layouts/prototype_staging_16_10.json"}else{"assets/layouts/prototype_compact_16_10.json"});
  let previous:PathBuf=format!("assets/layouts/prototype_{}_{}_16_10.json",if self.desktop{"desktop"}else{"mobile"},if self.separate{"staging"}else{"compact"}).into();
  let source=if self.path.exists(){&self.path}else if previous.exists(){&previous}else if !self.desktop{&legacy}else{&self.path};
  self.elements=designer_preview::layout(self.separate);self.texts.clear();self.show_cpu=true;self.cpu_expanded.clear();self.cpu_collapsed.clear();
  match fs::read_to_string(source){
   Ok(s)=>{
    let trimmed=s.trim_end();
    let parsed=serde_json::from_str::<serde_json::Value>(&s).or_else(|_|serde_json::from_str(trimmed.strip_suffix("\\n").unwrap_or(trimmed)));
    match parsed {
     Ok(root)=>{source_size=Vec2::new(root["canvas"]["width"].as_f64().unwrap_or(1920.) as f32,root["canvas"]["height"].as_f64().unwrap_or(1200.) as f32);
     if source_size.x<=0.||source_size.y<=0.{source_size=BOARD;}
     if let Some(map)=root["elements"].as_object(){
      for (name,v) in map {if let Some(b)=Box2::from_json(v){self.elements.insert(name.clone(),b);}}
     }
     // Derive newly editable children from the user's saved parent positions.
     for key in self.elements.keys().cloned().collect::<Vec<_>>() {if designer_preview::parent(&key).is_some(){self.elements.remove(&key);}}
     designer_preview::add_children(&mut self.elements);
     if let Some(map)=root["elements"].as_object(){for (name,v) in map{if let Some(b)=Box2::from_json(v){self.elements.insert(name.clone(),b);}}}
     if let Some(map)=root["labels"].as_object(){for (key,v) in map{self.texts.insert(key.clone(),LabelStyle{text:v["text"].as_str().unwrap_or("").into(),size:v["size"].as_f64().map(|n|n as f32),dx:v["dx"].as_f64().unwrap_or(0.) as f32,dy:v["dy"].as_f64().unwrap_or(0.) as f32});}}
     if let Some(map)=root["hand_card_styles"].as_object(){for (key,v) in map {
      self.hand_styles.insert(key.clone(),designer_preview::HandStyle{height:v["height"].as_f64().filter(|h|h.is_finite()&&*h>=10.&&*h<=1000.).map(|h|h as f32),dx:v["dx"].as_f64().unwrap_or(0.) as f32,dy:v["dy"].as_f64().unwrap_or(0.) as f32});
     }}
     if root["guides"].is_object(){self.grid_vertical=root["guides"]["vertical"].as_bool().unwrap_or(false);self.grid_horizontal=root["guides"]["horizontal"].as_bool().unwrap_or(false);self.grid_columns=(root["guides"]["columns"].as_u64().unwrap_or(4) as usize).clamp(1,32);self.grid_rows=(root["guides"]["rows"].as_u64().unwrap_or(2) as usize).clamp(1,32);}
     self.show_cpu=root["show_cpu_cards"].as_bool().unwrap_or(true);
      if let Some(map)=root["cpu_preview_counts"].as_object(){
       for (key,value) in map {
        if let Some(count)=value.as_u64(){
         if self.cpu_counts.contains_key(key){
          self.cpu_counts.insert(key.clone(),(count as usize).min(12));
         }
        }
       }
      }
     for (key,target) in [("cpu_expanded",&mut self.cpu_expanded),("cpu_collapsed",&mut self.cpu_collapsed)] {
      if let Some(map)=root[key].as_object(){for (n,v) in map{if n.starts_with("cpu_"){if let Some(b)=Box2::from_json(v){target.insert(n.clone(),b);}}}}
     }
     if !self.show_cpu && self.cpu_collapsed.is_empty(){
      self.cpu_expanded=self.elements.iter().filter(|(n,_)|n.starts_with("cpu_")).map(|(n,b)|(n.clone(),*b)).collect();
      designer_preview::collapse_cpu_rows(&mut self.elements);
     }
     self.status=format!("Loaded {}; saving to {}",source.display(),self.path.display());}
     Err(e)=>self.status=format!("Load failed: {e}")
    }
   }
   Err(_)=>self.status="Using default layout".into()
  }
  rescale(&mut self.elements,source_size,target);
  rescale(&mut self.cpu_expanded,source_size,target);
  rescale(&mut self.cpu_collapsed,source_size,target);
  if !self.desktop {
    // Remove the mobile-only header. Existing desktop layouts are untouched.
    // On the first load, reclaim its vertical band for the CPU panels.
    if let Some(header)=self.elements.remove("header"){
     let shift=header.h.min(72.);
     for (name,b) in self.elements.iter_mut(){
      if name.starts_with("cpu_"){
       b.y=(b.y-shift).max(0.);
      }
     }
     for map in [&mut self.cpu_expanded,&mut self.cpu_collapsed]{
      for (name,b) in map.iter_mut(){
       if name.starts_with("cpu_"){
        b.y=(b.y-shift).max(0.);
       }
      }
     }
    }
    self.toggle_cpu_cards(false);
   }
 }
 fn remember_cpu_layout(&mut self){
  let current=self.elements.iter().filter(|(n,_)|n.starts_with("cpu_")).map(|(n,b)|(n.clone(),*b)).collect();
  if self.show_cpu{self.cpu_expanded=current;}else{self.cpu_collapsed=current;}
 }
 fn toggle_cpu_cards(&mut self,show:bool){
  let show=show&&self.desktop;
  if show==self.show_cpu{return;}
  self.remember_cpu_layout();
  let target=if show{&self.cpu_expanded}else{&self.cpu_collapsed};
  if target.is_empty(){designer_preview::collapse_cpu_rows(&mut self.elements);}
  else{for (n,b) in target{self.elements.insert(n.clone(),*b);}}
  self.show_cpu=show;self.selected.clear();self.undo.clear();self.redo.clear();
 }
 fn save(&mut self){
  self.remember_cpu_layout();
  let mut root=fs::read_to_string(&self.path).ok()
   .and_then(|s|serde_json::from_str::<serde_json::Value>(&s).ok())
   .unwrap_or_else(||serde_json::json!({"schema_version":1,"canvas":{"width":1920,"height":1200},"elements":{}}));
  if !root["elements"].is_object(){root["elements"]=serde_json::json!({});}
  for (n,b) in &self.elements{root["elements"][n]=b.json();}
   if !self.desktop {
    if let Some(map)=root["elements"].as_object_mut(){
     map.remove("header");
    }
   }
  root["hand_card_styles"]=serde_json::Value::Object(self.hand_styles.iter().map(|(key,s)|(key.clone(),serde_json::json!({"height":s.height.map(|v|v.round()),"dx":s.dx.round(),"dy":s.dy.round()}))).collect());
  root["guides"]=serde_json::json!({"vertical":self.grid_vertical,"horizontal":self.grid_horizontal,"columns":self.grid_columns,"rows":self.grid_rows});
  root["labels"]=serde_json::Value::Object(self.texts.iter().map(|(k,v)|(k.clone(),serde_json::json!({"text":v.text,"size":v.size.map(|n|n.round()),"dx":v.dx.round(),"dy":v.dy.round()}))).collect());
  root["show_cpu_cards"]=self.show_cpu.into();
   root["cpu_preview_counts"]=serde_json::json!(self.cpu_counts);
  for (key,map) in [("cpu_expanded",&self.cpu_expanded),("cpu_collapsed",&self.cpu_collapsed)]{root[key]=serde_json::Value::Object(map.iter().map(|(n,b)|(n.clone(),b.json())).collect());}
  root["canvas"]=serde_json::json!({"width":self.board_size().x,"height":self.board_size().y});
  root["profile"]=if self.desktop{"desktop"}else{"mobile"}.into();
  let result=(||->Result<(),String>{
   if let Some(parent)=self.path.parent(){fs::create_dir_all(parent).map_err(|e|e.to_string())?;}
   fs::write(&self.path,format!("{}\n",serde_json::to_string_pretty(&root).map_err(|e|e.to_string())?)).map_err(|e|e.to_string())
  })();
  self.status=match result{Ok(())=>format!("Saved {}",self.path.display()),Err(e)=>format!("Save failed: {e}")};
 }
 fn checkpoint(&mut self){self.undo.push(self.elements.clone());if self.undo.len()>100{self.undo.remove(0);}self.redo.clear();}
 fn undo(&mut self){if self.selected_card.is_some(){if let Some(s)=self.hand_undo.pop(){self.hand_redo.push(self.hand_styles.clone());self.hand_styles=s;}return;}if let Some(s)=self.undo.pop(){self.redo.push(self.elements.clone());self.elements=s;}}
 fn redo(&mut self){if self.selected_card.is_some(){if let Some(s)=self.hand_redo.pop(){self.hand_undo.push(self.hand_styles.clone());self.hand_styles=s;}return;}if let Some(s)=self.redo.pop(){self.undo.push(self.elements.clone());self.elements=s;}}
 fn carry_children(&mut self) {
  let Some(old)=self.undo.last().cloned() else{return;};
  self.carry_from(&old);
 }
 fn carry_from(&mut self,old:&Snapshot) {
  for (name,b) in old {
   if self.selected.contains(name){continue;}
   if let Some(parent)=designer_preview::parent(name){
    if !self.selected.contains(&parent){continue;}
    if let (Some(a),Some(p))=(old.get(&parent),self.elements.get(&parent)){
     let sx=p.w/a.w;let sy=p.h/a.h;
     self.elements.insert(name.clone(),Box2{x:p.x+(b.x-a.x)*sx,y:p.y+(b.y-a.y)*sy,w:b.w*sx,h:b.h*sy});
    }
   }
  }
 }
 fn snap(&self,value:f32,axis:bool,ignore:&str)->f32{
  let mut result=value;let mut distance=7.0/self.zoom;
  if self.snap_grid {let candidate=(value/self.grid).round()*self.grid;let d=(candidate-value).abs();if d<distance{result=candidate;distance=d;}}
  if self.snap_edges{
   for target in [0.0,if axis{self.board_size().x/2.0}else{self.board_size().y/2.0},if axis{self.board_size().x}else{self.board_size().y}]{
    let d=(value-target).abs();if d<distance{result=target;distance=d;}
   }
  }
  if self.snap_objects {
   for (name,b) in &self.elements{
    if name==ignore||self.selected.contains(name){continue;}
    let targets=if axis{[b.x,b.x+b.w/2.0,b.x+b.w]}else{[b.y,b.y+b.h/2.0,b.y+b.h]};
    for target in targets{let d=(value-target).abs();if d<distance{result=target;distance=d;}}
   }
  }
  result
 }
 fn fit(&mut self,viewport:Rect){
  self.zoom=((viewport.width()-48.)/self.board_size().x).min((viewport.height()-48.)/self.board_size().y).clamp(0.1,2.0);
  self.pan=Vec2::ZERO;
 }
 fn board_rect(&self,viewport:Rect)->Rect{
  Rect::from_center_size(viewport.center()+self.pan,self.board_size()*self.zoom)
 }
 fn world(&self,screen:Pos2,board:Rect)->Pos2{
  Pos2::new((screen.x-board.min.x)/self.zoom,(screen.y-board.min.y)/self.zoom)
 }
}
impl eframe::App for Designer{
 fn update(&mut self,ctx:&egui::Context,_frame:&mut eframe::Frame){
  let undo=!self.play && !ctx.wants_keyboard_input() && ctx.input(|i|i.modifiers.command&&i.key_pressed(egui::Key::Z));
  let redo=!self.play && !ctx.wants_keyboard_input() && ctx.input(|i|i.modifiers.command&&i.key_pressed(egui::Key::Y));
  if undo{self.undo();}if redo{self.redo();}
  egui::TopBottomPanel::top("toolbar").show(ctx,|ui|{
   ui.horizontal_wrapped(|ui|{
    ui.heading("PhaseUP UI Designer");
    ui.separator();
    ui.checkbox(&mut self.play,"Play preview");
    let mut desktop=self.desktop;let mut separate=self.separate;let mut aspect=self.aspect;
    egui::ComboBox::from_id_salt("profile").selected_text(if desktop{"Desktop"}else{"Mobile"}).show_ui(ui,|ui|{ui.selectable_value(&mut desktop,false,"Mobile");ui.selectable_value(&mut desktop,true,"Desktop");});
    aspect=device_aspect(desktop,aspect);
    egui::ComboBox::from_id_salt("aspect").selected_text(format!("{} landscape",ASPECTS[aspect].0)).show_ui(ui,|ui|{
     for i in if desktop{2..4}else{0..2}{ui.selectable_value(&mut aspect,i,ASPECTS[i].0);}
    });
    if desktop {ui.checkbox(&mut separate,"Separate staging");}else{separate=false;}
    if desktop!=self.desktop||separate!=self.separate||aspect!=self.aspect {
     self.save();self.desktop=desktop;self.separate=separate;self.aspect=aspect;self.path=profile_path(desktop,separate,aspect);self.zoom=0.;
     self.selected.clear();self.undo.clear();self.redo.clear();self.load();
    }
    if ui.button("Save  Ctrl+S").clicked(){self.save();}
    if ui.button("Reload").clicked(){self.load();}
    if ui.button("Undo").clicked(){self.undo();}
    if ui.button("Redo").clicked(){self.redo();}
    if ui.button("Fit board  F").clicked(){self.zoom=0.0;}
    ui.label(format!("Zoom {:.0}%",self.zoom*100.));
   });
  });
  if ctx.input(|i|i.modifiers.command&&i.key_pressed(egui::Key::S)){self.save();}
  egui::SidePanel::left("tree").default_width(220.).min_width(170.).show(ctx,|ui|{
   ui.heading("Elements");ui.label("Ctrl+click: multi-select");
   ui.separator();
   egui::ScrollArea::vertical().show(ui,|ui|{
    let names:Vec<String>=self.elements.keys().filter(|n|self.preview.visible(n,self.show_cpu,self.separate)).cloned().collect();
    for name in names{
     let selected=self.selected.contains(&name);
     if ui.selectable_label(selected,&name).clicked(){
      self.selected_card=None;
      let multi=ui.input(|i|i.modifiers.ctrl||i.modifiers.command);
      if !multi{self.selected.clear();}
      if multi&&selected{self.selected.remove(&name);}else{self.selected.insert(name);}
     }
    }
   });
  });
  // Disable wheel scrolling only when the pointer is over a numeric editor.
  // Use rectangles from the previous frame so the ScrollArea knows its wheel
  // policy before it processes the current frame's content.
  let numeric_id=egui::Id::new("phaseup_numeric_rects");
  let numeric_rects=ctx.data(|data|data.get_temp::<Vec<Rect>>(numeric_id).unwrap_or_default());
  let over_numeric=ctx.input(|i|i.pointer.hover_pos())
   .is_some_and(|pos|numeric_rects.iter().any(|rect|rect.contains(pos)));
  ctx.data_mut(|data|data.insert_temp(numeric_id,Vec::<Rect>::new()));
  egui::SidePanel::right("properties").default_width(255.).min_width(210.).show(ctx,|ui|{
   egui::ScrollArea::vertical()
    .enable_scrolling(!over_numeric)
    .show(ui,|ui|{
   if self.desktop {let mut show=self.show_cpu;
    if ui.checkbox(&mut show,"Show CPU hand cards").changed(){self.toggle_cpu_cards(show);}
   }else{ui.label("Mobile: compact phase area | CPU hand cards hidden");}
   ui.checkbox(&mut self.cpu_laid,"Preview CPU laid phases");
    ui.collapsing("CPU laid-phase card counts",|ui|{
     ui.label("Preview only — does not change game rules.");
     for cpu in 1..=3 {
      ui.label(format!("CPU {cpu}"));
      for group in 1..=2 {
       let key=format!("cpu_{cpu}_laid_{group}");
       let count=self.cpu_counts.entry(key).or_insert(3);
       ui.horizontal(|ui|{
        ui.label(format!("Group {group}"));
        if ui.button("−").clicked(){*count=count.saturating_sub(1);}
        ui.label(format!("{count}"));
        if ui.button("+").clicked(){*count=(*count+1).min(12);}
       });
      }
     }
    });
   ui.label("Hiding hand cards collapses the CPU panels. Both arrangements are saved; card counts stay visible.");
   self.preview.controls(ui);
   ui.separator();
   ui.heading("Properties");
   ui.label(format!("{} selected",if self.selected_card.is_some(){1}else{self.selected.len()}));
   if self.selected_card.is_some()||!self.selected.is_empty(){
    egui::ComboBox::from_label("Resize anchor").selected_text(["Center","Left edge","Right edge","Top edge","Bottom edge"][self.resize_anchor]).show_ui(ui,|ui|{for (i,name) in ["Center","Left edge","Right edge","Top edge","Bottom edge"].iter().enumerate(){ui.selectable_value(&mut self.resize_anchor,i,*name);}});
   }
   if let Some((key,index))=self.selected_card.clone() {
    ui.label(format!("{key}: card {}",index+1));
    ui.label("Resizing applies to every card in this hand. Proportions are locked.");
     if key=="hand" {
      ui.label("Individual card position (pixels)");
      let card_key=format!("{key}_card_{index}");
      let original=self.hand_styles.get(&card_key).copied().unwrap_or_default();
      let mut dx=original.dx;
      let mut dy=original.dy;
      ui.horizontal(|ui|{
       ui.label("X");
       wheel_drag_value(ui,&mut dx,-10000.,10000.);
       ui.label("Y");
       wheel_drag_value(ui,&mut dy,-10000.,10000.);
      });
      if dx!=original.dx||dy!=original.dy {
       self.hand_undo.push(self.hand_styles.clone());
       self.hand_redo.clear();
       let style=self.hand_styles.entry(card_key).or_default();
       style.dx=dx;
       style.dy=dy;
      }
      if ui.button("Reset this card position").clicked(){
       self.hand_undo.push(self.hand_styles.clone());
       self.hand_redo.clear();
       self.hand_styles.remove(&format!("{key}_card_{index}"));
      }
     }

    if let Some((_,_,r))=self.preview.hand_items(&self.elements,&self.hand_styles,self.show_cpu).into_iter().find(|(k,i,_)|*k==key&&*i==index){
     let old=r.height();let mut height=old;let mut width=r.width();
     let w_changed=wheel_drag_value(ui,&mut width,6.8,680.0);
     let h_changed=wheel_drag_value(ui,&mut height,10.0,1000.0);
     if w_changed||h_changed {if w_changed{height=width/0.68;}self.hand_undo.push(self.hand_styles.clone());self.hand_redo.clear();designer_preview::resize_hand(self.hand_styles.entry(key.clone()).or_default(),old,height,self.resize_anchor);
      self.queue_size_sync(SizeSync::Hand(key.clone(),height),ctx.input(|i|i.time));}
    }
    if ui.button("Reset this hand's card size").clicked(){self.hand_undo.push(self.hand_styles.clone());self.hand_redo.clear();self.hand_styles.remove(&key);}
   }
   if self.selected.len()==1{
    let name=self.selected.iter().next().unwrap().clone();
    ui.label(&name);
    let mut b=self.elements[&name];let old=b;
    ui.label(format!("Center: X {:.1} / Y {:.1}",b.x+b.w/2.,b.y+b.h/2.));
    let is_laid_group =
        (name.starts_with("phase_") && name.contains("_group_"))
        || (name.starts_with("staging_") && name.contains("_group_"))
        || (name.starts_with("cpu_") && name.contains("_laid_"));

    if is_laid_group {
        ui.separator();
        ui.heading("Laid-phase card size");
        ui.label("Changes the cards, not the surrounding phase area.");
        ui.label("All cards in this group use the same size.");

        let original=self.hand_styles.get(&name).copied().unwrap_or_default();

        let default_height=if name.starts_with("cpu_"){
            ((b.h-34.)/1.0).max(10.).min((b.w-12.)/0.68)
        }else{
            ((b.h-34.)/1.0).max(10.).min((b.w-16.)/0.68)
        }.max(10.);

        let old_height=original.height.unwrap_or(default_height);
        let mut height=old_height;
        let mut width=height*0.68;

        let mut size_changed=false;
        ui.horizontal(|ui|{
            ui.label("Card width");
            if wheel_drag_value(ui,&mut width,6.8,680.0){
                height=(width/0.68).clamp(10.,1000.);
                size_changed=true;
            }
        });
        ui.horizontal(|ui|{
            ui.label("Card height");
            if wheel_drag_value(ui,&mut height,10.0,1000.0){
                width=height*0.68;
                size_changed=true;
            }
        });

        if size_changed && (height-old_height).abs()>0.001 {
            self.hand_undo.push(self.hand_styles.clone());
            self.hand_redo.clear();
            designer_preview::resize_hand(
                self.hand_styles.entry(name.clone()).or_default(),
                old_height,height,self.resize_anchor
            );
            self.queue_size_sync(
                SizeSync::Hand(name.clone(),height),
                ctx.input(|i|i.time)
            );
        }

        if ui.button("Reset laid-phase card size").clicked(){
            self.hand_undo.push(self.hand_styles.clone());
            self.hand_redo.clear();
            self.hand_styles.remove(&name);
        }
        ui.separator();
    }
    let mut resized=false;
    egui::Grid::new("properties_grid").show(ui,|ui|{
     ui.label("X");wheel_drag_value(ui,&mut b.x,-10000.0,10000.0);ui.end_row();
     ui.label("Y");wheel_drag_value(ui,&mut b.y,-10000.0,10000.0);ui.end_row();
     ui.label("Width");resized|=wheel_drag_value(ui,&mut b.w,1.0,self.board_size().x);ui.end_row();
     ui.label("Height");resized|=wheel_drag_value(ui,&mut b.h,1.0,self.board_size().y);ui.end_row();
    });
    if resized {let anchored=anchored_resize(old,b.w,b.h,self.resize_anchor);b.x+=anchored.x-old.x;b.y+=anchored.y-old.y;}
    if b.x!=old.x||b.y!=old.y||b.w!=old.w||b.h!=old.h{self.checkpoint();self.elements.insert(name.clone(),b);self.carry_children();
     if resized && (b.w-old.w).abs()>0.001 || resized && (b.h-old.h).abs()>0.001 {self.queue_size_sync(SizeSync::Elements(vec![(name,b.w,b.h)]),ctx.input(|i|i.time));}}
   }
   if self.selected.len()==1 {
    let name=self.selected.iter().next().unwrap().clone();
    let label=self.texts.entry(name).or_default();
    ui.heading("Text");ui.label("Blank = live game label. {value} inserts the live label.");
    ui.text_edit_singleline(&mut label.text);
    let mut size=label.size.unwrap_or(22.);
    if wheel_drag_value(ui,&mut size,8.0,80.0){label.size=Some(size);}
    wheel_drag_value(ui,&mut label.dx,-10000.0,10000.0);
    wheel_drag_value(ui,&mut label.dy,-10000.0,10000.0);
    if ui.button("Reset text").clicked(){*label=LabelStyle::default();}
   }
   if self.selected.len()>1{
    ui.label("Resize each selected object using the chosen anchor:");
    let mut dw=0.;let mut dh=0.;
    ui.horizontal(|ui|{if ui.button("Narrower").clicked(){dw=-1.;}if ui.button("Wider").clicked(){dw=1.;}});
    ui.horizontal(|ui|{if ui.button("Shorter").clicked(){dh=-1.;}if ui.button("Taller").clicked(){dh=1.;}});
    if dw!=0.||dh!=0.{self.checkpoint();for name in &self.selected{let b=self.elements[name];self.elements.insert(name.clone(),anchored_resize(b,(b.w+dw).max(1.),(b.h+dh).max(1.),self.resize_anchor));}self.carry_children();
      let changes=self.selected.iter().filter_map(|name|self.elements.get(name).map(|b|(name.clone(),b.w,b.h))).collect();
      self.queue_size_sync(SizeSync::Elements(changes),ctx.input(|i|i.time));}

    if ui.button("Align left").clicked(){self.checkpoint();let x=self.selected.iter().map(|n|self.elements[n].x).fold(f32::INFINITY,f32::min);for n in &self.selected{self.elements.get_mut(n).unwrap().x=x;}self.carry_children();}
    if ui.button("Align top").clicked(){self.checkpoint();let y=self.selected.iter().map(|n|self.elements[n].y).fold(f32::INFINITY,f32::min);for n in &self.selected{self.elements.get_mut(n).unwrap().y=y;}self.carry_children();}
    if ui.button("Center horizontally").clicked(){self.checkpoint();for n in &self.selected{let b=self.elements.get_mut(n).unwrap();b.x=(canvas(self.desktop,self.aspect).x-b.w)/2.;}self.carry_children();}
    if ui.button("Center vertically").clicked(){self.checkpoint();for n in &self.selected{let b=self.elements.get_mut(n).unwrap();b.y=(canvas(self.desktop,self.aspect).y-b.h)/2.;}self.carry_children();}
   }
   if !self.play && !self.selected.is_empty() {
    let mut delta=Vec2::ZERO;
    ui.horizontal(|ui|{if ui.button("Left").clicked(){delta.x-=1.;}if ui.button("Right").clicked(){delta.x+=1.;}if ui.button("Up").clicked(){delta.y-=1.;}if ui.button("Down").clicked(){delta.y+=1.;}});
    if !ctx.wants_keyboard_input(){ctx.input(|i|{let step=if i.modifiers.shift{10.}else{1.};for (key,d) in [(egui::Key::ArrowLeft,Vec2::new(-step,0.)),(egui::Key::ArrowRight,Vec2::new(step,0.)),(egui::Key::ArrowUp,Vec2::new(0.,-step)),(egui::Key::ArrowDown,Vec2::new(0.,step))]{if i.key_pressed(key){delta+=d;}}});}
    if delta!=Vec2::ZERO {self.checkpoint();for n in &self.selected{let b=self.elements.get_mut(n).unwrap();b.x+=delta.x;b.y+=delta.y;}self.carry_children();}
   }
   ui.separator();ui.heading("Guide grid");
   ui.checkbox(&mut self.grid_vertical,"Vertical guides");
   ui.add(egui::DragValue::new(&mut self.grid_columns).range(1..=32).prefix("Columns across "));
   ui.checkbox(&mut self.grid_horizontal,"Horizontal guides");
   ui.add(egui::DragValue::new(&mut self.grid_rows).range(1..=32).prefix("Rows down "));
   ui.label("4 columns and 2 rows divide the board into eight equal areas. These are visual guides.");
   ui.separator();ui.heading("Snapping");
   ui.checkbox(&mut self.center_guides,"Show selected centerlines");
   ui.checkbox(&mut self.snap_grid,"Grid");ui.add(egui::Slider::new(&mut self.grid,1.0..=64.0).text("Grid px"));
   ui.checkbox(&mut self.snap_edges,"Board edges / centers");
   ui.checkbox(&mut self.snap_objects,"Other objects");
   ui.separator();ui.heading("Navigation");
   ui.label("Wheel: zoom to pointer");
   ui.label("Middle-drag: pan");
   ui.label("Ctrl+click: add to selection");
   ui.label("Drag empty canvas: box select");
   ui.label("F: fit board");
   ui.label("Ctrl+Z / Ctrl+Y: undo / redo");
   ui.separator();ui.label(&self.status);
   });
  });
   // Sync is shown only when the edited item loses selection.
   if let Some(change)=self.sync_prompt.clone(){
    let other=if self.aspect==0{"19.5:9"}else{"20:9"};
    egui::Window::new("Sync mobile item size?")
     .collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER,[0.,0.])
     .show(ctx,|ui|{
      ui.label(format!("Apply the new size to the {other} mobile layout?"));
      ui.label("Only width and height will change; X/Y positions stay untouched.");
      ui.horizontal(|ui|{
       if ui.button("No, keep separate").clicked(){self.sync_prompt=None;}
       if ui.button("Yes, apply size").clicked(){
        self.save();
        if !self.status.starts_with("Save failed"){
         self.status=match self.apply_size_sync(change.clone()){
          Ok(message)=>message,Err(error)=>format!("Size sync failed: {error}")
         };
        }
        self.sync_prompt=None;
       }
      });
     });
   }
  egui::CentralPanel::default().show(ctx,|ui|{
   let viewport=ui.available_rect_before_wrap();
   let response=ui.allocate_rect(viewport,Sense::click_and_drag());
   if self.zoom==0.0||(!ctx.wants_keyboard_input()&&ctx.input(|i|i.key_pressed(egui::Key::F))){self.fit(viewport);}
   let hover=response.hover_pos();
   if response.hovered(){
    let wheel=ctx.input(|i|i.raw_scroll_delta.y);
    if wheel!=0.0 {
     let old=self.zoom;let new=(old*(wheel*0.0015).exp()).clamp(0.1,4.0);
     if let Some(pointer)=hover{
      let offset=pointer-(viewport.center()+self.pan);
      self.pan+=offset*(1.0-new/old);
     }
     self.zoom=new;
    }
    let middle=ctx.input(|i|i.pointer.button_down(egui::PointerButton::Middle));
    if middle{self.pan+=ctx.input(|i|i.pointer.delta());}
   }
   let board=self.board_rect(viewport);
   let painter=ui.painter_at(viewport);
   painter.rect_filled(viewport,0.,Color32::from_rgb(20,25,34));
   painter.rect_filled(board,0.,Color32::from_rgb(12,23,38));
   painter.rect_stroke(board,0.,Stroke::new(2.0_f32,Color32::from_rgb(80,170,225)));
   let center=board.center();
   painter.line_segment([Pos2::new(center.x,board.top()),Pos2::new(center.x,board.bottom())],Stroke::new(1.0_f32,Color32::from_rgb(36,90,95)));
   painter.line_segment([Pos2::new(board.left(),center.y),Pos2::new(board.right(),center.y)],Stroke::new(1.0_f32,Color32::from_rgb(36,90,95)));
   self.preview.draw(ui,board,self.zoom,&self.elements,self.separate,self.play,&self.texts,self.show_cpu,self.cpu_laid,&self.hand_styles,&self.cpu_counts);
   if self.play { return; }
   let grid_painter=painter.with_clip_rect(viewport.intersect(board));
   let grid_color=Color32::from_rgba_unmultiplied(150,175,215,125);
   if self.grid_vertical {for i in 1..self.grid_columns {let x=board.left()+board.width()*i as f32/self.grid_columns as f32;grid_painter.line_segment([Pos2::new(x,board.top()),Pos2::new(x,board.bottom())],Stroke::new(1.0_f32,grid_color));}}
   if self.grid_horizontal {for i in 1..self.grid_rows {let y=board.top()+board.height()*i as f32/self.grid_rows as f32;grid_painter.line_segment([Pos2::new(board.left(),y),Pos2::new(board.right(),y)],Stroke::new(1.0_f32,grid_color));}}
   let hand_items=self.preview.hand_items(&self.elements,&self.hand_styles,self.show_cpu);
   let pointer=ctx.input(|i|i.pointer.interact_pos());
   let card_hit=pointer.and_then(|pos|{let world=self.world(pos,board);hand_items.iter().rev().find(|(_,_,r)|r.contains(world)).map(|(key,index,_)|(key.clone(),*index))});
   if response.drag_started_by(egui::PointerButton::Primary) {
     if let (Some(pos),Some((key,index)))=(pointer,card_hit.as_ref()) {
      if key=="hand" {
       self.hand_undo.push(self.hand_styles.clone());
       self.hand_redo.clear();
       self.card_move=Some((pos,self.hand_styles.clone()));
       self.selected_card=Some((key.clone(),*index));
       self.selected.clear();
      }
     }
    }
    if response.clicked_by(egui::PointerButton::Primary)||response.drag_started_by(egui::PointerButton::Primary){
    self.selected_card=card_hit.clone();if card_hit.is_some(){self.selected.clear();}
   }
   if let (Some((start,original)),Some(pos),Some((key,index)))=
     (&self.card_move,pointer,&self.selected_card) {
     if response.dragged_by(egui::PointerButton::Primary) && key=="hand" {
      let delta=(pos-*start)/self.zoom;
      let card_key=format!("{key}_card_{index}");
      let base=original.get(&card_key).copied().unwrap_or_default();
      let style=self.hand_styles.entry(card_key).or_default();
      style.dx=(base.dx+delta.x).round();
      style.dy=(base.dy+delta.y).round();
     }
    }
    if response.drag_stopped_by(egui::PointerButton::Primary){
     self.card_move=None;
    }
    if let Some((key,index))=&self.selected_card {
    if let Some((_,_,r))=hand_items.iter().find(|(k,i,_)|k==key&&i==index){
     let r=Rect::from_min_size(board.min+r.min.to_vec2()*self.zoom,r.size()*self.zoom);
     painter.rect_stroke(r,2.,Stroke::new(2.0_f32,Color32::YELLOW));
     if self.center_guides {let c=r.center();let guides=painter.with_clip_rect(viewport.intersect(board));guides.line_segment([Pos2::new(c.x,board.top()),Pos2::new(c.x,board.bottom())],Stroke::new(1.0_f32,Color32::YELLOW));guides.line_segment([Pos2::new(board.left(),c.y),Pos2::new(board.right(),c.y)],Stroke::new(1.0_f32,Color32::YELLOW));}
     let handle=Rect::from_center_size(r.right_bottom(),Vec2::splat(12.));
     painter.rect_filled(handle,1.,Color32::YELLOW);
     let resize=ui.interact(handle,ui.id().with("card_resize_handle"),Sense::drag());
     if resize.drag_started(){self.hand_undo.push(self.hand_styles.clone());self.hand_redo.clear();self.card_resize=Some((r.height()/self.zoom,self.hand_styles.get(key).copied().unwrap_or_default()));}
     if resize.dragged(){if let Some((height,original))=self.card_resize {
      let new_height=(height+resize.drag_delta().y/self.zoom).clamp(10.,1000.);
      let mut style=original;designer_preview::resize_hand(&mut style,height,new_height,self.resize_anchor);self.hand_styles.insert(key.clone(),style);
     }}
     if resize.drag_stopped(){
       if let (Some((original_height,_)),Some((key,_)))=(self.card_resize,&self.selected_card){
        if let Some(height)=self.hand_styles.get(key).and_then(|s|s.height){
         if (height-original_height).abs()>0.5 {
          self.queue_size_sync(SizeSync::Hand(key.clone(),height),ctx.input(|i|i.time));
         }
        }
       }
       self.card_resize=None;
      }
    }
   }
   if self.center_guides {
    // Draw over the preview so panels cannot obscure the reference lines.
    let guides=painter.with_clip_rect(viewport.intersect(board));
    let board_color=Color32::from_rgb(100,180,205);
    guides.line_segment([Pos2::new(center.x,board.top()),Pos2::new(center.x,board.bottom())],Stroke::new(1.0_f32,board_color));
    guides.line_segment([Pos2::new(board.left(),center.y),Pos2::new(board.right(),center.y)],Stroke::new(1.0_f32,board_color));
    for name in &self.selected {
     if let Some(b)=self.elements.get(name) {
      let c=board.min+Vec2::new(b.x+b.w/2.,b.y+b.h/2.)*self.zoom;
      let color=Color32::from_rgb(255,205,65);
      guides.line_segment([Pos2::new(c.x,board.top()),Pos2::new(c.x,board.bottom())],Stroke::new(1.0_f32,color));
      guides.line_segment([Pos2::new(board.left(),c.y),Pos2::new(board.right(),c.y)],Stroke::new(1.0_f32,color));
      guides.circle_stroke(c,4.,Stroke::new(1.5_f32,color));
     }
    }
   }
   for (name,b) in self.elements.iter().filter(|(n,_)|self.preview.visible(n,self.show_cpu,self.separate)){
    let r=Rect::from_min_size(board.min+b.rect().min.to_vec2()*self.zoom,b.rect().size()*self.zoom);
    let chosen=self.selected.contains(name);
    painter.rect_stroke(r,2.,Stroke::new(if chosen{2.5_f32}else{1.0_f32},if chosen{Color32::YELLOW}else{Color32::from_rgb(70,145,190)}));
    if r.width()>55. && r.height()>20.{painter.text(r.min+Vec2::new(4.,3.),egui::Align2::LEFT_TOP,name,egui::FontId::proportional(11.),Color32::LIGHT_GRAY);}
   }
   let pointer=ctx.input(|i|i.pointer.interact_pos());
   if response.drag_started_by(egui::PointerButton::Primary)&&card_hit.is_none(){
    if let Some(pos)=pointer{
     let world=self.world(pos,board);
     let hit=self.elements.iter().filter(|(n,b)|self.preview.visible(n,self.show_cpu,self.separate)&&b.rect().contains(world)).min_by(|(_,a),(_,b)|(a.w*a.h).total_cmp(&(b.w*b.h))).map(|(n,_)|n.clone());
     if let Some(name)=hit{
      let multi=ctx.input(|i|i.modifiers.ctrl||i.modifiers.command);
      if !self.selected.contains(&name){if !multi{self.selected.clear();}self.selected.insert(name);}
      else if multi{self.selected.remove(&name);}
      self.drag_start=Some((pos,self.elements.clone()));self.marquee=None;
     }else{
      if !ctx.input(|i|i.modifiers.ctrl){self.selected.clear();}
      self.marquee=Some(pos);self.drag_start=None;
     }
    }
   }
   if response.dragged_by(egui::PointerButton::Primary){
    if let (Some((start,original)),Some(pos))=(&self.drag_start,pointer){
     let delta=(pos-*start)/self.zoom;
     let selection:Vec<String>=self.selected.iter().cloned().collect();
     for name in selection{
      if let Some(base)=original.get(&name){
       let x=self.snap(base.x+delta.x,true,&name).round();
       let y=self.snap(base.y+delta.y,false,&name).round();
       self.elements.insert(name,Box2{x,y,..*base});
      }
     }
     let original=original.clone();self.carry_from(&original);
    }
   }
   if response.drag_stopped_by(egui::PointerButton::Primary){
    if let Some((_,old))=self.drag_start.take(){
     if old.iter().any(|(n,b)|self.elements.get(n).is_some_and(|v|v.x!=b.x||v.y!=b.y)){
      self.undo.push(old);self.redo.clear();
     }
    }
    if let (Some(start),Some(end))=(self.marquee.take(),pointer){
     let selection=Rect::from_two_pos(self.world(start,board),self.world(end,board));
     for (name,b) in self.elements.iter().filter(|(n,_)|self.preview.visible(n,self.show_cpu,self.separate)){if selection.intersects(b.rect()){self.selected.insert(name.clone());}}
    }
   }
   if let (Some(start),Some(pos))=(self.marquee,pointer){
    painter.rect_stroke(Rect::from_two_pos(start,pos),0.,Stroke::new(1.0_f32,Color32::YELLOW));
   }
   if response.clicked_by(egui::PointerButton::Primary)&&card_hit.is_none(){
    if let Some(pos)=pointer{
     let world=self.world(pos,board);
     let hit=self.elements.iter().filter(|(n,b)|self.preview.visible(n,self.show_cpu,self.separate)&&b.rect().contains(world)).min_by(|(_,a),(_,b)|(a.w*a.h).total_cmp(&(b.w*b.h))).map(|(n,_)|n.clone());
     if let Some(name)=hit{
      let multi=ctx.input(|i|i.modifiers.ctrl||i.modifiers.command);
      if !multi{self.selected.clear();}
      if multi&&self.selected.contains(&name){self.selected.remove(&name);}else{self.selected.insert(name);}
     }else if !ctx.input(|i|i.modifiers.ctrl){self.selected.clear();}
    }
   }
  });
  if !self.desktop && self.sync_prompt.is_none() && self.sync_draft.is_some(){
   let active=self.selected_card.as_ref().map(|(key,_)|key.clone())
    .or_else(||self.selected.iter().next().cloned());
   if active!=self.sync_owner {
    self.sync_prompt=self.sync_draft.take();
    self.sync_owner=None;
   }
  }
 }
}
fn main()->eframe::Result<()>{
 // eframe restores saved window bounds before applying these first-run defaults.
 let options=eframe::NativeOptions{persist_window:true,viewport:egui::ViewportBuilder::default().with_title("PhaseUP UI Designer").with_inner_size([1500.,900.]),..Default::default()};
 eframe::run_native("PhaseUP UI Designer",options,Box::new(|_|Ok(Box::new(Designer::new()))))
}

#[cfg(test)]
mod aspect_tests {
 use super::*;
 #[test] fn mobile_profiles_keep_geometry_in_bounds_and_do_not_share_files(){
  for aspect in 0..4 {
   let size=canvas(false,aspect);let mut e=defaults();rescale(&mut e,BOARD,size);
   for b in e.values(){assert!(b.x>=0.&&b.y>=0.&&b.x+b.w<=size.x+0.01&&b.y+b.h<=size.y+0.01);}
   assert_ne!(profile_path(false,false,aspect),profile_path(true,false,aspect));
   assert_eq!(profile_path(false,false,aspect),profile_path(false,true,aspect));
   assert_ne!(profile_path(true,false,aspect),profile_path(true,true,aspect));
  }
  assert_ne!(profile_path(false,false,0),profile_path(false,false,1));
  assert_ne!(profile_path(true,false,2),profile_path(true,false,3));
  assert_eq!(device_aspect(false,3),0);assert_eq!(device_aspect(true,0),3);
  assert!((canvas(true,2).x/canvas(true,2).y-16./9.).abs()<0.001);
 }
}
