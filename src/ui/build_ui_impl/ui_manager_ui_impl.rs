use crate::ui::build_ui_impl::UiItem;
use crate::ui::ui_system::UiManagers;
use crate::FallibleFn;
use imgui::*;

impl UiItem for UiManagers {
    fn render(&mut self, ui: &Ui, visible: bool) -> FallibleFn {
        self.font_manager.render(ui, visible)?;
        self.frame_info.render(ui, visible)?;

        Ok(())
    }
}
