use wlroots::{Compositor, OutputBuilder, OutputBuilderResult, OutputHandle, OutputHandler,
              OutputManagerHandler};

pub struct OutputManager {
    //TODO
}

pub struct Output {
    // TODO fullscreen view
    // TODO layers
    // TODO last_frame: Duration
    // TODO damage
    // TODO usable_area: Area
    pub output: OutputHandle
}

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             _: &mut Compositor,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        let output = builder.handle();
        let res = builder.build_best_mode(Output::new(output));
        // TODO Use output config to set:
        // * mode
        // * enabled/disabled
        // * layout
        // * scale
        // * transform

        // TODO Go through seat in compositor state and configure the seat

        // TODO arrange layers

        // TODO Damage tracking: damage the whole output (because it just started)
        Some(res)
    }
}

impl OutputManager {
    pub fn new() -> Self {
        OutputManager {}
    }
}

impl OutputHandler for Output {}

impl Output {
    pub fn new(output: OutputHandle) -> Self {
        Output { output }
    }
}
