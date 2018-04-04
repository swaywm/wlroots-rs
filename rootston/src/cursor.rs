use wlroots;

pub struct CursorHandler {}

#[derive(Debug)]
pub struct Cursor {
    pub cursor: wlroots::CursorHandle
}

impl CursorHandler {
    fn new() -> CursorHandler {
        CursorHandler {}
    }
}

impl Cursor {
    pub fn new() -> Self {
        Cursor { cursor: wlroots::Cursor::create(Box::new(CursorHandler::new())) }
    }
}

impl wlroots::CursorHandler for CursorHandler {}
