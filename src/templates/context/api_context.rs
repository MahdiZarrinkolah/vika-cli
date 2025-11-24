use serde::Serialize;

/// Context for API client function generation.
#[derive(Debug, Clone, Serialize)]
pub struct ApiContext {
    pub function_name: String,
    pub operation_id: Option<String>,
    pub http_method: String,
    pub path: String,
    pub path_params: Vec<Parameter>,
    pub query_params: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    pub responses: Vec<Response>,
    pub type_imports: String,
    pub http_import: String,
    pub return_type: String,
    pub function_body: String,
    pub module_name: String,
    pub params: String,
    pub description: String,
    /// Spec name (for multi-spec mode)
    pub spec_name: Option<String>,
}

/// Parameter information for API functions.
#[derive(Debug, Clone, Serialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub optional: bool,
    pub description: Option<String>,
}

/// Request body information.
#[derive(Debug, Clone, Serialize)]
pub struct RequestBody {
    pub type_name: String,
    pub description: Option<String>,
}

/// Response information.
#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub status_code: u16,
    pub body_type: String,
}

impl ApiContext {
    /// Create a new ApiContext.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        function_name: String,
        operation_id: Option<String>,
        http_method: String,
        path: String,
        path_params: Vec<Parameter>,
        query_params: Vec<Parameter>,
        request_body: Option<RequestBody>,
        responses: Vec<Response>,
        type_imports: String,
        http_import: String,
        return_type: String,
        function_body: String,
        module_name: String,
        params: String,
        description: String,
        spec_name: Option<String>,
    ) -> Self {
        Self {
            function_name,
            operation_id,
            http_method,
            path,
            path_params,
            query_params,
            request_body,
            responses,
            type_imports,
            http_import,
            return_type,
            function_body,
            module_name,
            params,
            description,
            spec_name,
        }
    }
}

impl Parameter {
    /// Create a new Parameter.
    pub fn new(
        name: String,
        param_type: String,
        optional: bool,
        description: Option<String>,
    ) -> Self {
        Self {
            name,
            param_type,
            optional,
            description,
        }
    }
}

impl RequestBody {
    /// Create a new RequestBody.
    pub fn new(type_name: String, description: Option<String>) -> Self {
        Self {
            type_name,
            description,
        }
    }
}

impl Response {
    /// Create a new Response.
    pub fn new(status_code: u16, body_type: String) -> Self {
        Self {
            status_code,
            body_type,
        }
    }
}
