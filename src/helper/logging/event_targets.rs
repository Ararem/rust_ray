//! This file contains event targets used to indicate what system/section an event is part of
//! For example, [UI_SPAMMY] indicates that the event comes from the UI system, in a hot-path (i.e. called each frame, maybe multiple times), and will probably spam the logs

macro_rules! target {
    ($name:ident, $docs:literal) => {
        #[doc=$docs]
        #[allow(dead_code)]
        pub const $name: &str = stringify!(rust_ray::$name);
    };
}
target!(
    UI_PERFRAME_SPAMMY,
    r"High-frequency (i.e. every frame) logs from the UI"
);
target!(
    PROGRAM_MESSAGE_POLL_SPAMMY,
    r"High-frequency (i.e. every frame) logs from the program message poll loop"
);
target!(
    PROGRAM_MESSAGE_POLL,
    r"Logs from the program message poll loop that shouldn't be called super often, or spam the log"
);
target!(UI_USER_EVENT, r"Interaction between the user and the UI");
target!(
    DATA_DUMP,
    r"A log call that dumps the value of some data, such as an array of bytes, or a received data packet"
);
