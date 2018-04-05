use super::config::MainConfig;
use wlroots::{self, OutputHandle, OutputLayoutHandle};

struct OutputLayout;
impl wlroots::OutputLayoutHandler for OutputLayout {}

#[derive(Debug)]
pub struct Server {
    pub config: MainConfig,
    pub layout: OutputLayoutHandle,
    pub outputs: Vec<OutputHandle>
}

impl Server {
    pub fn new(config: MainConfig) -> Self {
        Server { config,
                 layout: wlroots::OutputLayout::create(Box::new(OutputLayout)),
                 outputs: vec![] }
    }
}
