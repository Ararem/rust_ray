use nameof::{name_of, name_of_type};
use std::fmt::{Debug, Display, Formatter};
use std::time::{Duration, Instant};

/// A struct that can be used to time operations, and can be logged by [tracing] (just use % operator)
///
/// Create this before the span, create the span with `timer=tracing::field::Empty`, and on exit of the span record the [SpanTimeElapsedField] `span.record("timer", my_span_time_elapsed_field_i_created_earlier);`
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SpanTimeElapsedField {
    pub start: Instant,
}

impl SpanTimeElapsedField {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now() - self.start
    }
}

impl Display for SpanTimeElapsedField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", humantime::format_duration(self.elapsed()))
    }
}
impl Debug for SpanTimeElapsedField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(name_of_type!(SpanTimeElapsedField))
            .field(name_of!(start in Self), &self.start)
            .field("elapsed", &self.elapsed())
            .finish()
    }
}
