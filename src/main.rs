mod input;
mod utils;
const DEBOUNCE_TIME:std::time::Duration=std::time::Duration::from_millis(12);
#[tokio::main]
async fn main(){
    utils::LOGGER.init();
    #[allow(unused_variables)]
    let (tx,rx):(tokio::sync::broadcast::Sender<u16>,tokio::sync::broadcast::Receiver<u16>)=tokio::sync::broadcast::channel(1<<8);
    let mut events:Vec<input::InputEvent>=Vec::<input::InputEvent>::new();
    loop{
        use std::io::{Read,Write};
        let mut input:[u8;input::SIZEOF_EVENT]=[0;input::SIZEOF_EVENT];
        if let Err(e)=std::io::stdin().read_exact(&mut input){
            log::error!("{}",e);
            std::process::exit(1);
        }else{
            let new_event=unsafe{
                *(input.as_ptr() as *const input::InputEvent)
            };
            events.push(new_event);
            log::trace!("new event comes: {:?}",&new_event);
            if new_event.type_!=0||new_event.code!=0{
                continue;
            }
        }
        let release_event:Vec<&input::InputEvent>=events.as_slice().into_iter().filter(|e|
            e.type_ as u32==input::EV_KEY&&e.value==0
        ).collect();
        if release_event.len()>0{
            let mut canceled=false;
            let code=release_event[0].code;
            let mut receiver=tx.subscribe();
            let mut events=events.clone();
            tokio::spawn(async move{
                for e in &events{
                    log::debug!("\x1b[33mdelaying event\x1b[0m: {:?}",e);
                }
                let start_time=std::time::SystemTime::now();
                while std::time::SystemTime::now().duration_since(start_time).unwrap()<DEBOUNCE_TIME{
                    if let Ok(c)=receiver.try_recv(){
                        if c==code{
                            canceled=true;
                            break;
                        }
                    }
                }
                let time=std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
                let (sec,usec)=(time.as_secs(),time.subsec_micros());
                for e in &mut events{
                    e.time.tv_sec=sec.try_into().unwrap();
                    e.time.tv_usec=usec.try_into().unwrap();
                }
                if !canceled{
                    for e in &events{
                        log::debug!("\x1b[32mdelayed event\x1b[0m: {:?}",e);
                        let ptr=&e.clone() as *const input::InputEvent as *const u8;
                        let bytes=unsafe{std::slice::from_raw_parts(ptr,input::SIZEOF_EVENT)};
                        let _=std::io::stdout().write(bytes);
                        let _=std::io::stdout().flush();
                    }
                }
            });
        }else{
            for e in &events{
                log::debug!("\x1b[36mnormal event\x1b[0m: {:?}",e);
                if e.type_ as u32==input::EV_KEY{
                    tx.send(e.code).unwrap();
                }
                let ptr=&e.clone() as *const input::InputEvent as *const u8;
                let bytes=unsafe{std::slice::from_raw_parts(ptr,input::SIZEOF_EVENT)};
                let _=std::io::stdout().write(bytes);
                let _=std::io::stdout().flush();
            }
        }
        events.clear();
    }
}
