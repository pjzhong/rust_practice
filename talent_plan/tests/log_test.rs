use log::debug;
use log::error;
use log::info;
use log::warn;
use env_logger::Env;

#[test]
fn slog_test() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    debug!("Mary has a little lamb");
    error!("{}", "Its fleece was white as snow");
    info!("{:?}", "And every where that Mary went");
    warn!("{:#?}", "The lamb was sure to go");
}
