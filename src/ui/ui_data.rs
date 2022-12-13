#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct UiData {
    pub windows: ShownWindows,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ShownWindows {
    pub show_demo_window: bool,
    pub show_metrics_window: bool,
    pub show_ui_management_window: bool,
}

impl Default for UiData {
    fn default() -> Self {
        Self {
            windows: ShownWindows {
                show_demo_window: true,
                show_metrics_window: true,
                show_ui_management_window: true,
            },
        }
    }
}
