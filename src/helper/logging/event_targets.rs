//! This file contains event targets used to indicate what system/section an event is part of
//! For example, [UI_TRACE_EVENT_LOOP] indicates that the event comes from the UI system's event loop, in a hot-path (i.e. called each frame, maybe multiple times), and will probably spam the logs
macro_rules! target {
    ($name:ident, $docs:expr) => {
        #[doc=indoc::indoc!{$docs}]
        // #[allow(dead_code)]
        pub const $name: &str = concat!("rust_ray::target::", stringify!($name));
    };
}

// ===== Ui =====
target!(
    UI_TRACE_EVENT_LOOP,
    r"Log event that traces the execution of the ui event loop. not important unless debugging the ui event loop"
);
target!(
    UI_TRACE_RENDER,
    r"Log event that traces the rendering/drawing of the ui. not important unless debugging rendering issues"
);
target!(
    UI_TRACE_BUILD_INTERFACE,
    r"
    Log event that traces the execution of the building of the ui.

    Leave this off all the time. Like actually it's completely useless to enable it
"
);
target!(UI_DEBUG_USER_INTERACTION, r"Event for when the user does something to the ui (but why would they want to do that?)");
target!(
    UI_TRACE_USER_INPUT,
    r"
    Events for when checking input to see if the user has pressed anything
"
);
target!(
    UI_TRACE_MISC_PERFRAME_CALCULATIONS,
    r"
    General UI calculations that take place every frame. For example, may be calculating the window size to
"
);

target!(UI_DEBUG_GENERAL, r"General debug events relating to the UI");

// ===== Engine =====
target!(ENGINE_TRACE_GLOBAL_LOOP, r"poll events when the engine does it's global loop");

// ===== Program/Main =====
target!(MAIN_DEBUG_GENERAL, r#"main.rs general logs, like initialising something"#);
target!(PROGRAM_INFO_LIFECYCLE, r#"program lifecycle events like "the app is starting" and "the app completed""#);
target!(PROGRAM_DEBUG_GENERAL, r"general program events, initialising something");
target!(PROGRAM_TRACE_THREAD_STATUS_POLL, r"poll events when the program thread checks the status of all the other threads");
target!(PROGRAM_TRACE_GLOBAL_LOOP, r"poll events when the program does it's global loop");

// ===== Threads/Inter-thread communication =====
target!(
    THREAD_TRACE_MESSAGE_IGNORED,
    r"An event that is logged every time a inter-thread message is ignored because it was for the wrong thread. you probably never want to enable this target"
);
target!(
    THREAD_DEBUG_MESSAGE_RECEIVED,
    r"An event that is logged every time a inter-thread message is ignored because it was for the wrong thread. not important unless debugging"
);
target!(
    THREAD_DEBUG_MESSAGE_SEND,
    r"An event that is logged every time a inter-thread message is sent to another thread. not important unless debugging"
);
target!(THREAD_DEBUG_GENERAL, r"Event relating to inter-thread stuff, like barriers, thread spawns, and joining");
target!(
    THREAD_DEBUG_MESSENGER_LIFETIME,
    r"
    An event that tracks the lifetime of an inter-thread messaging object (sender or receiver)

    For example, would be used when creating a new message channel, and when it is dropped
"
);
target!(
    THREAD_TRACE_MESSAGE_LOOP,
    r#"More general messages about the inter-thread messaging loops, such as "checking for messages" and "got 5 messages this frame""#
);
target!(
    THREAD_TRACE_MUTEX_SYNC,
    r#"
    Events that are logged when trying to synchronise between threads using [std::sync::mutex::Mutex]
"#
);

// ===== RESOURCES & FILESYSTEM =====
target!(
    RESOURCES_DEBUG_LOAD,
    r#"
    Events for when resources are being loaded from disk/memory/etc, or when looking for said resources
"#
);
target!(
    RESOURCES_WARNING_NON_FATAL,
    r"
    Warnings generated when processing resources.

    Separated from [GENERAL_WARNING_NON_FATAL] since these are more of an assets issue rather than a coding issue
"
);
target!(
    FONT_MANAGER_TRACE_FONT_LOAD,
    r"
    Trace events for when the font manager is loading fonts
"
);

// ===== DATA/MEMORY =====
target!(DATA_DEBUG_DUMP_OBJECT, r"a log event that dumps the value of some object or buffer");

// ===== BAD THINGS =====
target!(
    GENERAL_WARNING_NON_FATAL,
    r"
A general warning that something went wrong, but it shouldn't cause the app to fail.

# Note:
You should always keep this target enabled.
Prefer using this over the event target that corresponds to the rest of the code in the scope, as they might be disabled, but this should alwasy stay enabled
"
);

target!(
    GENERAL_ERROR_FATAL,
    r"
An error occured, which should cause the app to exit.

# Note:
You should always keep this enabled.
"
);

// ===== REALLY BAD THINGS =====
target!(
    DOMINO_EFFECT_FAILURE,
    r"
An error that occured because there was some sort of invalid state caused by an error somewhere else that propagated here.

For example, if you couldn't access the database because it was corrupted (by some other code), then you would classify as this event (since this code can't run because something else failed).
If are logging messages with this target, you should also make sure your code (and/or even the app) exits ASAP
"
);
target!(
    REALLY_FUCKING_BAD_UNREACHABLE,
    r"
    YOU DO NOT WANT TO EVER SEE ONE OF THESE

    This is a real edge-case error that should never happen, but is kept in just-in-case something really goes wrong and the app keeps running.
    Use these where you would normally panic (panic=bad, kapishe?)
"
);
