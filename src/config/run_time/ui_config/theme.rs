use serde::{Deserialize, Serialize};

/// Type alias for the type used by [imgui-rs] for colours
pub type Colour = mint::Vector4<f32>;

/// Colour arrays for use with [`imgui`]
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default)]
pub struct Theme {
    pub text: TextColours,
    pub value: ValueColours,
    pub severity: SeverityColours,
}

/// Theme struct for general text colours that would be used with most normal (non-specialised) text
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct TextColours {
    pub normal: Colour,
    pub subtle: Colour,
    pub accent: Colour,
}
impl Default for TextColours {
    fn default() -> Self {
        Self {
            normal: [1.0, 1.0, 1.0, 1.0].into(),       // Full white
            subtle: [0.8, 0.8, 0.8, 1.0].into(),       // Slightly grey
            accent: [0.223, 0.287, 0.783, 1.0].into(), // Darkish pale blue
        }
    }
}
/// Theme struct that contains colours for different types of values that can be displayed.
///
/// For example, there are different levels for each of the possible values of [tracing]'s [tracing::Level], and for function names
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ValueColours {
    /// Colour for the associated [tracing::Level::TRACE]
    pub level_trace: Colour,
    /// Colour for the associated [tracing::Level::DEBUG]
    pub level_debug: Colour,
    /// Colour for the associated [tracing::Level::INFO]
    pub level_info: Colour,
    /// Colour for the associated [tracing::Level::WARN]
    pub level_warn: Colour,
    /// Colour for the associated [tracing::Level::ERROR]
    pub level_error: Colour,

    /// A value that is the name of a [tracing::event::Event] or [tracing::span::Span]
    pub tracing_event_name: Colour,
    /// The name of a field attached to a span or event from [tracing]
    pub tracing_event_field_name: Colour,
    /// The value of a field in a [tracing] span/event
    pub tracing_event_field_value: Colour,

    /// The shown value represents the name of a function in some code somewhere
    pub function_name: Colour,
    /// A value that points to a file
    pub file_location: Colour,

    /// The textual representation of an error, or the message associated with that error
    pub error_message: Colour,

    /// The colour for a label of a value
    pub value_label: Colour,

    /// Miscellaneous value that doesn't match any of the other values
    pub misc_value: Colour,
    /// Represents a value that is non-existent/missing
    pub missing_value: Colour,
    /// A textual symbol, like hyphens, colons, commas, brackets, etc
    pub symbol: Colour,
    /// A numeric value of some sort
    pub number: Colour,
}

impl Default for ValueColours {
    fn default() -> Self {
        Self {
            // Tracing event levels
            level_trace: [0.815, 0.548, 1.0, 1.0].into(), // Pale pink
            level_debug: [0.548, 0.656, 1.0, 1.0].into(), // Pale blue
            level_info: [0.63, 1.0, 0.63, 1.0].into(),    // Very pale green
            level_warn: [1.0, 0.683, 0.0, 1.0].into(),    // Yellow
            level_error: [1.0, 0.0, 0.0, 1.0].into(),     // Bright red

            tracing_event_name: [1.0, 0.324, 0.324, 1.0].into(), // Pale red
            tracing_event_field_name: [0.224, 0.462, 1.0, 1.0].into(), // Medium pale blue
            tracing_event_field_value: [0.516, 0.664, 1.0, 1.0].into(), // Lighter blue

            function_name: [0.0, 1.0, 0.872, 1.0].into(), // Light cyan
            file_location: [0.17, 0.4, 1.0, 1.0].into(),  // Blue, hyperlink style

            error_message: [1.0, 0.4, 0.4, 1.0].into(), // Slightly pale red

            value_label: [0.9, 0.95, 1.0, 1.0].into(), // Ever so slightly blue

            misc_value: [0.31, 1.0, 0.31, 1.0].into(), // Pale green
            missing_value: [0.27, 0.27, 0.27, 1.0].into(), // Dark grey
            symbol: [0.93, 1.0, 0.79, 1.0].into(),     // Off-white (ultra pale green)
            number: [0.05, 1.0, 0.68, 1.0].into(), // Green with a tint of blue
        }
    }
}

/// Colours for things that may have a severity, such as the status of something
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SeverityColours {
    /// Some information that indicates something good
    pub good: Colour,
    /// Some information that is neither positive nor negative
    pub neutral: Colour,
    /// Something that is there to provide extra information
    pub note: Colour,
    /// Something went wrong, but not necessarily too badly
    pub warning: Colour,
    /// Something went *badly* wrong somewhere
    pub very_bad: Colour,
}

impl Default for SeverityColours {
    fn default() -> Self {
        Self {
            good: [0.3, 1.0, 0.3, 1.0].into(),       // Green
            neutral: [0.75, 0.75, 0.75, 1.0].into(), // Grey
            note: [0.75, 0.75, 0.75, 1.0].into(),    // Grey
            warning: [1.0, 0.5, 0.0, 1.0].into(),    // Amber
            very_bad: [1.0, 0.0, 0.0, 1.0].into(),   // Red
        }
    }
}
