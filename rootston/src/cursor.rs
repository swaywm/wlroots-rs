use wlroots;

#[allow(dead_code)]
pub struct CursorHandler {}

#[derive(Debug)]
pub struct Cursor {
    pub cursor: wlroots::CursorHandle
}

#[allow(dead_code)]
impl CursorHandler {
    #[allow(dead_code)]
    fn new() -> CursorHandler {
        CursorHandler {}
    }
}

impl Cursor {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Cursor { cursor: wlroots::Cursor::create(Box::new(CursorHandler::new())) }
    }
}

#[allow(dead_code)]
impl wlroots::CursorHandler for CursorHandler {}
