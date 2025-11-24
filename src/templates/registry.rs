use std::str::FromStr;

/// Template identifier enum for strongly-typed template references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateId {
    TypeInterface,
    TypeEnum,
    TypeAlias,
    ZodSchema,
    ZodEnum,
    ApiClientFetch,
}

impl TemplateId {
    /// Returns the template filename (without extension).
    pub fn name(self) -> &'static str {
        match self {
            TemplateId::TypeInterface => "type-interface",
            TemplateId::TypeEnum => "type-enum",
            TemplateId::TypeAlias => "type-alias",
            TemplateId::ZodSchema => "zod-schema",
            TemplateId::ZodEnum => "zod-enum",
            TemplateId::ApiClientFetch => "api-client-fetch",
        }
    }

    /// Returns the full template filename with extension.
    pub fn filename(self) -> String {
        format!("{}.tera", self.name())
    }

    /// Returns all available template IDs.
    pub fn all() -> Vec<TemplateId> {
        vec![
            TemplateId::TypeInterface,
            TemplateId::TypeEnum,
            TemplateId::TypeAlias,
            TemplateId::ZodSchema,
            TemplateId::ZodEnum,
            TemplateId::ApiClientFetch,
        ]
    }
}

impl FromStr for TemplateId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "type-interface" => Ok(TemplateId::TypeInterface),
            "type-enum" => Ok(TemplateId::TypeEnum),
            "type-alias" => Ok(TemplateId::TypeAlias),
            "zod-schema" => Ok(TemplateId::ZodSchema),
            "zod-enum" => Ok(TemplateId::ZodEnum),
            "api-client-fetch" => Ok(TemplateId::ApiClientFetch),
            _ => Err(format!("Unknown template: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_id_name() {
        assert_eq!(TemplateId::TypeInterface.name(), "type-interface");
        assert_eq!(TemplateId::ZodSchema.name(), "zod-schema");
    }

    #[test]
    fn test_template_id_filename() {
        assert_eq!(
            TemplateId::TypeInterface.filename(),
            "type-interface.tera"
        );
    }

    #[test]
    fn test_template_id_from_str() {
        assert_eq!(
            TemplateId::from_str("type-interface").unwrap(),
            TemplateId::TypeInterface
        );
        assert!(TemplateId::from_str("unknown").is_err());
    }

    #[test]
    fn test_template_id_all() {
        let all = TemplateId::all();
        assert_eq!(all.len(), 6);
        assert!(all.contains(&TemplateId::TypeInterface));
    }
}

