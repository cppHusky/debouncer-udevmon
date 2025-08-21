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
    pub debounce_time:u64,
}
impl Config{
    pub fn new()->Self{
        Config{
            exceptions:vec![],
            debounce_time:14,
        }
    }
    pub fn init()->Self{
        Self::init_impl().unwrap_or_else(|e|{
            log::error!("Failed to init Config: {}",e);
            Self::new()
        })
    }
    fn init_impl()->Result<Self,config::ConfigError>{
        let config=config::Config::builder()
            .set_default("exceptions",Vec::<u16>::new())?
            .set_default("debounce_time",14)?
            .add_source(config::File::with_name("/etc/debouncer.toml"))
            .build()?;
        config.try_deserialize::<Config>()
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
            .set_default("debounce_time",14u64)
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
    };
    (debounce_time)=>{
        $crate::utils::CONFIG.get().unwrap().debounce_time
    };
}
