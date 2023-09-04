use compositor_common::scene::ShaderParam;
use naga::{ArraySize, ConstantInner, Handle, Module, ScalarKind, ShaderStage, Type, VectorSize};

use super::VERTEX_ENTRYPOINT_NAME;

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
}

pub fn validate_contains_header(
    header: &naga::Module,
    shader: &naga::Module,
) -> Result<(), ShaderValidationError> {
    validate_globals(header, shader)?;
    validate_vertex_input(header, shader)?;
    Ok(())
}

fn validate_globals(
    header: &naga::Module,
    shader: &naga::Module,
) -> Result<(), ShaderValidationError> {
    for (_, global) in header.global_variables.iter() {
        let (_, global_in_shader) = shader
            .global_variables
            .iter()
            .find(|(_, s_global)| {
                s_global.space == global.space && s_global.binding == global.binding
            })
            .ok_or_else(|| ShaderValidationError::GlobalNotFound(global.clone()))?;

        validate_type_equivalent(global.ty, header, global_in_shader.ty, shader)
            .map_err(ShaderValidationError::GlobalBadType)?;
    }

    Ok(())
}

fn validate_vertex_input(
    header: &naga::Module,
    shader: &naga::Module,
) -> Result<(), ShaderValidationError> {
    let vertex = shader
        .entry_points
        .iter()
        .find(|entry_point| {
            entry_point.name == super::VERTEX_ENTRYPOINT_NAME
                && entry_point.stage == ShaderStage::Vertex
        })
        .ok_or(ShaderValidationError::VertexShaderNotFound)?;

    if vertex.function.arguments.len() != 1 {
        return Err(ShaderValidationError::VertexShaderBadArgumentAmount(
            vertex.function.arguments.len(),
        ));
    }

    let vertex_input = vertex.function.arguments[0].ty;
    let vertex_input_type = &shader.types[vertex_input];

    let (header_vertex_input, _) = header
        .types
        .iter()
        .find(|(_, ty)| ty.name == vertex_input_type.name)
        .ok_or_else(|| {
            ShaderValidationError::VertexShaderBadInputTypeName(
                vertex_input_type
                    .name
                    .clone()
                    .unwrap_or("<unknown>".to_string()),
            )
        })?;

    validate_type_equivalent(header_vertex_input, header, vertex_input, shader)
        .map_err(ShaderValidationError::VertexShaderBadInput)?;

    Ok(())
}

// TODO: improve these errors, they're terrible
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

    #[error("Error while evaluating array size")]
    BadArraySize(#[from] ConstArraySizeEvalError),

    #[error("Sizes of an array don't match: {0:?} != {1:?}.")]
    ArraySizeMismatch(u64, u64),
}

fn validate_type_equivalent(
    ty1: Handle<Type>,
    mod1: &Module,
    ty2: Handle<Type>,
    mod2: &Module,
) -> Result<(), TypeEquivalenceError> {
    let type1 = &mod1.types[ty1];
    let type2 = &mod2.types[ty2];

    if type1.name != type2.name {
        return Err(TypeEquivalenceError::TypeNameMismatch(
            type1.name.clone(),
            type2.name.clone(),
        ));
    }

    let ti1 = match type1.inner.canonical_form(&mod1.types) {
        Some(t) => t,
        None => type1.inner.clone(),
    };
    let ti2 = match type2.inner.canonical_form(&mod2.types) {
        Some(t) => t,
        None => type2.inner.clone(),
    };

    match ti1 {
        naga::TypeInner::Scalar { .. }
        | naga::TypeInner::Vector { .. }
        | naga::TypeInner::Matrix { .. }
        | naga::TypeInner::Atomic { .. }
        | naga::TypeInner::Image { .. }
        | naga::TypeInner::Sampler { .. }
        | naga::TypeInner::AccelerationStructure
        | naga::TypeInner::RayQuery
        | naga::TypeInner::ValuePointer { .. } => {
            if ti1 != ti2 {
                return Err(TypeEquivalenceError::TypeStructureMismatch {
                    expected: type1.inner.clone(),
                    actual: type2.inner.clone(),
                });
            }
        }

        naga::TypeInner::Array {
            base: base1,
            size: size1,
            stride: stride1,
        } => {
            let naga::TypeInner::Array {
                base: base2,
                size: size2,
                stride: stride2,
            } = ti2
            else {
                return Err(TypeEquivalenceError::TypeStructureMismatch {
                    expected: ti1,
                    actual: ti2,
                });
            };

            if stride1 != stride2 {
                return Err(TypeEquivalenceError::TypeStructureMismatch {
                    expected: ti1,
                    actual: ti2,
                });
            }

            validate_array_size_equivalent(size1, mod1, size2, mod2)?;
            return validate_type_equivalent(base1, mod1, base2, mod2);
        }

        naga::TypeInner::BindingArray {
            base: base1,
            size: size1,
        } => {
            let naga::TypeInner::BindingArray {
                base: base2,
                size: size2,
            } = ti2
            else {
                return Err(TypeEquivalenceError::TypeStructureMismatch {
                    expected: ti1,
                    actual: ti2,
                });
            };

            validate_array_size_equivalent(size1, mod1, size2, mod2)?;
            return validate_type_equivalent(base1, mod1, base2, mod2);
        }

        naga::TypeInner::Struct {
            members: ref members1,
            span: span1,
        } => {
            let naga::TypeInner::Struct {
                members: ref members2,
                span: span2,
            } = ti2
            else {
                return Err(TypeEquivalenceError::TypeStructureMismatch {
                    expected: ti1.clone(),
                    actual: ti2.clone(),
                });
            };

            if span1 != span2 || members1.len() != members2.len() {
                return Err(TypeEquivalenceError::TypeStructureMismatch {
                    expected: ti1.clone(),
                    actual: ti2.clone(),
                });
            }

            for (m1, m2) in members1.iter().zip(members2.iter()) {
                if m1.binding != m2.binding || m1.name != m2.name || m1.offset != m2.offset {
                    return Err(TypeEquivalenceError::TypeStructureMismatch {
                        expected: ti1,
                        actual: ti2,
                    });
                }

                validate_type_equivalent(m1.ty, mod1, m2.ty, mod2)?;
            }
        }

        naga::TypeInner::Pointer { .. } => {
            panic!("this should never happen bc of canonicalization")
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ConstArraySizeEvalError {
    #[error("Dynamic array size is not allowed.")]
    DynamicSize,

    #[error("A value below zero is not allowed as array size.")]
    BelowZero(i64),

    #[error("Composite types are not allowed as array sizes (found {0:?}).")]
    CompositeType(ConstantInner),

    #[error("Bools and floats are not allowed as array sizes (found {0:?}).")]
    WrongType(naga::ScalarValue),
}

fn eval_array_size(size: ArraySize, module: &naga::Module) -> Result<u64, ConstArraySizeEvalError> {
    match size {
        ArraySize::Constant(c) => {
            let c = &module.constants[c];

            // TODO: what do we do with c1.specialization? It doesn't occur in WGSL, but it can occur in vulkan shaders, which we might want to support later.
            // There are also plans of adding them to WGSL

            match c.inner {
                ConstantInner::Scalar { value, .. } => match value {
                    naga::ScalarValue::Uint(v) => Ok(v),

                    naga::ScalarValue::Sint(v) => {
                        if v < 0 {
                            Err(ConstArraySizeEvalError::BelowZero(v))
                        } else {
                            Ok(v as u64)
                        }
                    }

                    naga::ScalarValue::Float(_) | naga::ScalarValue::Bool(_) => {
                        Err(ConstArraySizeEvalError::WrongType(value))
                    }
                },
                ConstantInner::Composite { .. } => {
                    Err(ConstArraySizeEvalError::CompositeType(c.inner.clone()))
                }
            }
        }
        ArraySize::Dynamic => Err(ConstArraySizeEvalError::DynamicSize),
    }
}

fn validate_array_size_equivalent(
    size1: ArraySize,
    mod1: &Module,
    size2: ArraySize,
    mod2: &Module,
) -> Result<(), TypeEquivalenceError> {
    let size1 = eval_array_size(size1, mod1)?;
    let size2 = eval_array_size(size2, mod2)?;

    if size1 != size2 {
        return Err(TypeEquivalenceError::ArraySizeMismatch(size1, size2));
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ParametersValidationError {
    #[error("No user-defined binding was found in the shader, even though parameters were provided in the request.")]
    NoBindingInShader,

    #[error("A type used in the shader cannot be provided at node registration: {0}.")]
    ForbiddenType(&'static str),

    #[error("An unsupported scalar kind.")]
    UnsupportedScalarKind(ScalarKind, u8),

    #[error("Expected type {1:?}, got {0:?}.")]
    WrongType(ShaderParam, naga::TypeInner),

    #[error("A list type has wrong length: expected {expected}, got {provided}")]
    WrongListLength { expected: usize, provided: usize },

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

pub fn validate_params(
    params: &ShaderParam,
    ty: Handle<Type>,
    module: &naga::Module,
) -> Result<(), ParametersValidationError> {
    let ty = &module.types[ty];

    // TODO: tests, make examples run

    match &ty.inner {
        naga::TypeInner::Scalar { kind, width } => validate_scalar(params, *kind, *width),

        naga::TypeInner::Vector { size, kind, width } => {
            validate_vector(params, *size, *kind, *width)
        }

        naga::TypeInner::Matrix {
            columns,
            rows,
            width,
        } => validate_matrix(params, *columns, *rows, *width),

        naga::TypeInner::Array { base, size, stride } => {
            validate_array(params, *base, *size, *stride, module)
        }

        naga::TypeInner::Struct { members, span } => {
            validate_struct(params, ty.name.as_ref().unwrap(), members, *span, module)
        }

        naga::TypeInner::Pointer { .. }
        | naga::TypeInner::ValuePointer { .. }
        | naga::TypeInner::Atomic { .. }
        | naga::TypeInner::Image { .. }
        | naga::TypeInner::Sampler { .. }
        | naga::TypeInner::AccelerationStructure
        | naga::TypeInner::RayQuery
        | naga::TypeInner::BindingArray { .. } => Err(ParametersValidationError::ForbiddenType(
            ty.inner.type_name(),
        )),
    }
}

fn validate_struct(
    params: &ShaderParam,
    struct_name_in_shader: &str,
    struct_members_in_shader: &[naga::StructMember],
    span: u32,
    module: &naga::Module,
) -> Result<(), ParametersValidationError> {
    match params {
        ShaderParam::Struct(param_fields) => {
            if struct_members_in_shader.len() != param_fields.len() {
                return Err(ParametersValidationError::WrongShaderFieldsAmount {
                    expected: struct_members_in_shader.len(),
                    provided: param_fields.len(),
                });
            }

            for (shader_member, param_field) in
                struct_members_in_shader.iter().zip(param_fields.iter())
            {
                if shader_member.name.as_ref().unwrap() != &param_field.field_name {
                    return Err(ParametersValidationError::WrongFieldName {
                        struct_name: struct_name_in_shader.into(),
                        expected: shader_member.name.as_ref().unwrap().clone(),
                        provided: param_field.field_name.clone(),
                    });
                }

                validate_params(&param_field.value, shader_member.ty, module).map_err(|err| {
                    ParametersValidationError::WrongFieldType {
                        struct_name: struct_name_in_shader.into(),
                        struct_field: param_field.field_name.clone(),
                        error: Box::new(err),
                    }
                })?
            }

            Ok(())
        }

        _ => Err(ParametersValidationError::WrongType(
            params.clone(),
            naga::TypeInner::Struct {
                members: struct_members_in_shader.to_owned(),
                span,
            },
        )),
    }
}

fn validate_array(
    params: &ShaderParam,
    base: Handle<Type>,
    size: ArraySize,
    stride: u32,
    module: &naga::Module,
) -> Result<(), ParametersValidationError> {
    // ignoring the `stride`, it probably doesn't matter if the types are correct
    let evaluated_size = eval_array_size(size, module)?;

    match params {
        ShaderParam::List(list) => {
            if list.len() != evaluated_size as usize {
                return Err(ParametersValidationError::WrongListLength {
                    expected: evaluated_size as usize,
                    provided: list.len(),
                });
            }

            for (idx, param) in list.iter().enumerate() {
                validate_params(param, base, module).map_err(|err| {
                    ParametersValidationError::WrongArrayElementType {
                        idx,
                        error: Box::new(err),
                    }
                })?
            }

            Ok(())
        }

        _ => Err(ParametersValidationError::WrongType(
            params.clone(),
            naga::TypeInner::Array { base, size, stride },
        )),
    }
}

fn validate_matrix(
    params: &ShaderParam,
    columns: VectorSize,
    rows: VectorSize,
    width: u8,
) -> Result<(), ParametersValidationError> {
    match params {
        ShaderParam::List(rows_list) => {
            if rows_list.len() != rows as usize {
                return Err(ParametersValidationError::WrongListLength {
                    expected: rows as usize,
                    provided: rows_list.len(),
                });
            }

            for (idx, row) in rows_list.iter().enumerate() {
                validate_vector(row, columns, ScalarKind::Float, width).map_err(|err| {
                    ParametersValidationError::WrongMatrixRowType {
                        idx,
                        error: Box::new(err),
                    }
                })?
            }

            Ok(())
        }

        _ => Err(ParametersValidationError::WrongType(
            params.clone(),
            naga::TypeInner::Matrix {
                columns,
                rows,
                width,
            },
        )),
    }
}

fn validate_vector(
    params: &ShaderParam,
    size: VectorSize,
    kind: ScalarKind,
    width: u8,
) -> Result<(), ParametersValidationError> {
    match params {
        ShaderParam::List(list) => {
            if list.len() != size as usize {
                return Err(ParametersValidationError::WrongListLength {
                    expected: size as usize,
                    provided: list.len(),
                });
            }

            for (idx, v) in list.iter().enumerate() {
                validate_scalar(v, kind, width).map_err(|err| {
                    ParametersValidationError::WrongVectorElementType {
                        idx,
                        error: Box::new(err),
                    }
                })?
            }

            Ok(())
        }

        _ => Err(ParametersValidationError::WrongType(
            params.clone(),
            naga::TypeInner::Vector { size, kind, width },
        )),
    }
}

fn validate_scalar(
    params: &ShaderParam,
    kind: ScalarKind,
    width: u8,
) -> Result<(), ParametersValidationError> {
    match (kind, width) {
        (ScalarKind::Float, 4) => match params {
            ShaderParam::F32(_) => Ok(()),
            _ => Err(ParametersValidationError::WrongType(
                params.clone(),
                naga::TypeInner::Scalar { kind, width },
            )),
        },

        (ScalarKind::Uint, 4) => match params {
            ShaderParam::U32(_) => Ok(()),
            _ => Err(ParametersValidationError::WrongType(
                params.clone(),
                naga::TypeInner::Scalar { kind, width },
            )),
        },

        (ScalarKind::Sint, 4) => match params {
            ShaderParam::I32(_) => Ok(()),
            _ => Err(ParametersValidationError::WrongType(
                params.clone(),
                naga::TypeInner::Scalar { kind, width },
            )),
        },

        _ => Err(ParametersValidationError::UnsupportedScalarKind(
            kind, width,
        )),
    }
}

trait TypeInnerExt {
    fn type_name(&self) -> &'static str;
}

impl TypeInnerExt for naga::TypeInner {
    fn type_name(&self) -> &'static str {
        match self {
            naga::TypeInner::Scalar { .. } => "scalar",
            naga::TypeInner::Vector { .. } => "vector",
            naga::TypeInner::Matrix { .. } => "matrix",
            naga::TypeInner::Atomic { .. } => "atomic",
            naga::TypeInner::Pointer { .. } => "pointer",
            naga::TypeInner::ValuePointer { .. } => "value pointer",
            naga::TypeInner::Array { .. } => "array",
            naga::TypeInner::Struct { .. } => "struct",
            naga::TypeInner::Image { .. } => "texture",
            naga::TypeInner::Sampler { .. } => "sampler",
            naga::TypeInner::AccelerationStructure => "acceleration structure",
            naga::TypeInner::RayQuery => "ray query",
            naga::TypeInner::BindingArray { .. } => "binding array",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_len() {
        let s1 = r#"
        var a: array<i32, 16>;
        "#;

        let s2 = r#"
        var a: array<i32, 17>;
        "#;

        let s1 = naga::front::wgsl::parse_str(s1).unwrap();
        let s2 = naga::front::wgsl::parse_str(s2).unwrap();

        assert!(validate_contains_header(&s1, &s2).is_err());
    }

    #[test]
    fn binding() {
        let s1 = r#"
        @group(0) @binding(0) var a: i32;
        "#;

        let s2 = r#"
        @group(0) @binding(1) var a: i32;
        "#;

        let s1 = naga::front::wgsl::parse_str(s1).unwrap();
        let s2 = naga::front::wgsl::parse_str(s2).unwrap();

        assert!(validate_contains_header(&s1, &s2).is_err());
    }

    #[test]
    fn vertex_input() {
        let s1 = r#"
        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(1) tex_coords: vec2<f32>,
        }
        "#;

        let s2 = r#"
        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(1) tex_coords: vec2<u32>,
        }

        @vertex
        fn vs_main(in: VertexInput) -> @builtin(position) vec4<f32> {
            return vec4(0);
        }
        "#;

        let s1 = naga::front::wgsl::parse_str(s1).unwrap();
        let s2 = naga::front::wgsl::parse_str(s2).unwrap();

        assert!(validate_contains_header(&s1, &s2).is_err());
    }

    #[test]
    fn vertex_input_locations() {
        let s1 = r#"
        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(1) tex_coords: vec2<f32>,
        }
        "#;

        let s2 = r#"
        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(2) tex_coords: vec2<f32>,
        }

        @vertex
        fn vs_main(in: VertexInput) -> @builtin(position) vec4<f32> {
            return vec4(0);
        }
        "#;

        let s1 = naga::front::wgsl::parse_str(s1).unwrap();
        let s2 = naga::front::wgsl::parse_str(s2).unwrap();

        assert!(validate_contains_header(&s1, &s2).is_err());
    }
}
