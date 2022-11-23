//! This file contains event targets used to indicate what system/section an event is part of
//! For example, [UI_SPAMMY] indicates that the event comes from the UI system, in a hot-path (i.e. called each frame, maybe multiple times), and will probably spam the logs

macro_rules! target {
    ($name:ident) => {pub const $name: &str = stringify!($name);};
}
target!(UI_SPAMMY);
target!(PROGRAM_MAIN);
target!(UI_USER_EVENT);
target!();