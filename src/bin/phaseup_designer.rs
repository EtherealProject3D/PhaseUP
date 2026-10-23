use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use std::{collections::{BTreeMap,BTreeSet},fs,path::PathBuf};

const BOARD:Vec2=Vec2::new(1920.0,1200.0);
const PATH:&str="assets/layouts/game_16_10.json";
#[derive(Clone,Copy,Debug)]
struct Box2{x:f32,y:f32,w:f32,h:f32}
impl Box2 {
 fn from_json(v:&serde_json::Value)->Option<Self>{Some(Self{x:v["x"].as_f64()? as f32,y:v["y"].as_f64()? as f32,w:v["width"].as_f64()? as f32,h:v["height"].as_f64()? as f32})}
 fn json(self)->serde_json::Value{serde_json::json!({"x":self.x,"y":self.y,"width":self.w,"height":self.h})}
 fn rect(self)->Rect{Rect::from_min_size(Pos2::new(self.x,self.y),Vec2::new(self.w,self.h))}
}
fn defaults()->BTreeMap<String,Box2>{
 let mut m=BTreeMap::new();
 for (n,x,y,w,h) in [
 ("game_logo",24.,10.,230.,42.),("turn_banner",16.,55.,1888.,62.),
 ("cpu_1_panel",16.,72.,626.,376.),("cpu_2_panel",648.,72.,626.,376.),
 ("cpu_3_panel",1280.,72.,626.,376.),("player_phase",560.,656.,800.,140.),
 ("stage_1",274.,853.,680.,117.),("stage_2",960.,853.,680.,117.),
 ("hand_panel",12.,971.,1896.,217.),
 ("cpu_1_group_1",32.,263.,305.,116.),("cpu_1_group_2",344.,263.,305.,116.),
 ("cpu_2_group_1",664.,263.,305.,116.),("cpu_2_group_2",976.,263.,305.,116.),
 ("cpu_3_group_1",1296.,263.,305.,116.),("cpu_3_group_2",1608.,263.,305.,116.)
 ]{m.insert(n.into(),Box2{x,y,w,h});}m
}
type Snapshot=BTreeMap<String,Box2>;
struct Designer {
 elements:Snapshot, selected:BTreeSet<String>,undo:Vec<Snapshot>,redo:Vec<Snapshot>,
 zoom:f32,pan:Vec2,grid:f32,snap_grid:bool,snap_edges:bool,snap_objects:bool,
 drag_start:Option<(Pos2,Snapshot)>,marquee:Option<Pos2>,status:String,
 path:PathBuf
}
impl Designer {
 fn new()->Self{
  let mut app=Self{elements:defaults(),selected:BTreeSet::new(),undo:vec![],redo:vec![],
   zoom:0.38,pan:Vec2::ZERO,grid:8.,snap_grid:true,snap_edges:true,snap_objects:true,
   drag_start:None,marquee:None,status:String::new(),path:PATH.into()};
  app.load();app
 }
 fn load(&mut self){
  match fs::read_to_string(&self.path){
   Ok(s)=>{
    let trimmed=s.trim_end();
    let parsed=serde_json::from_str::<serde_json::Value>(&s).or_else(|_|serde_json::from_str(trimmed.strip_suffix("\\n").unwrap_or(trimmed)));
    match parsed {
     Ok(root)=>{if let Some(map)=root["elements"].as_object(){
      for (name,v) in map {if let Some(b)=Box2::from_json(v){self.elements.insert(name.clone(),b);}}
     }self.status=format!("Loaded {}",self.path.display());}
     Err(e)=>self.status=format!("Load failed: {e}")
    }
   }
   Err(_)=>self.status="Using default layout".into()
  }
 }
 fn save(&mut self){
  let mut root=fs::read_to_string(&self.path).ok()
   .and_then(|s|serde_json::from_str::<serde_json::Value>(&s).ok())
   .unwrap_or_else(||serde_json::json!({"schema_version":1,"canvas":{"width":1920,"height":1200},"elements":{}}));
  if !root["elements"].is_object(){root["elements"]=serde_json::json!({});}
  for (n,b) in &self.elements{root["elements"][n]=b.json();}
  let result=(||->Result<(),String>{
   if let Some(parent)=self.path.parent(){fs::create_dir_all(parent).map_err(|e|e.to_string())?;}
   fs::write(&self.path,format!("{}\n",serde_json::to_string_pretty(&root).map_err(|e|e.to_string())?)).map_err(|e|e.to_string())
  })();
  self.status=match result{Ok(())=>format!("Saved {}",self.path.display()),Err(e)=>format!("Save failed: {e}")};
 }
 fn checkpoint(&mut self){self.undo.push(self.elements.clone());if self.undo.len()>100{self.undo.remove(0);}self.redo.clear();}
 fn undo(&mut self){if let Some(s)=self.undo.pop(){self.redo.push(self.elements.clone());self.elements=s;}}
 fn redo(&mut self){if let Some(s)=self.redo.pop(){self.undo.push(self.elements.clone());self.elements=s;}}
 fn snap(&self,value:f32,axis:bool,ignore:&str)->f32{
  let mut result=value;let mut distance=7.0/self.zoom;
  if self.snap_grid {let candidate=(value/self.grid).round()*self.grid;let d=(candidate-value).abs();if d<distance{result=candidate;distance=d;}}
  if self.snap_edges{
   for target in [0.0,if axis{BOARD.x/2.0}else{BOARD.y/2.0},if axis{BOARD.x}else{BOARD.y}]{
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
  self.zoom=((viewport.width()-48.)/BOARD.x).min((viewport.height()-48.)/BOARD.y).clamp(0.1,2.0);
  self.pan=Vec2::ZERO;
 }
 fn board_rect(&self,viewport:Rect)->Rect{
  Rect::from_center_size(viewport.center()+self.pan,BOARD*self.zoom)
 }
 fn world(&self,screen:Pos2,board:Rect)->Pos2{
  Pos2::new((screen.x-board.min.x)/self.zoom,(screen.y-board.min.y)/self.zoom)
 }
}
impl eframe::App for Designer{
 fn update(&mut self,ctx:&egui::Context,_frame:&mut eframe::Frame){
  let undo=ctx.input(|i|i.modifiers.command&&i.key_pressed(egui::Key::Z));
  let redo=ctx.input(|i|i.modifiers.command&&i.key_pressed(egui::Key::Y));
  if undo{self.undo();}if redo{self.redo();}
  egui::TopBottomPanel::top("toolbar").show(ctx,|ui|{
   ui.horizontal(|ui|{
    ui.heading("PhaseUP UI Designer");
    ui.separator();
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
    let names:Vec<String>=self.elements.keys().cloned().collect();
    for name in names{
     let selected=self.selected.contains(&name);
     if ui.selectable_label(selected,&name).clicked(){
      let multi=ui.input(|i|i.modifiers.ctrl||i.modifiers.command);
      if !multi{self.selected.clear();}
      if multi&&selected{self.selected.remove(&name);}else{self.selected.insert(name);}
     }
    }
   });
  });
  egui::SidePanel::right("properties").default_width(255.).min_width(210.).show(ctx,|ui|{
   ui.heading("Properties");
   ui.label(format!("{} selected",self.selected.len()));
   if self.selected.len()==1{
    let name=self.selected.iter().next().unwrap().clone();
    ui.label(&name);
    let mut b=self.elements[&name];let old=b;
    egui::Grid::new("properties_grid").show(ui,|ui|{
     ui.label("X");ui.add(egui::DragValue::new(&mut b.x).speed(1.));ui.end_row();
     ui.label("Y");ui.add(egui::DragValue::new(&mut b.y).speed(1.));ui.end_row();
     ui.label("Width");ui.add(egui::DragValue::new(&mut b.w).speed(1.).range(1.0..=1920.));ui.end_row();
     ui.label("Height");ui.add(egui::DragValue::new(&mut b.h).speed(1.).range(1.0..=1200.));ui.end_row();
    });
    if b.x!=old.x||b.y!=old.y||b.w!=old.w||b.h!=old.h{self.checkpoint();self.elements.insert(name,b);}
   }
   if self.selected.len()>1{
    if ui.button("Align left").clicked(){self.checkpoint();let x=self.selected.iter().map(|n|self.elements[n].x).fold(f32::INFINITY,f32::min);for n in &self.selected{self.elements.get_mut(n).unwrap().x=x;}}
    if ui.button("Align top").clicked(){self.checkpoint();let y=self.selected.iter().map(|n|self.elements[n].y).fold(f32::INFINITY,f32::min);for n in &self.selected{self.elements.get_mut(n).unwrap().y=y;}}
    if ui.button("Center horizontally").clicked(){self.checkpoint();for n in &self.selected{let b=self.elements.get_mut(n).unwrap();b.x=(BOARD.x-b.w)/2.;}}
    if ui.button("Center vertically").clicked(){self.checkpoint();for n in &self.selected{let b=self.elements.get_mut(n).unwrap();b.y=(BOARD.y-b.h)/2.;}}
   }
   ui.separator();ui.heading("Snapping");
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
  egui::CentralPanel::default().show(ctx,|ui|{
   let viewport=ui.available_rect_before_wrap();
   let response=ui.allocate_rect(viewport,Sense::click_and_drag());
   if self.zoom==0.0||ctx.input(|i|i.key_pressed(egui::Key::F)){self.fit(viewport);}
   let hover=response.hover_pos();
   if response.hovered(){
    let wheel=ctx.input(|i|i.raw_scroll_delta.y);
    if wheel!=0.0 {
     let old=self.zoom;let new=(old*(wheel*0.0015).exp()).clamp(0.1,4.0);
     if let Some(pointer)=hover{
      let offset=pointer-(viewport.center()+self.pan);
      self.pan+=offset*(1.0-old/new);
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
   painter.rect_stroke(board,0.,Stroke::new(2.,Color32::from_rgb(80,170,225)));
   let center=board.center();
   painter.line_segment([Pos2::new(center.x,board.top()),Pos2::new(center.x,board.bottom())],Stroke::new(1.,Color32::from_rgb(36,90,95)));
   painter.line_segment([Pos2::new(board.left(),center.y),Pos2::new(board.right(),center.y)],Stroke::new(1.,Color32::from_rgb(36,90,95)));
   for (name,b) in &self.elements{
    let r=Rect::from_min_size(board.min+b.rect().min.to_vec2()*self.zoom,b.rect().size()*self.zoom);
    let chosen=self.selected.contains(name);
    painter.rect_stroke(r,2.,Stroke::new(if chosen{2.5}else{1.},if chosen{Color32::YELLOW}else{Color32::from_rgb(70,145,190)}));
    if r.width()>55. && r.height()>20.{painter.text(r.min+Vec2::new(4.,3.),egui::Align2::LEFT_TOP,name,egui::FontId::proportional(11.),Color32::LIGHT_GRAY);}
   }
   let pointer=ctx.input(|i|i.pointer.interact_pos());
   if response.drag_started_by(egui::PointerButton::Primary){
    if let Some(pos)=pointer{
     let world=self.world(pos,board);
     let hit=self.elements.iter().filter(|(_,b)|b.rect().contains(world)).map(|(n,_)|n.clone()).last();
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
     for (name,b) in &self.elements{if selection.intersects(b.rect()){self.selected.insert(name.clone());}}
    }
   }
   if let (Some(start),Some(pos))=(self.marquee,pointer){
    painter.rect_stroke(Rect::from_two_pos(start,pos),0.,Stroke::new(1.,Color32::YELLOW));
   }
   if response.clicked_by(egui::PointerButton::Primary){
    if let Some(pos)=pointer{
     let world=self.world(pos,board);
     let hit=self.elements.iter().filter(|(_,b)|b.rect().contains(world)).map(|(n,_)|n.clone()).last();
     if let Some(name)=hit{
      let multi=ctx.input(|i|i.modifiers.ctrl||i.modifiers.command);
      if !multi{self.selected.clear();}
      if multi&&self.selected.contains(&name){self.selected.remove(&name);}else{self.selected.insert(name);}
     }else if !ctx.input(|i|i.modifiers.ctrl){self.selected.clear();}
    }
   }
  });
 }
}
fn main()->eframe::Result<()>{
 let options=eframe::NativeOptions{viewport:egui::ViewportBuilder::default().with_title("PhaseUP UI Designer").with_inner_size([1500.,900.]),..Default::default()};
 eframe::run_native("PhaseUP UI Designer",options,Box::new(|_|Ok(Box::new(Designer::new()))))
}
