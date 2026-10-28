pub(super) fn window_size_path() -> std::path::PathBuf {
    let base=std::env::var_os("APPDATA").map(std::path::PathBuf::from)
        .or_else(||std::env::var_os("HOME").map(|h|std::path::PathBuf::from(h).join(".config")))
        .unwrap_or_else(||std::path::PathBuf::from("."));
    base.join("PhaseUP").join("window_size.txt")
}
pub(super) fn read_window_size() -> (i32,i32) {
    if let Ok(s)=std::fs::read_to_string(window_size_path()) {
        let mut parts=s.split_whitespace();
        if let (Some(w),Some(h))=(parts.next().and_then(|v|v.parse::<i32>().ok()),parts.next().and_then(|v|v.parse::<i32>().ok())) {
            if (640..=7680).contains(&w)&&(480..=4320).contains(&h) {return fit_window_to_desktop(w,h);}
        }
    }
    fit_window_to_desktop(1600,1000)
}
#[cfg(target_os="windows")]
pub(super) fn fit_window_to_desktop(w:i32,h:i32)->(i32,i32){
    use std::ffi::c_void;
    #[repr(C)]
    struct Rect {left:i32,top:i32,right:i32,bottom:i32}
    #[link(name="user32")]
    extern "system" {
        fn SystemParametersInfoW(action:u32,param:u32,out:*mut c_void,flags:u32)->i32;
    }
    let mut area=Rect{left:0,top:0,right:0,bottom:0};
    if unsafe{SystemParametersInfoW(0x0030,0,&mut area as *mut _ as *mut c_void,0)}==0{return (w,h);}
    // Reserve a little room for the title bar, borders and desktop margins.
    let max_w=(area.right-area.left-32).max(640);
    let max_h=(area.bottom-area.top-80).max(480);
    let scale=(max_w as f32/w as f32).min(max_h as f32/h as f32).min(1.0);
    ((w as f32*scale).round() as i32,(h as f32*scale).round() as i32)
}
#[cfg(not(target_os="windows"))]
pub(super) fn fit_window_to_desktop(w:i32,h:i32)->(i32,i32){(w,h)}
pub(super) fn save_window_size(w:i32,h:i32) {
    let path=window_size_path();
    if let Some(parent)=path.parent() {
        if std::fs::create_dir_all(parent).is_err(){return;}
    }
    let _=std::fs::write(path,format!("{w} {h}\n"));
}
// Keep the requested preset separate from the fitted client-area dimensions.
pub(super) fn selected_resolution_path()->std::path::PathBuf {
    window_size_path().with_file_name("selected_resolution.txt")
}
pub(super) fn save_selected_resolution(w:i32,h:i32) {
    let path=selected_resolution_path();
    if let Some(parent)=path.parent(){let _=std::fs::create_dir_all(parent);}
    let _=std::fs::write(path,format!("{w} {h}\n"));
}
pub(super) fn read_selected_resolution()->Option<(i32,i32)> {
    let data=std::fs::read_to_string(selected_resolution_path()).ok()?;
    let mut parts=data.split_whitespace();
    let w=parts.next()?.parse::<i32>().ok()?;
    let h=parts.next()?.parse::<i32>().ok()?;
    if (640..=7680).contains(&w)&&(480..=4320).contains(&h){Some((w,h))}else{None}
}
// Persist interface preferences alongside the existing window-size file.
pub(super) fn preferences_path()->std::path::PathBuf {
    window_size_path().with_file_name("settings.txt")
}
pub(super) fn read_preferences()->(bool,bool){
    let mut show_moves=true;let mut fullscreen=false;
    if let Ok(data)=std::fs::read_to_string(preferences_path()){
        for line in data.lines(){
            if let Some((key,value))=line.split_once('='){
                match key.trim(){
                    "recent_moves"=>match value.trim(){"true"=>show_moves=true,"false"=>show_moves=false,_=>{}},
                    "fullscreen"=>match value.trim(){"true"=>fullscreen=true,"false"=>fullscreen=false,_=>{}},
                    _=>{}
                }
            }
        }
    }
    (show_moves,fullscreen)
}
pub(super) fn save_preferences(show_moves:bool,fullscreen:bool){
    let path=preferences_path();
    if let Some(parent)=path.parent(){if std::fs::create_dir_all(parent).is_err(){return;}}
    let theme=read_theme();
    let _=std::fs::write(path,format!("recent_moves={}\nfullscreen={}\ntheme={}\n",show_moves,fullscreen,theme.key()));
}

pub(super) fn read_theme()->crate::theme::Theme {
    if let Ok(data)=std::fs::read_to_string(preferences_path()){
        for line in data.lines(){
            if let Some((key,value))=line.split_once('='){
                if key.trim()=="theme" {return crate::theme::Theme::from_key(value.trim());}
            }
        }
    }
    crate::theme::Theme::Futuristic
}
pub(super) fn save_theme(theme:crate::theme::Theme){
    let path=preferences_path();
    let data=std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines:Vec<String>=data.lines().filter(|line|!line.starts_with("theme=")).map(str::to_owned).collect();
    lines.push(format!("theme={}",theme.key()));
    if let Some(parent)=path.parent(){if std::fs::create_dir_all(parent).is_err(){return;}}
    let _=std::fs::write(path,format!("{}\n",lines.join("\n")));
}

#[cfg(target_os="windows")]
pub(super) mod window_placement {
    use std::ffi::c_void;
    type Hwnd=*mut c_void;
    #[repr(C)]
    struct Rect {left:i32,top:i32,right:i32,bottom:i32}
    #[link(name="user32")]
    extern "system" {
        fn FindWindowW(class_name:*const u16,window_name:*const u16)->Hwnd;
        fn GetWindowRect(hwnd:Hwnd,rect:*mut Rect)->i32;
        fn SetWindowPos(hwnd:Hwnd,after:Hwnd,x:i32,y:i32,cx:i32,cy:i32,flags:u32)->i32;
        fn GetSystemMetrics(index:i32)->i32;
        fn SystemParametersInfoW(action:u32,param:u32,out:*mut c_void,flags:u32)->i32;
    }
    fn hwnd()->Hwnd {
        let title:Vec<u16>="PhaseUP - Offline Card Game".encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {FindWindowW(std::ptr::null(),title.as_ptr())}
    }
    fn path()->std::path::PathBuf {
        super::window_size_path().with_file_name("window_position.txt")
    }
    pub fn current()->Option<(i32,i32)> {
        let h=hwnd();if h.is_null(){return None;}
        let mut r=Rect{left:0,top:0,right:0,bottom:0};
        if unsafe{GetWindowRect(h,&mut r)}==0{return None;}
        Some((r.left,r.top))
    }
    pub fn restore() {
        let Ok(data)=std::fs::read_to_string(path()) else{return;};
        let mut p=data.split_whitespace();
        let (Some(x),Some(y))=(p.next().and_then(|v|v.parse::<i32>().ok()),p.next().and_then(|v|v.parse::<i32>().ok())) else{return;};
        // Avoid restoring a completely off-screen position after monitor changes.
        let (left,top,width,height)=unsafe{(GetSystemMetrics(76),GetSystemMetrics(77),GetSystemMetrics(78),GetSystemMetrics(79))};
        if width<=0||height<=0||x<left-100||y<top-100||x>left+width-100||y>top+height-100{return;}
        let h=hwnd();if h.is_null(){return;}
        unsafe {SetWindowPos(h,std::ptr::null_mut(),x,y,0,0,0x0001|0x0004|0x0010);}
    }
    // Center the complete decorated window inside the desktop work area,
    // leaving the title bar visible even after a resolution change.
    pub fn center() -> Option<(i32,i32)> {
        let h=hwnd();
        if h.is_null(){return None;}
        let mut rect=Rect{left:0,top:0,right:0,bottom:0};
        let mut area=Rect{left:0,top:0,right:0,bottom:0};
        if unsafe{GetWindowRect(h,&mut rect)}==0{return None;}
        let ok=unsafe{SystemParametersInfoW(0x0030,0,&mut area as *mut _ as *mut c_void,0)};
        if ok==0{return None;}
        let width=rect.right-rect.left;
        let height=rect.bottom-rect.top;
        let x=area.left+((area.right-area.left-width)/2).max(0);
        let y=area.top+((area.bottom-area.top-height)/2).max(0);
        unsafe{SetWindowPos(h,std::ptr::null_mut(),x,y,0,0,0x0001|0x0004|0x0010);}
        Some((x,y))
    }
    pub fn save(x:i32,y:i32) {
        let p=path();
        if let Some(parent)=p.parent(){if std::fs::create_dir_all(parent).is_err(){return;}}
        let _=std::fs::write(p,format!("{x} {y}\n"));
    }
}

 
// CPU difficulty is stored independently of display settings.
pub(super) fn cpu_difficulty_path()->std::path::PathBuf {
    window_size_path().with_file_name("cpu_difficulty.txt")
}
pub(super) fn read_cpu_difficulty()->crate::Difficulty {
    std::fs::read_to_string(cpu_difficulty_path())
        .map(|s|crate::Difficulty::from_key(s.trim()))
        .unwrap_or(crate::Difficulty::Normal)
}
pub(super) fn save_cpu_difficulty(level:crate::Difficulty) {
    let path=cpu_difficulty_path();
    if let Some(parent)=path.parent(){if std::fs::create_dir_all(parent).is_err(){return;}}
    let _=std::fs::write(path,level.key());
}

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub(super) enum DisplayLayout { Desktop, Mobile }
impl DisplayLayout {
    pub(super) fn name(self)->&'static str {match self {Self::Desktop=>"DESKTOP",Self::Mobile=>"MOBILE"}}
    pub(super) fn next(self)->Self {match self {Self::Desktop=>Self::Mobile,Self::Mobile=>Self::Desktop}}
    pub(super) fn presets(self)->&'static [(i32,i32)] {
        match self {
            Self::Desktop=>&[(1280,800),(1440,900),(1600,1000),(1920,1200),(2560,1600),(1280,720),(1600,900),(1920,1080),(2560,1440)],
            Self::Mobile=>&[(2400,1080),(2340,1080)]
        }
    }
    pub(super) fn aspect(self,w:i32,h:i32)->&'static str {
        if self==Self::Mobile {if (w as f32/h as f32-20.0/9.0).abs()<0.03 {"20_9"}else{"19_5_9"}}
        else if (w as f32/h as f32-16.0/9.0).abs()<0.03 {"16_9"}else{"16_10"}
    }
    pub(super) fn layout_file(self,w:i32,h:i32)->String {
        let kind=if self==Self::Mobile {"mobile"}else{"desktop"};
        format!("assets/layouts/prototype_{}_compact_{}.json",kind,self.aspect(w,h))
    }
}
pub(super) fn display_layout_path()->std::path::PathBuf {window_size_path().with_file_name("display_layout.txt")}
pub(super) fn read_display_layout()->DisplayLayout {
    if std::fs::read_to_string(display_layout_path()).is_ok_and(|s|s.trim()=="mobile"){DisplayLayout::Mobile}else{DisplayLayout::Desktop}
}
pub(super) fn save_display_layout(mode:DisplayLayout){
    let path=display_layout_path();
    if let Some(parent)=path.parent(){let _=std::fs::create_dir_all(parent);}
    let _=std::fs::write(path,if mode==DisplayLayout::Mobile {"mobile"}else{"desktop"});
}
pub(super) fn selected_layout_file()->String {
    let mode=read_display_layout();
    let (w,h)=read_selected_resolution().unwrap_or(if mode==DisplayLayout::Mobile {(2400,1080)}else{(1600,1000)});
    mode.layout_file(w,h)
}
