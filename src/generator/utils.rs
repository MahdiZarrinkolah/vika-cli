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
}
