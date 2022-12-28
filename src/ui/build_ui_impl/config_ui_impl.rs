use crate::config::init_time::InitTimeAppConfig;
use crate::config::run_time::RuntimeAppConfig;
use crate::ui::build_ui_impl::UiItem;
use crate::FallibleFn;
use imgui::Ui;

impl UiItem for InitTimeAppConfig {
    fn render(&mut self, _ui: &Ui) -> FallibleFn {
        Ok(())
    }
}
impl UiItem for RuntimeAppConfig {
    fn render(&mut self, _ui: &Ui) -> FallibleFn {
        Ok(())
    }
}