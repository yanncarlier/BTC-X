//! JSON serialisation helpers with a configurable indent width.

use serde::Serialize;

/// Serialises `value` using Python's `json.dumps` compatible layout.
///
/// The Python originals indent with either two or four spaces depending on the
/// script; `serde_json`'s built-in pretty printer is hard coded to two, so this
/// helper wraps a `PrettyFormatter` with a custom indent.
///
/// # Panics
/// Panics if serialisation fails, which can only happen for a type whose
/// `Serialize` implementation itself fails.
pub fn to_string_pretty<T: Serialize>(value: &T, indent: u8) -> String {
    let spaces = vec![b' '; indent as usize];
    let formatter = serde_json::ser::PrettyFormatter::with_indent(&spaces);
    let mut buffer = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut buffer, formatter);
    value
        .serialize(&mut serializer)
        .expect("JSON serialisation failed");
    String::from_utf8(buffer).expect("serde_json always emits valid UTF-8")
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::*;

    #[derive(Serialize)]
    struct Sample {
        name: &'static str,
        values: Vec<u32>,
    }

    #[test]
    fn honours_the_requested_indent_width() {
        let sample = Sample {
            name: "demo",
            values: vec![1, 2],
        };

        assert_eq!(
            to_string_pretty(&sample, 2),
            "{\n  \"name\": \"demo\",\n  \"values\": [\n    1,\n    2\n  ]\n}"
        );
        assert_eq!(
            to_string_pretty(&sample, 4),
            "{\n    \"name\": \"demo\",\n    \"values\": [\n        1,\n        2\n    ]\n}"
        );
    }
}
