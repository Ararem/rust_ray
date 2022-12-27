use crate::ui::build_ui_impl::UiItem;
use crate::ui::ui_system::UiManagers;
use crate::FallibleFn;
use imgui::*;
use crate::config::Config;

impl UiItem for UiManagers {
    fn render(&mut self, ui: &Ui, config: Config) -> FallibleFn {
        self.font_manager.render(ui, config)?;
        self.frame_info.render(ui, config)?;

        Ok(())
    }
}
