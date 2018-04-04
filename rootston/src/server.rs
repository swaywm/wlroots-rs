use super::config::MainConfig;
use wlroots::{OutputHandle, OutputLayout};

#[derive(Debug)]
pub struct Server {
    pub config: MainConfig,
    pub layout: OutputLayout,
    pub outputs: Vec<OutputHandle>
}

impl Server {
    pub fn new(config: MainConfig) -> Self {
        Server { config,
                 layout: OutputLayout::new(None).expect("Could not construct an OutputLayout"),
                 outputs: vec![] }
    }
}
