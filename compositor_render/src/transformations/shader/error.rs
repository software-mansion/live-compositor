use compositor_common::scene::ShaderParam;

use super::{USER_DEFINED_BUFFER_BINDING, USER_DEFINED_BUFFER_GROUP, VERTEX_ENTRYPOINT_NAME};

#[derive(Debug, thiserror::Error)]
pub enum ShaderValidationError {
    #[error("A global that should be declared in the shader is not declared: \n{0:#?}.")]
    GlobalNotFound(naga::GlobalVariable),

    #[error("A global in the shader has a wrong type. {0}")]
    GlobalBadType(#[source] TypeEquivalenceError),

    #[error("Could not find a vertex shader entrypoint. Expected \"fn {VERTEX_ENTRYPOINT_NAME}(input: VertexInput)\"")]
    VertexShaderNotFound,

    #[error("Wrong vertex shader argument amount: found {0}, expected 1.")]
    VertexShaderBadArgumentAmount(usize),

    // TODO: do we enforce type name from header?
    // #[error("The input type of the vertex shader has a name that cannot be found in the header.")]
    #[error("The input type of the vertex shader (\"{0}\") was not declared.")]
    VertexShaderBadInputTypeName(String),

    #[error("The vertex shader input has a wrong type. {0}")]
    VertexShaderBadInput(#[source] TypeEquivalenceError),

    #[error("User defined binding (group {USER_DEFINED_BUFFER_GROUP}, binding {USER_DEFINED_BUFFER_BINDING}) is not a uniform buffer. Is it defined as var<uniform>?")]
    UserBindingNotUniform,
}

#[derive(Debug, thiserror::Error)]
pub enum TypeEquivalenceError {
    #[error("Type names don't match: {0:?} != {1:?}.")]
    TypeNameMismatch(Option<String>, Option<String>),

    #[error(
        "Type internal structure doesn't match:\nExpected:\n{expected:#?}\n\nActual:\n{actual:#?}."
    )]
    TypeStructureMismatch {
        expected: naga::TypeInner,
        actual: naga::TypeInner,
    },

    #[error("Struct {struct_name} has an incorrect number of fields: expected: {expected_field_number}, found: {actual_field_number}")]
    StructFieldNumberMismatch {
        struct_name: String,
        expected_field_number: usize,
        actual_field_number: usize,
    },

    #[error("Structure mismatch found while checking the type of field {field_name} in struct {struct_name}:\n{error}")]
    StructFieldStructureMismatch {
        struct_name: String,
        field_name: String,
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
