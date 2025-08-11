#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused_imports)]
#[allow(dead_code)]
mod bindings{
    include!(concat!(env!("OUT_DIR"),"/bindings.rs"));
}
pub use bindings::*;
pub type InputEvent=input_event;
impl InputEvent{
    pub const SIZE:usize=std::mem::size_of::<Self>();
    pub fn new()->Self{
        Self{type_:0,code:0,value:0,time:timeval{tv_sec:0,tv_usec:0}}
    }
    pub fn parse(input:[u8;Self::SIZE])->Self{
        unsafe{*(
            input.as_ptr() as *const Self
        )}
    }
    pub fn unparse(event:Self)->&'static [u8]{
        let u8s=&event as *const Self as *const u8;
        unsafe{
            std::slice::from_raw_parts(u8s,Self::SIZE)
        }
    }
    pub fn r#type(&self)->u16{
        self.type_
    }
    pub fn code(&self)->u16{
        self.code
    }
    pub fn value(&self)->i32{
        self.value
    }
    pub fn time_reset(&mut self){
        let time=std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
        let (sec,usec)=(time.as_secs(),time.subsec_micros());
        self.time.tv_sec=sec.try_into().unwrap();
        self.time.tv_usec=usec.try_into().unwrap();
    }
    pub fn is_key_release(&self)->bool{
        self.r#type() as u32==EV_KEY&&self.value()==0
    }
}
