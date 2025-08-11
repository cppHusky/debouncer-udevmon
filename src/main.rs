mod input;
mod utils;
fn main(){
    utils::LOGGER.init();
    #[allow(unused_variables)]
    trpl::run(async{
        let (tx,rx)=trpl::channel::<input::InputEvent>();
        let tx_future=get_events(tx);
        let rx_future=process_events(rx);
        trpl::join(tx_future,rx_future).await;
    });
}
async fn get_events(tx:trpl::Sender<input::InputEvent>){
    use std::io::Read;
    log::info!("async block \x1b[4mget_events\x1b[0m started");
    loop{
        let mut input=[0;input::InputEvent::SIZE];
        if let Err(e)=std::io::stdin().read_exact(&mut input){
            log::error!("{}",e);
            std::process::exit(1);
        }else{
            let event=input::InputEvent::parse(input);
            log::trace!("new event arrived: {:?}",&event);
            if let Err(e)=tx.send(event){
                log::error!("Send the event \x1b[4m{:?}\x1b[0m failed: {}",event,e);
            }
        }
        trpl::yield_now().await
    }
}
async fn process_events(mut rx:trpl::Receiver<input::InputEvent>){
    log::info!("async block \x1b[4mprocess_events\x1b[0m started");
    let mut is_release_event_last_time:bool=false;
    let mut event_cache=Vec::<input::InputEvent>::new();
    let (mpmc_tx,_mpmc_rx)=tokio::sync::broadcast::channel::<u16>(input::KEY_CNT as usize);
    while let Some(event)=rx.recv().await{
        match event.r#type() as u32{
            input::EV_MSC=>{
                event_cache.push(event);
            }
            input::EV_KEY=>{
                let keycode=event.code();
                event_cache.push(event);
                if event.is_key_release(){
                    is_release_event_last_time=true;
                    for e in &event_cache{
                        log::debug!("\x1b[33mdelaying event\x1b[0m: {:?}",e);
                    }
                    let mpmc_rx=mpmc_tx.subscribe();
                    trpl::spawn_task(delay_events(event_cache.clone(),mpmc_rx,keycode));
                }else{
                    is_release_event_last_time=false;
                    for e in event_cache.clone(){
                        log::debug!("\x1b[36mnormal event\x1b[0m: {:?}",&e);
                        output_event(e);
                    }
                    if let Err(e)=mpmc_tx.send(keycode){
                        log::error!("Send the keycode \x1b[4m{}\x1b[0m failed: {}",keycode,e);
                    }
                }
                event_cache.clear();
            }
            input::EV_SYN=>{
                if is_release_event_last_time{
                    log::debug!("\x1b[37mneglect event\x1b[0m: {:?}",&event);
                }else{
                    log::debug!("\x1b[36mnormal event\x1b[0m: {:?}",&event);
                    output_event(event);
                }
            }
            _=>{
                log::debug!("\x1b[36mnormal event\x1b[0m: {:?}",&event);
                output_event(event);
            }
        }
    }
}
async fn delay_events(
    mut event_cache:Vec<input::InputEvent>,
    mut mpmc_rx:tokio::sync::broadcast::Receiver<u16>,
    keycode:u16
){
    static DEBOUNCE_TIME:std::time::Duration=std::time::Duration::from_millis(14);
    log::debug!("\x1b[37mA new thread on keycode {} created and waiting for press event...\x1b[0m",keycode);
    let waiting=async{
        while let Ok(recv_keycode)=mpmc_rx.recv().await{
            if recv_keycode==keycode{
                break;
            }
        }
    };
    let timer=async{
        trpl::sleep(DEBOUNCE_TIME).await;
    };
    if let trpl::Either::Right(_)=trpl::race(waiting,timer).await{
        event_cache.push(input::InputEvent::new());
        for e in &mut event_cache{
            e.time_reset();
        }
        for e in event_cache{
            log::debug!("\x1b[34mdelayed event\x1b[0m: {:?}",&e);
            output_event(e);
        }
    }
}
fn output_event(event:input::InputEvent){
    use std::io::Write;
    if let Err(e)=std::io::stdout().write(input::InputEvent::unparse(event).as_slice()){
        log::error!("{}",e);
    }
    if let Err(e)=std::io::stdout().flush(){
        log::error!("{}",e);
    }
}
