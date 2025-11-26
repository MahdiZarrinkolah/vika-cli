use crate::error::Result;
use crate::generator::swagger_parser::{get_schema_name_from_ref, resolve_ref};
use openapiv3::{OpenAPI, ReferenceOr, Schema, SchemaKind, Type};
use std::collections::{HashMap, HashSet};

#[allow(dead_code)]
pub struct SchemaResolver {
    openapi: OpenAPI,
    resolved_cache: HashMap<String, ReferenceOr<Schema>>,
    dependency_graph: HashMap<String, Vec<String>>,
    circular_refs: HashSet<String>,
}

#[allow(dead_code)]
impl SchemaResolver {
    pub fn new(openapi: OpenAPI) -> Self {
        Self {
            openapi,
            resolved_cache: HashMap::new(),
            dependency_graph: HashMap::new(),
            circular_refs: HashSet::new(),
        }
    }

    pub fn build_dependency_graph(&mut self) -> Result<()> {
        let schema_names: Vec<String> = if let Some(components) = &self.openapi.components {
            components.schemas.keys().cloned().collect()
        } else {
            return Ok(());
        };

        for schema_name in schema_names {
            self.extract_dependencies(&schema_name, &mut HashSet::new())?;
        }
        Ok(())
    }

    fn extract_dependencies(
        &mut self,
        schema_name: &str,
        visited: &mut HashSet<String>,
    ) -> Result<Vec<String>> {
        if visited.contains(schema_name) {
            // Circular reference detected
            self.circular_refs.insert(schema_name.to_string());
            return Ok(vec![]);
        }

        if let Some(deps) = self.dependency_graph.get(schema_name) {
            return Ok(deps.clone());
        }

        visited.insert(schema_name.to_string());
        let mut dependencies = Vec::new();

        if let Some(components) = &self.openapi.components {
            if let Some(ReferenceOr::Item(schema)) = components.schemas.get(schema_name) {
                dependencies.extend(self.extract_schema_dependencies(schema, visited)?);
            }
        }

        visited.remove(schema_name);
        self.dependency_graph
            .insert(schema_name.to_string(), dependencies.clone());
        Ok(dependencies)
    }

    fn extract_schema_dependencies(
        &self,
        schema: &Schema,
        visited: &mut HashSet<String>,
    ) -> Result<Vec<String>> {
        let mut deps = Vec::new();

        match &schema.schema_kind {
            SchemaKind::Type(type_) => match type_ {
                Type::Array(array) => {
                    if let Some(items) = &array.items {
                        if let ReferenceOr::Reference { reference } = items {
                            if let Some(ref_name) = get_schema_name_from_ref(reference) {
                                deps.push(ref_name.clone());
                                if !visited.contains(&ref_name) {
                                    deps.extend(self.extract_schema_dependencies_from_ref(
                                        &ref_name, visited,
                                    )?);
                                }
                            }
                        } else if let ReferenceOr::Item(item_schema) = items {
                            deps.extend(self.extract_schema_dependencies(item_schema, visited)?);
                        }
                    }
                }
                Type::Object(object_type) => {
                    for (_, prop_schema_ref) in object_type.properties.iter() {
                        match prop_schema_ref {
                            ReferenceOr::Reference { reference } => {
                                if let Some(ref_name) = get_schema_name_from_ref(reference) {
                                    deps.push(ref_name.clone());
                                    if !visited.contains(&ref_name) {
                                        deps.extend(self.extract_schema_dependencies_from_ref(
                                            &ref_name, visited,
                                        )?);
                                    }
                                }
                            }
                            ReferenceOr::Item(prop_schema) => {
                                deps.extend(
                                    self.extract_schema_dependencies(prop_schema, visited)?,
                                );
                            }
                        }
                    }
                }
                _ => {}
            },
            SchemaKind::OneOf { one_of, .. } => {
                for item in one_of {
                    if let ReferenceOr::Reference { reference } = item {
                        if let Some(ref_name) = get_schema_name_from_ref(reference) {
                            deps.push(ref_name.clone());
                            if !visited.contains(&ref_name) {
                                deps.extend(
                                    self.extract_schema_dependencies_from_ref(&ref_name, visited)?,
                                );
                            }
                        }
                    } else if let ReferenceOr::Item(item_schema) = item {
                        deps.extend(self.extract_schema_dependencies(item_schema, visited)?);
                    }
                }
            }
            SchemaKind::AllOf { all_of, .. } => {
                for item in all_of {
                    if let ReferenceOr::Reference { reference } = item {
                        if let Some(ref_name) = get_schema_name_from_ref(reference) {
                            deps.push(ref_name.clone());
                            if !visited.contains(&ref_name) {
                                deps.extend(
                                    self.extract_schema_dependencies_from_ref(&ref_name, visited)?,
                                );
                            }
                        }
                    } else if let ReferenceOr::Item(item_schema) = item {
                        deps.extend(self.extract_schema_dependencies(item_schema, visited)?);
                    }
                }
            }
            SchemaKind::AnyOf { any_of, .. } => {
                for item in any_of {
                    if let ReferenceOr::Reference { reference } = item {
                        if let Some(ref_name) = get_schema_name_from_ref(reference) {
                            deps.push(ref_name.clone());
                            if !visited.contains(&ref_name) {
                                deps.extend(
                                    self.extract_schema_dependencies_from_ref(&ref_name, visited)?,
                                );
                            }
                        }
                    } else if let ReferenceOr::Item(item_schema) = item {
                        deps.extend(self.extract_schema_dependencies(item_schema, visited)?);
                    }
                }
            }
            _ => {}
        }

        Ok(deps)
    }

    fn extract_schema_dependencies_from_ref(
        &self,
        ref_name: &str,
        visited: &mut HashSet<String>,
    ) -> Result<Vec<String>> {
        if visited.contains(ref_name) {
            return Ok(vec![]);
        }

        visited.insert(ref_name.to_string());
        let mut deps = vec![ref_name.to_string()];

        if let Some(components) = &self.openapi.components {
            if let Some(ReferenceOr::Item(schema)) = components.schemas.get(ref_name) {
                deps.extend(self.extract_schema_dependencies(schema, visited)?);
            }
        }

        visited.remove(ref_name);
        Ok(deps)
    }

    pub fn resolve_schema_ref(&mut self, ref_path: &str) -> Result<ReferenceOr<Schema>> {
        if let Some(cached) = self.resolved_cache.get(ref_path) {
            return Ok(cached.clone());
        }

        let resolved = resolve_ref(&self.openapi, ref_path)?;
        self.resolved_cache
            .insert(ref_path.to_string(), resolved.clone());
        Ok(resolved)
    }

    pub fn resolve_with_dependencies(&mut self, schema_name: &str) -> Result<Vec<String>> {
        let mut all_schemas = vec![schema_name.to_string()];

        if let Some(deps) = self.dependency_graph.get(schema_name) {
            for dep in deps {
                if !all_schemas.contains(dep) {
                    all_schemas.push(dep.clone());
                }
            }
        }

        Ok(all_schemas)
    }

    pub fn detect_circular_dependencies(&self) -> Result<Vec<Vec<String>>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        if let Some(components) = &self.openapi.components {
            for schema_name in components.schemas.keys() {
                if !visited.contains(schema_name) {
                    self.dfs_cycle_detection(
                        schema_name,
                        &mut visited,
                        &mut rec_stack,
                        &mut Vec::new(),
                        &mut cycles,
                    )?;
                }
            }
        }

        Ok(cycles)
    }

    fn dfs_cycle_detection(
        &self,
        schema_name: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) -> Result<()> {
        visited.insert(schema_name.to_string());
        rec_stack.insert(schema_name.to_string());
        path.push(schema_name.to_string());

        if let Some(deps) = self.dependency_graph.get(schema_name) {
            for dep in deps {
                if !visited.contains(dep) {
                    self.dfs_cycle_detection(dep, visited, rec_stack, path, cycles)?;
                } else if rec_stack.contains(dep) {
                    // Cycle detected
                    let cycle_start = path.iter().position(|x| x == dep).unwrap_or(0);
                    cycles.push(path[cycle_start..].to_vec());
                }
            }
        }

        rec_stack.remove(schema_name);
        path.pop();
        Ok(())
    }

    pub fn is_circular(&self, schema_name: &str) -> bool {
        self.circular_refs.contains(schema_name)
    }

    pub fn get_openapi(&self) -> &OpenAPI {
        &self.openapi
    }

    pub fn classify_schema(&self, schema: &Schema) -> SchemaType {
        match &schema.schema_kind {
            SchemaKind::Type(type_) => match type_ {
                Type::String(string_type) => {
                    if !string_type.enumeration.is_empty() {
                        let enum_values: Vec<String> = string_type
                            .enumeration
                            .iter()
                            .filter_map(|v| v.as_ref().cloned())
                            .collect();
                        if !enum_values.is_empty() {
                            return SchemaType::Enum {
                                values: enum_values,
                            };
                        }
                    }
                    SchemaType::Primitive(PrimitiveType::String)
                }
                Type::Number(_) => SchemaType::Primitive(PrimitiveType::Number),
                Type::Integer(_) => SchemaType::Primitive(PrimitiveType::Integer),
                Type::Boolean(_) => SchemaType::Primitive(PrimitiveType::Boolean),
                Type::Array(_) => SchemaType::Array {
                    item_type: Box::new(SchemaType::Primitive(PrimitiveType::Any)),
                },
                Type::Object(_) => SchemaType::Object {
                    properties: HashMap::new(),
                },
            },
            SchemaKind::OneOf { .. } => SchemaType::OneOf { variants: vec![] },
            SchemaKind::AllOf { .. } => SchemaType::AllOf { all: vec![] },
            SchemaKind::AnyOf { .. } => SchemaType::AnyOf { any: vec![] },
            SchemaKind::Any(_) => SchemaType::Primitive(PrimitiveType::Any),
            SchemaKind::Not { .. } => SchemaType::Primitive(PrimitiveType::Any),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SchemaType {
    Primitive(PrimitiveType),
    Array {
        item_type: Box<SchemaType>,
    },
    Object {
        properties: HashMap<String, SchemaType>,
    },
    Enum {
        values: Vec<String>,
    },
    Reference {
        ref_path: String,
    },
    OneOf {
        variants: Vec<SchemaType>,
    },
    AllOf {
        all: Vec<SchemaType>,
    },
    AnyOf {
        any: Vec<SchemaType>,
    },
    Nullable(Box<SchemaType>),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum PrimitiveType {
    String,
    Number,
    Integer,
    Boolean,
    Any,
}
