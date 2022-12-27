use imgui::Ui;
use crate::config::Config;
use crate::config::init_time::InitTimeAppConfig;
use crate::config::run_time::RuntimeAppConfig;
use crate::FallibleFn;
use crate::ui::build_ui_impl::UiItem;

impl UiItem for InitTimeAppConfig{
    fn render(&mut self, ui: &Ui, config: Config) -> FallibleFn {
        Ok(())
    }
}
impl UiItem for RuntimeAppConfig{
    fn render(&mut self, ui: &Ui, config: Config) -> FallibleFn {
        Ok(())
    }
}
