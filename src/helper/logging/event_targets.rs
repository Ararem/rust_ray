//! This file contains event targets used to indicate what system/section an event is part of
//! For example, [UI_TRACE_EVENT_LOOP] indicates that the event comes from the UI system's event loop, in a hot-path (i.e. called each frame, maybe multiple times), and will probably spam the logs
macro_rules! target {
    ($name:ident, $docs:expr) => {
        #[doc=$docs]
        // #[allow(dead_code)]
        pub const $name: &str = stringify!(rust_ray::target::$name);
    };
}
// ===== Ui =====
target!(UI_TRACE_EVENT_LOOP, r"Log event that traces the execution of the ui event loop. not important unless debugging the ui event loop");
target!(UI_TRACE_RENDER, r"Log event that traces the rendering/drawing of the ui. not important unless debugging rendering issues");
target!(UI_TRACE_BUILD_INTERFACE, r"Log event that traces the execution of the building of the ui. not important unless debugging the ui not working properly");
target!(UI_DEBUG_USER_INTERACTION, r"Event for when the user does something to the ui (but why would they want to do that?)");

// ===== Engine =====

// ===== Program/Main =====
target!(MAIN_DEBUG_GENERAL,r#"main.rs general logs, like initialising something"#);
target!(PROGRAM_INFO_LIFECYCLE, r###"program lifecycle events like "the app is starting" and "the app completed""###);
target!(PROGRAM_DEBUG_GENERAL, r###"general program events, initialising something"###);
target!(PROGRAM_TRACE_THREAD_STATUS_POLL, r#"poll events when the program thread checks the status of all the other threads"#);
target!(PROGRAM_TRACE_GLOBAL_LOOP, r#"poll events when the program does it's global loop"#);

// ===== Threads/Inter-thread communication =====
target!(THREAD_TRACE_MESSAGE_IGNORED, r"An event that is logged every time a inter-thread message is ignored because it was for the wrong thread. you probably never want to enable this target");
target!(THREAD_DEBUG_MESSAGE_RECEIVED, r"An event that is logged every time a inter-thread message is ignored because it was for the wrong thread. not important unless debugging");
target!(THREAD_DEBUG_MESSAGE_SEND, r"An event that is logged every time a inter-thread message is sent to another thread. not important unless debugging");
target!(THREAD_DEBUG_GENERAL, r"Event relating to inter-thread stuff, like barriers, thread spawns, and joining");
target!(THREAD_DEBUG_MESSENGER_LIFETIME,
    indoc::indoc!{r"
    An event that tracks the lifetime of an inter-thread messaging object (sender or receiver)

    For example, would be used when creating a new message channel, and when it is dropped
"});
target!(THREAD_TRACE_MESSAGE_LOOP, r#"More general messages about the inter-thread messaging loops, such as "checking for messages" and "got 5 messages this frame""#);

// ===== FILESYSTEM =====
// ===== Data/Memory =====
target!(DATA_DEBUG_DUMP_OBJECT, r"a log event that dumps the value of some object or buffer");

// ===== PANIC-PILL =====
target!(PANIC_PILL, r"Special internal event from the [crate::helper::panic_pill::PanicPill]");

// ===== REALLY BAD THINGS =====
target!(REALLY_FUCKING_BAD_UNREACHABLE, indoc::indoc!{r"
    YOU DO NOT WANT TO EVER SEE ONE OF THESE

    This is a real edge-case error that should never happen, but is kept in just-in-case something really goes wrong and the app keeps running.
    Use these where you would normally panic (panic=bad, kapishe?)
"});