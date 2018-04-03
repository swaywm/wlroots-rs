use super::config::MainConfig;
use wlroots::OutputLayout;

#[derive(Debug)]
pub struct Server {
    config: MainConfig,
    layout: OutputLayout
}

impl Server {
    pub fn new(config: MainConfig) -> Self {
        Server { config,
                 layout: OutputLayout::new(None).expect("Could not construct an OutputLayout") }
    }
}
