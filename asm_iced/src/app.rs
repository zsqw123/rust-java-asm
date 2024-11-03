use iced::widget::{pane_grid, text, PaneGrid};
use java_asm_server::ui::{App, Content, Left};

#[derive(Default)]
pub struct IcedApp {
    pub server_app: App,
}

// state
pub struct PanelState {
    pub panels: pane_grid::State<Panel>,
}

pub enum Panel {
    L(Left),
    R(Content),
}

impl Default for PanelState {
    fn default() -> Self {
        let (panels, _) = pane_grid::State::new(
            Panel::L(Left::default()),
        );
        Self { panels }
    }
}

impl PanelState {
    pub fn view(&self) -> PaneGrid<'_, ()> {
        pane_grid(&self.panels, |pane, state, is_maximized| {
            pane_grid::Content::new(match state {
                Panel::L(left) => {
                    text("This is left pane 中文😊")
                }
                Panel::R(right) => {
                    text("This is right pane")
                }
            })
        })
    }
    
    pub fn update(&mut self, _message: ()) {
        // do nothing
    }
}

