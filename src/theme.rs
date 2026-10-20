use macroquad::prelude::*;
use std::cell::Cell;

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub(super) enum Theme { Modern, Futuristic, Playful }
impl Theme {
    pub(super) fn next(self)->Self {match self {Self::Modern=>Self::Futuristic,Self::Futuristic=>Self::Playful,Self::Playful=>Self::Modern}}
    pub(super) fn name(self)->&'static str {match self {Self::Modern=>"MODERN & CLEAN",Self::Futuristic=>"BOLD & FUTURISTIC",Self::Playful=>"COLORFUL & PLAYFUL"}}
    pub(super) fn key(self)->&'static str {match self {Self::Modern=>"modern",Self::Futuristic=>"futuristic",Self::Playful=>"playful"}}
    pub(super) fn from_key(s:&str)->Self {match s {"futuristic"=>Self::Futuristic,"playful"=>Self::Playful,_=>Self::Modern}}
    pub(super) fn accent(self)->Color {match self {Self::Modern=>Color::new(0.25,0.81,0.72,1.),Self::Futuristic=>Color::new(0.0,0.88,1.0,1.),Self::Playful=>Color::new(1.0,0.75,0.30,1.)}}
    pub(super) fn panel(self)->Color {match self {Self::Modern=>Color::new(0.10,0.16,0.24,1.),Self::Futuristic=>Color::new(0.035,0.11,0.20,1.),Self::Playful=>Color::new(0.12,0.24,0.22,1.)}}
    pub(super) fn background(self)->Color {match self {Self::Modern=>Color::new(0.055,0.09,0.15,1.),Self::Futuristic=>Color::new(0.015,0.045,0.10,1.),Self::Playful=>Color::new(0.035,0.13,0.12,1.)}}
}
thread_local! {static ACTIVE:Cell<Theme>=const {Cell::new(Theme::Modern)};}
pub(super) fn active_theme()->Theme {ACTIVE.with(|v|v.get())}
pub(super) fn set_theme(theme:Theme) {ACTIVE.with(|v|v.set(theme));}
pub(super) fn themed_color(color:Color)->Color {
    let theme=active_theme();
    if theme==Theme::Modern {return color;}
    if color==crate::ACCENT {return theme.accent();}
    if color==crate::PANEL {return theme.panel();}
    if color==crate::BG {return theme.background();}
    color
}
