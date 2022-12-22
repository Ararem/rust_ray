use crate::engine::EngineData;
use crate::program::ProgramData;
use crate::ui::UiData;

/// Main data structure used
//TODO: Display trait implementation for ProgramData
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ThreadData {
    pub ui_data: UiData,
    pub engine_data: EngineData,
    pub program_data: ProgramData,
}