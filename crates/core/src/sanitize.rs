/// Sanitize text by removing control characters and truncating
pub fn sanitize_text(input: &str, max_length: usize) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .take(max_length)
        .collect()
}

/// Sanitize node name - removes special characters
pub fn sanitize_node_name(name: &str) -> String {
    let text = sanitize_text(name, 200);
    text.chars()
        .filter(|c| !matches!(c, '<' | '>' | '{' | '}' | '[' | ']'))
        .collect()
}

/// Sanitize entire draft node structure
pub fn sanitize_draft(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            serde_json::Value::String(sanitize_text(s, 10000))
        }
        serde_json::Value::Object(map) => {
            let mut result = serde_json::Map::new();
            for (k, v) in map.iter() {
                result.insert(k.clone(), sanitize_draft(v));
            }
            serde_json::Value::Object(result)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(sanitize_draft).collect())
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_text_removes_control_chars() {
        let input = "hello\x00world";
        assert_eq!(sanitize_text(input, 100), "helloworld");
    }

    #[test]
    fn test_sanitize_text_keeps_newlines() {
        let input = "hello\nworld";
        assert_eq!(sanitize_text(input, 100), "hello\nworld");
    }

    #[test]
    fn test_sanitize_text_truncates() {
        let input = "hello world";
        assert_eq!(sanitize_text(input, 5), "hello");
    }

    #[test]
    fn test_sanitize_node_name() {
        let input = "hello<script>world[1]";
        assert_eq!(sanitize_node_name(input), "helloscriptworld1");
    }
}
