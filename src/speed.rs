use std::cell::Cell;
use crate::*;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) enum GameSpeed { Slow, Normal, Fast }
impl GameSpeed {
    pub(super) fn next(self)->Self {match self {Self::Slow=>Self::Normal,Self::Normal=>Self::Fast,Self::Fast=>Self::Slow}}
    pub(super) fn name(self)->&'static str {match self {Self::Slow=>"Slow",Self::Normal=>"Normal",Self::Fast=>"Fast"}}
    pub(super) fn key(self)->&'static str {match self {Self::Slow=>"slow",Self::Normal=>"normal",Self::Fast=>"fast"}}
    pub(super) fn from_key(key:&str)->Self {match key.trim() {"slow"=>Self::Slow,"fast"=>Self::Fast,_=>Self::Normal}}
    pub(super) fn code(self)->u8 {match self {Self::Fast=>1,Self::Normal=>2,Self::Slow=>3}}
    pub(super) fn from_code(code:u8)->Self {match code {1=>Self::Fast,3=>Self::Slow,_=>Self::Normal}}
    fn factor(self)->f64 {match self {Self::Slow=>1.75,Self::Normal=>1.0,Self::Fast=>0.5}}
    pub(super) fn duration(self,normal:f64)->f64 {normal*self.factor()}
}
thread_local! {static SPEED:Cell<GameSpeed>=const {Cell::new(GameSpeed::Normal)};}
pub(super) fn active_game_speed()->GameSpeed {SPEED.with(|s|s.get())}
pub(super) fn set_game_speed(speed:GameSpeed) {SPEED.with(|s|s.set(speed));}
pub(super) fn read_game_speed()->GameSpeed {
    std::fs::read_to_string(window_size_path().with_file_name("game_speed.txt"))
        .map(|s|GameSpeed::from_key(&s)).unwrap_or(GameSpeed::Normal)
}
pub(super) fn save_game_speed(speed:GameSpeed) {
    let path=window_size_path().with_file_name("game_speed.txt");
    if let Some(parent)=path.parent(){if std::fs::create_dir_all(parent).is_err(){return;}}
    let _=std::fs::write(path,speed.key());
}
// Preserve progress when an active animation is retimed from Settings.
fn retime(started:&mut f64,duration:&mut f64,now:f64,ratio:f64){
    *started=now-(now-*started)*ratio;
    *duration*=ratio;
}
impl Game {
    pub(super) fn action_duration(&self,normal:f64)->f64 {GameSpeed::from_code(self.cpu_speed).duration(normal)}
    pub(super) fn change_speed(&mut self,speed:GameSpeed,now:f64){
        if self.cpu_speed==speed.code(){return;}
        let ratio=speed.factor()/GameSpeed::from_code(self.cpu_speed).factor();
        self.cpu_next_at=now+(self.cpu_next_at-now).max(0.0)*ratio;
        if let Some(a)=&mut self.cpu_draw_flight {retime(&mut a.started,&mut a.duration,now,ratio);}
        if let Some(a)=&mut self.cpu_discard_flight {retime(&mut a.started,&mut a.duration,now,ratio);}
        if let Some(a)=&mut self.cpu_hit_flight {retime(&mut a.started,&mut a.duration,now,ratio);}
        if let Some(a)=&mut self.cpu_phase_flight {retime(&mut a.started,&mut a.per_card,now,ratio);}
        if let Some(a)=&mut self.human_phase_flight {retime(&mut a.started,&mut a.per_card,now,ratio);}
        if let Some(a)=&mut self.human_draw_flight {retime(&mut a.started,&mut a.duration,now,ratio);}
        if let Some(a)=&mut self.human_discard_flight {retime(&mut a.started,&mut a.duration,now,ratio);}
        self.cpu_speed=speed.code();
    }
}
