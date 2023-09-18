use compositor_common::scene::shader::ShaderParam;

use super::{USER_DEFINED_BUFFER_BINDING, USER_DEFINED_BUFFER_GROUP, VERTEX_ENTRYPOINT_NAME};

// TODO: Add real URL.
const HEADER_DOCS_URL: &str = "https://docs.placeholder.com/shader_header";

#[derive(Debug, thiserror::Error)]
pub enum ShaderValidationError {
    #[error("A global \"{0}\" should be declared in the shader. Make sure to include the compositor header code inside your shader. Learn more: {HEADER_DOCS_URL}.")]
    GlobalNotFound(String),

    #[error("A global in the shader has a wrong type.")]
    GlobalBadType(#[source] TypeEquivalenceError, String),

    #[error("Could not find a vertex shader entrypoint. Expected \"fn {VERTEX_ENTRYPOINT_NAME}(input: VertexInput)\"")]
    VertexShaderNotFound,

    #[error("Wrong vertex shader argument amount: found {0}, expected 1.")]
    VertexShaderBadArgumentAmount(usize),

    #[error(
        "The input type of the vertex shader has to be named \"VertexInput\" (received: \"{0}\")."
    )]
    VertexShaderBadInputTypeName(String),

    #[error("The vertex shader input has a wrong type.")]
    VertexShaderBadInput(#[source] TypeEquivalenceError),

    #[error("User defined binding (group {USER_DEFINED_BUFFER_GROUP}, binding {USER_DEFINED_BUFFER_BINDING}) is not a uniform buffer. Is it defined as var<uniform>?")]
    UserBindingNotUniform,
}

#[derive(Debug, thiserror::Error)]
pub enum TypeEquivalenceError {
    #[error("Type names don't match: {0:?} != {1:?}.")]
    TypeNameMismatch(Option<String>, Option<String>),

    #[error("Type mismatch (expected: {expected}, actual: {actual}).")]
    TypeStructureMismatch { expected: String, actual: String },

    #[error("Struct {struct_name} has an incorrect number of fields: expected: {expected_field_number}, found: {actual_field_number}")]
    StructFieldNumberMismatch {
        struct_name: String,
        expected_field_number: usize,
        actual_field_number: usize,
    },

    #[error("Field \"{field_name}\" in struct {struct_name} has invalid type.")]
    StructFieldStructureMismatch {
        struct_name: String,
        field_name: String,
        #[source]
        error: Box<TypeEquivalenceError>,
    },

    #[error("Struct {struct_name} has mismatched field names: expected {expected_field_name}, found: {actual_field_name}.")]
    StructFieldNameMismatch {
        struct_name: String,
        expected_field_name: String,
        actual_field_name: String,
    },

    #[error("Field {field_name} in struct {struct_name} has an incorrect binding: expected {expected_binding:?}, found {actual_binding:?}.")]
    StructFieldBindingMismatch {
        struct_name: String,
        field_name: String,
        expected_binding: Option<naga::Binding>,
        actual_binding: Option<naga::Binding>,
    },

    #[error(transparent)]
    BadArraySize(#[from] ConstArraySizeEvalError),

    #[error("Sizes of an array don't match: {0:?} != {1:?}.")]
    ArraySizeMismatch(u64, u64),
}

#[derive(Debug, thiserror::Error)]
pub enum ConstArraySizeEvalError {
    #[error("Dynamic array size is not allowed.")]
    DynamicSize,

    #[error("A value below zero is not allowed as array size.")]
    NegativeLength(i64),

    #[error("Composite types are not allowed as array sizes (found {0:?}).")]
    CompositeType(naga::ConstantInner),

    #[error("Bools and floats are not allowed as array sizes (found {0:?}).")]
    WrongType(naga::ScalarValue),
}

#[derive(Debug, thiserror::Error)]
pub enum ParametersValidationError {
    #[error("No user-defined binding was found in the shader, even though parameters were provided in the request.")]
    NoBindingInShader,

    #[error("A type used in the shader cannot be provided at node registration: {0}.")]
    ForbiddenType(&'static str),

    #[error("An unsupported scalar kind.")]
    UnsupportedScalarKind(naga::ScalarKind, u8),

    #[error("Expected type {1:?}, got {0:?}.")]
    WrongType(ShaderParam, naga::TypeInner),

    #[error("A list of parameters is too long: expected a max of {expected}, got {provided}")]
    ListTooLong { expected: usize, provided: usize },

    #[error("Error while evaluating array size")]
    ArraySizeEvalError(#[from] ConstArraySizeEvalError),

    #[error("A struct has a wrong amount of fields: expected {expected}, got {provided}")]
    WrongShaderFieldsAmount { expected: usize, provided: usize },

    #[error("A field in the provided {struct_name} struct has a different name than in the expected struct: expected \"{expected}\", got \"{provided}\"")]
    WrongFieldName {
        struct_name: String,
        expected: String,
        provided: String,
    },

    #[error("Error while verifying field {struct_field} in struct {struct_name}:\n{error}")]
    WrongFieldType {
        struct_name: String,
        struct_field: String,
        error: Box<ParametersValidationError>,
    },

    #[error("Error while verifying array element at index {idx}:\n{error}")]
    WrongArrayElementType {
        idx: usize,
        error: Box<ParametersValidationError>,
    },

    #[error("Error while verifying vector element at index {idx}:\n{error}")]
    WrongVectorElementType {
        idx: usize,
        error: Box<ParametersValidationError>,
    },

    #[error("Error while verifying matrix row {idx}:\n{error}")]
    WrongMatrixRowType {
        idx: usize,
        error: Box<ParametersValidationError>,
    },
}

pub(crate) trait ShaderGlobalVariableExt {
    fn to_string(&self) -> String;
}

impl ShaderGlobalVariableExt for naga::GlobalVariable {
    fn to_string(&self) -> String {
        let group_and_binding = match self.binding {
            Some(naga::ResourceBinding { group, binding }) => {
                format!("@group({}) @binding({}) ", group, binding)
            }
            None => "".to_string(),
        };
        let space = match self.space {
            naga::AddressSpace::Function => "<function>".to_string(),
            naga::AddressSpace::Private => "<private>".to_string(),
            naga::AddressSpace::WorkGroup => "<workgroup>".to_string(),
            naga::AddressSpace::Uniform => "<uniform>".to_string(),
            naga::AddressSpace::Storage { access } => {
                let access = match access.bits() {
                    1 => "read",
                    2 => "write",
                    3 => "read_write",
                    _ => "",
                };
                format!("<storage, {}>", access)
            }
            naga::AddressSpace::Handle => "".to_string(),
            naga::AddressSpace::PushConstant => "<push_constant>".to_string(),
        };
        let name = self.name.clone().unwrap_or("value".to_string());
        format!("{group_and_binding}var{space} {name}")
    }
}
