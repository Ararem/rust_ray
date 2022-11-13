use imgui::Ui;
use tracing::info;

pub(crate) fn tick(ui: &Ui){
if ui.button("Test") {info!("test button::click()");}
}