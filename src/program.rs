use imgui::Ui;
use tracing::{info, Level, span, trace_span};
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
    pub(crate) fn render(mut self, ui: &Ui)
    {
        if super::build_config::tracing::ENABLE_UI_TRACE { let _ = span!(parent: None, Level::TRACE, "render", "ΔT: {deltaT}", deltaT= ui.io().delta_time).enter();}
        ui.show_demo_window(&mut self.test);
        if ui.button("Crash") {
            panic!("Test");
        }
    }
}