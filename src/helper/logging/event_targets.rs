//! This file contains event targets used to indicate what system/section an event is part of
//! For example, [UI_SPAMMY] indicates that the event comes from the UI system, in a hot-path (i.e. called each frame, maybe multiple times), and will probably spam the logs

macro_rules! target {
    ($name:ident, $docs:literal) => {
        #[doc=$docs]
        #[allow(dead_code)]
        pub const $name: &str = stringify!(rust_ray::$name);
    };
}
target!(UI_SPAMMY, r"High-frequency (i.e. every frame) logs");
target!(
    PROGRAM_CORE,
    r"Part of the core program (such as initialisation, or the main event loop)"
);
target!(UI_USER_EVENT, r"Interaction between the user and the UI");
target!(
    DATA_DUMP,
    r"A log call that dumps the value of some data, such as an array of bytes, or a received data packet"
);
