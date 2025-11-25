pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

pub fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    if pascal.is_empty() {
        return pascal;
    }

    let mut chars = pascal.chars();
    let first = chars.next().unwrap().to_lowercase().next().unwrap();
    format!("{}{}", first, chars.as_str())
}

/// Sanitizes a property name to be a valid JavaScript identifier
/// Returns the original name if valid, or the name in quotes if invalid
pub fn sanitize_property_name(name: &str) -> String {
    let first_char = name.chars().next();
    let needs_quotes = match first_char {
        Some(c) if c.is_ascii_digit() => true, // starts with number
        _ => name.contains(' ') || name.contains('-') && !name.starts_with('-'), // contains spaces or hyphens (but not negative numbers)
    };

    if needs_quotes {
        format!("\"{}\"", name)
    } else {
        name.to_string()
    }
}

/// Sanitizes module names for use as directory/file names
/// Replaces spaces with hyphens, converts to lowercase, and removes other invalid characters
/// This ensures consistent casing across case-insensitive filesystems (like macOS)
pub fn sanitize_module_name(name: &str) -> String {
    name.replace(' ', "-")
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '/')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case_simple() {
        assert_eq!(to_pascal_case("hello"), "Hello");
        assert_eq!(to_pascal_case("world"), "World");
    }

    #[test]
    fn test_to_pascal_case_with_underscore() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("user_name"), "UserName");
    }

    #[test]
    fn test_to_pascal_case_with_hyphen() {
        assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(to_pascal_case("api-key"), "ApiKey");
    }

    #[test]
    fn test_to_pascal_case_with_space() {
        assert_eq!(to_pascal_case("hello world"), "HelloWorld");
        assert_eq!(to_pascal_case("user name"), "UserName");
    }

    #[test]
    fn test_to_pascal_case_mixed() {
        assert_eq!(to_pascal_case("hello_world-test"), "HelloWorldTest");
        assert_eq!(to_pascal_case("api_key-name"), "ApiKeyName");
    }

    #[test]
    fn test_to_pascal_case_empty() {
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn test_to_pascal_case_single_word() {
        assert_eq!(to_pascal_case("test"), "Test");
    }

    #[test]
    fn test_to_camel_case_simple() {
        assert_eq!(to_camel_case("hello"), "hello");
        assert_eq!(to_camel_case("world"), "world");
    }

    #[test]
    fn test_to_camel_case_with_underscore() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("user_name"), "userName");
    }

    #[test]
    fn test_to_camel_case_with_hyphen() {
        assert_eq!(to_camel_case("hello-world"), "helloWorld");
        assert_eq!(to_camel_case("api-key"), "apiKey");
    }

    #[test]
    fn test_to_camel_case_empty() {
        assert_eq!(to_camel_case(""), "");
    }

    #[test]
    fn test_to_camel_case_single_word() {
        assert_eq!(to_camel_case("test"), "test");
    }

    #[test]
    fn test_to_camel_case_mixed() {
        assert_eq!(to_camel_case("hello_world-test"), "helloWorldTest");
    }

    #[test]
    fn test_sanitize_property_name_valid() {
        assert_eq!(sanitize_property_name("name"), "name");
        assert_eq!(sanitize_property_name("userName"), "userName");
        assert_eq!(sanitize_property_name("_private"), "_private");
    }

    #[test]
    fn test_sanitize_property_name_starts_with_number() {
        assert_eq!(sanitize_property_name("2xl"), "\"2xl\"");
        assert_eq!(sanitize_property_name("3xl"), "\"3xl\"");
        assert_eq!(sanitize_property_name("404error"), "\"404error\"");
    }

    #[test]
    fn test_sanitize_property_name_with_spaces() {
        assert_eq!(
            sanitize_property_name("Translation name"),
            "\"Translation name\""
        );
        assert_eq!(sanitize_property_name("user name"), "\"user name\"");
    }

    #[test]
    fn test_sanitize_module_name() {
        assert_eq!(sanitize_module_name("cart"), "cart");
        assert_eq!(sanitize_module_name("AI Chat"), "ai-chat");
        assert_eq!(sanitize_module_name("admin/orders"), "admin/orders");
        assert_eq!(sanitize_module_name("test module name"), "test-module-name");
        assert_eq!(sanitize_module_name("Inventory"), "inventory");
    }
}
