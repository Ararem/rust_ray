use crate::engine::EngineData;
use crate::ui::ui_data::UiData;

/// Main data structure used
//TODO: Display trait implementation for ProgramData
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ProgramData {
    pub ui_data: UiData,
    pub engine_data: EngineData,
}