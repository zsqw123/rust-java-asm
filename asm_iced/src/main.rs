use crate::app::PanelState;

pub mod app;


fn main() -> iced::Result {
    iced::run(
        "Asm App 中文😊", PanelState::update, PanelState::view
    )
}