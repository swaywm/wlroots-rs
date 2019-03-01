use wlroots::{shell::xdg_shell, wlroots_dehandle};

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum Shell {
    Xdg(xdg_shell::Handle)
}

impl Into<Shell> for xdg_shell::Handle {
    fn into(self) -> Shell { Shell::Xdg(self) }
}

impl Shell {
    #[wlroots_dehandle]
    pub fn surface(&self) -> wlroots::surface::Handle {
        match self {
            Shell::Xdg(xdg) => {
                #[dehandle] let xdg = xdg;
                xdg.surface()
            }
        }
    }

    #[wlroots_dehandle]
    pub fn set_activated(&self, activated: bool) {
        use wlroots::shell::xdg_shell::ShellState::*;
        match self {
            Shell::Xdg(xdg) => {
                #[dehandle] let xdg = xdg;
                match xdg.state() {
                    Some(TopLevel(toplevel)) => {
                        toplevel.set_activated(activated);
                    }
                    _ => {}
                }
            }
        }
    }
}
