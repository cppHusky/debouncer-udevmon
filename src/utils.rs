pub struct Logger;
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
pub static LOGGER:Logger=Logger;
impl Logger{
    pub fn init(&'static self){
        let log_level=if cfg!(debug_assertions){
            log::LevelFilter::Trace
        }else{
            log::LevelFilter::Warn
        };
        if let Err(e)=log::set_logger(self).map(|_|log::set_max_level(log_level)){
            eprintln!("Failed to initialize logger: {}",e);
        };
    }
}
