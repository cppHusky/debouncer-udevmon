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
#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,serde::Deserialize)]
pub struct Config{
    pub exceptions:Vec<u16>,
}
impl Config{
    pub fn new()->Self{
        Config{
            exceptions:vec![],
        }
    }
    pub fn init()->Self{
        let builder=config::Config::builder();
        let builder=match builder.set_default("exceptions",Vec::<u16>::new()){
            Ok(b)=>b,
            Err(e)=>{
                log::error!("Failed to set default config: {}",e);
                return Self::new();
            }
        };
        let builder=builder.add_source(config::File::with_name("/etc/debouncer.toml"));
        let config=match builder.build(){
            Ok(c)=>c,
            Err(e)=>{
                log::warn!("Failed to build config: {}",e);
                return Self::new();
            }
        };
        match config.try_deserialize::<Config>(){
            Ok(c)=>c,
            Err(e)=>{
                log::error!("Failed to parse config file: {}",e);
                return Self::new();
            }
        }
    }
}
pub static CONFIG:std::sync::OnceLock<Config>=std::sync::OnceLock::new();
#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn default_config(){
        let config=config::Config::builder()
            .set_default("exceptions",Vec::<u16>::new())
            .unwrap()
            .build()
            .unwrap()
            .try_deserialize::<Config>()
            .unwrap();
        assert_eq!(config,Config::new());
    }
    #[test]
    #[should_panic]
    fn config_not_found(){
        let _=config::Config::builder()
            .add_source(config::File::with_name("./debouncer.toml"))
            .build()
            .unwrap();
    }
}
#[macro_export]
macro_rules! config{
    (exceptions)=>{
        &$crate::utils::CONFIG.get().unwrap().exceptions
    }
}
