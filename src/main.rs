#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"),"/bindings.rs"));
struct Logger;
type InputEvent=input_event;
impl log::Log for Logger{
    fn enabled(&self,_metadata:&log::Metadata)->bool{
        true
    }
    fn log(&self,record:&log::Record){
        let label:&str=match record.level(){
            log::Level::Error=>"\x1b[31;1m[ERROR]\x1b[0m",
            log::Level::Warn=>"\x1b[33;1m[WARNING]\x1b[0m",
            log::Level::Info=>"\x1b[36;1m[INFO]\x1b[0m",
            log::Level::Debug=>"\x1b[35;1m[DEBUG]\x1b[0m",
            log::Level::Trace=>"\x1b[32;1m[TRACE]\x1b[0m",
        };
        eprintln!("{} {}",label,record.args());
    }
    fn flush(&self){}
}
static LOGGER:Logger=Logger;
const SIZEOF_EVENT:usize=std::mem::size_of::<InputEvent>();
const DEBOUNCE_TIME:std::time::Duration=std::time::Duration::from_millis(12);
#[tokio::main]
async fn main(){
    if let Err(e)=log::set_logger(&LOGGER).map(|_|log::set_max_level(log::LevelFilter::Trace)){
        eprintln!("Failed to initialize logger: {}",e);
    };
    #[allow(unused_mut,unused_variables)]
    let (tx,mut rx):(tokio::sync::broadcast::Sender<u16>,tokio::sync::broadcast::Receiver<u16>)=tokio::sync::broadcast::channel(1<<8);
    let mut events:Vec<InputEvent>=Vec::<InputEvent>::new();
    loop{
        use std::io::{Read,Write};
        let mut input:[u8;SIZEOF_EVENT]=[0;SIZEOF_EVENT];
        if let Err(e)=std::io::stdin().read_exact(&mut input){
            log::error!("{}",e);
            std::process::exit(1);
        }else{
            let new_event=unsafe{
                *(input.as_ptr() as *const InputEvent)
            };
            events.push(new_event);
            if new_event.type_!=0||new_event.code!=0||new_event.value!=0{
                continue;
            }
        }
        let release_event:Vec<&InputEvent>=events.as_slice().into_iter().filter(|e|
            e.type_ as u32==EV_KEY&&e.value==0
        ).collect();
        if release_event.len()>0{
            let mut cancelled=false;
            let code=release_event[0].code;
            let mut receiver=tx.subscribe();
            let events=events.clone();
            tokio::spawn(async move{
                let start_time=std::time::SystemTime::now();
                for e in &events{
                    log::trace!("\x1b[33mdelaying event\x1b[0m: {:?}",e);
                }
                while std::time::SystemTime::now()<=start_time+DEBOUNCE_TIME{
                    if let Ok(c)=receiver.try_recv(){
                        if c==code{
                            cancelled=true;
                            break;
                        }
                    }
                }
                if !cancelled{
                    for e in &events{
                        log::trace!("\x1b[32mdelayed event\x1b[0m: {:?}",e);
                        let ptr=&e.clone() as *const InputEvent as *const u8;
                        let bytes=unsafe{std::slice::from_raw_parts(ptr,SIZEOF_EVENT)};
                        let _=std::io::stdout().write(bytes);
                        let _=std::io::stdout().flush();
                    }
                }
            });
        }else{
            for e in &events{
                log::trace!("\x1b[36mnormal event\x1b[0m: {:?}",e);
                if e.type_ as u32==EV_KEY{
                    tx.send(e.code).unwrap();
                }
                let ptr=&e.clone() as *const InputEvent as *const u8;
                let bytes=unsafe{std::slice::from_raw_parts(ptr,SIZEOF_EVENT)};
                let _=std::io::stdout().write(bytes);
                let _=std::io::stdout().flush();
            }
        }
        events.clear();
    }
}
