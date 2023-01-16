#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct UiData {
    pub windows: ShownWindows,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ShownWindows {
    pub show_demo_window: bool,
    pub show_metrics_window: bool,
    pub show_ui_management_window: bool,
    pub show_config_window: bool,
}

impl Default for UiData {
    fn default() -> Self {
        Self {
            windows: ShownWindows {
                show_demo_window: true,
                show_metrics_window: true,
                show_ui_management_window: true,
                show_config_window: true,
            },
        }
    }
}

// impl Display for UiData {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         writedoc! {f, r#"
//             Shown Windows:
//                 Demo Window: {demo}, Metrics Window: {metrics}, UI Managers Window: {managers} Window: {config}
//         "#, demo=self.windows.show_demo_window, metrics=self.windows.show_metrics_window, managers=self.windows.show_ui_management_window=self.windows.show_config_window}
//     }
// }
