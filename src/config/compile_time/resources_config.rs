use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Regex that filters a file path to select only font files
    pub static ref FONTS_FILE_PATH_FILTER : Regex = Regex::new(r".*\.ttf$").expect("compile-time regex constant should be valid");

    /// Regex that extracts information from font file names
    ///
    /// Records 3 capture groups per match (with examples):
    /// *  `base_font_name` (e.g. "Fira Code")
    /// *  `weight_name` (e.g. "Extra Bold")
    /// *  `file_extension` (e.g. "ttf")
    pub static ref FONTS_FILE_NAME_EXTRACTOR : Regex = Regex::new(r"[\\/](?P<base_font_name>[\w \-_\.]*) \((?P<weight_name>[\w \-_\.]*)\)\.(?P<file_extension>\w+)").expect("compile-time regex constant should be valid");
}
