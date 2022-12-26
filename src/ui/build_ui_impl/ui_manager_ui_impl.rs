use imgui::*;
use crate::FallibleFn;
use crate::ui::build_ui_impl::UiItem;
use crate::ui::ui_system::UiManagers;

impl UiItem for UiManagers {
    fn render(&mut self, ui: &Ui) -> FallibleFn{
        self.font_manager.render(ui)?;
        self.frame_info.render(ui)?;

        Ok(())
    }
}