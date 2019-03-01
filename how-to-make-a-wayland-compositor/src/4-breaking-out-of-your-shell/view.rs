use Shell;

pub struct View {
    pub shell: Shell,
    pub mapped: bool,
    pub x: i32,
    pub y: i32,
}


impl View {
    pub fn new(shell: impl Into<Shell>) -> Self {
        View { shell: shell.into(), mapped: false, x: 0, y: 0 }
    }
}
