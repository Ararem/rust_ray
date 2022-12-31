use color_eyre::Report;
use imgui::sys::ImGuiID;
use crate::FallibleFn;

pub struct PopupManager {
    pub (in super)popups: Vec<Popup>,
}

pub struct Popup {
    pub name: String,

    /// Renders/displays the popup
    pub render: fn(&imgui::Ui) -> FallibleFn,
    //
    opened: bool,
}

impl PopupManager {
    pub fn show_popup(&mut self, popup_render: fn(&imgui::Ui) -> FallibleFn){
        self.popups.push(Popup{
            render: popup_render,
            opened: true,
        });
    }
    pub fn close_popup(&mut self, popup: &Popup) -> FallibleFn{
        // First, check that we own the popup
        // There should only be one PopupManager instance so this should always be true
        if self.popups.contains(popup) == false{
            return Err(Report::msg("the current PopupManager does not contain the passed popup"));
        }

        //
    }
}
