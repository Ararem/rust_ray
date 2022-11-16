use imgui::Ui;
use tracing::info;
use super::engine;
use std::fmt::Display;

#[derive(Debug, Copy, Clone)]
pub(crate) struct Program{
    pub test: bool,
}

impl Program {
    pub(crate) fn init(){

    }

    /// Called every frame, only place where rendering can occur
    pub(crate) fn tick(mut self, ui: &Ui)
    {
        ui.show_demo_window(&mut self.test);
        if ui.button("Crash") {
            panic!("Test");
        }
    }
}