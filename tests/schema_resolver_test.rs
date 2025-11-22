#[cfg(test)]
mod tests {
    use openapiv3::OpenAPI;
    use crate::generator::schema_resolver::SchemaResolver;

    #[test]
    fn test_schema_resolver_creation() {
        let openapi = OpenAPI::default();
        let resolver = SchemaResolver::new(openapi);
        // Just test that it can be created
        assert!(true);
    }
}

