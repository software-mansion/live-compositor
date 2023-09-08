use compositor_common::scene::shader::ShaderParam;
use naga::{ArraySize, ConstantInner, Handle, Module, ScalarKind, ShaderStage, Type, VectorSize};

use super::{
    error::{
        ConstArraySizeEvalError, ParametersValidationError, ShaderGlobalVariableExt,
        ShaderValidationError, TypeEquivalenceError,
    },
    USER_DEFINED_BUFFER_BINDING, USER_DEFINED_BUFFER_GROUP,
};

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
            .ok_or_else(|| ShaderValidationError::GlobalNotFound(global.to_string()))?;

        validate_type_equivalent(global.ty, header, global_in_shader.ty, shader).map_err(
            |err| {
                ShaderValidationError::GlobalBadType(
                    err,
                    global.name.clone().unwrap_or("value".to_string()),
                )
            },
        )?;
    }

    // validate user-defined buffer is a uniform
    shader
        .global_variables
        .iter()
        .find(|(_, global)| {
            global.binding.is_some()
                && global.binding.as_ref().unwrap().group == USER_DEFINED_BUFFER_GROUP
                && global.binding.as_ref().unwrap().binding == USER_DEFINED_BUFFER_BINDING
                && global.space == naga::AddressSpace::Uniform
        })
        .map_or(Ok(()), |(_, global)| match global.space {
            naga::AddressSpace::Uniform => Ok(()),
            _ => Err(ShaderValidationError::UserBindingNotUniform),
        })?;

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
                    expected: type1.inner.to_string(mod1),
                    actual: type2.inner.to_string(mod2),
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
                    expected: ti1.to_string(mod1),
                    actual: ti2.to_string(mod2),
                });
            };

            if stride1 != stride2 {
                return Err(TypeEquivalenceError::TypeStructureMismatch {
                    expected: ti1.to_string(mod1),
                    actual: ti2.to_string(mod2),
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
                    expected: ti1.to_string(mod1),
                    actual: ti2.to_string(mod2),
                });
            };

            validate_array_size_equivalent(size1, mod1, size2, mod2)?;
            return validate_type_equivalent(base1, mod1, base2, mod2);
        }

        naga::TypeInner::Struct {
            members: ref members1,
            ..
        } => {
            let naga::TypeInner::Struct {
                members: ref members2,
                ..
            } = ti2
            else {
                return Err(TypeEquivalenceError::TypeStructureMismatch {
                    expected: ti1.to_string(mod1),
                    actual: ti2.to_string(mod2),
                });
            };

            // skipped checking if ti1.span == ti2.span
            // if all fields have the same types, how can the spans be different?

            if members1.len() != members2.len() {
                return Err(TypeEquivalenceError::StructFieldNumberMismatch {
                    struct_name: type1.name.as_ref().unwrap().clone(),
                    expected_field_number: members1.len(),
                    actual_field_number: members2.len(),
                });
            }

            for (m1, m2) in members1.iter().zip(members2.iter()) {
                if m1.name != m2.name {
                    return Err(TypeEquivalenceError::StructFieldNameMismatch {
                        struct_name: type1.name.as_ref().unwrap().clone(),
                        expected_field_name: m1.name.as_ref().unwrap().clone(),
                        actual_field_name: m2.name.as_ref().unwrap().clone(),
                    });
                }

                // skipped checking if m1.offset == m2.offset
                // if all fields have the same types, how can the offsets be different?

                if m1.binding != m2.binding {
                    return Err(TypeEquivalenceError::StructFieldBindingMismatch {
                        struct_name: type1.name.as_ref().unwrap().clone(),
                        field_name: m1.name.as_ref().unwrap().clone(),
                        expected_binding: m1.binding.clone(),
                        actual_binding: m2.binding.clone(),
                    });
                }

                validate_type_equivalent(m1.ty, mod1, m2.ty, mod2).map_err(|err| {
                    TypeEquivalenceError::StructFieldStructureMismatch {
                        struct_name: type1.name.as_ref().unwrap().clone(),
                        field_name: m1.name.as_ref().unwrap().clone(),
                        error: Box::new(err),
                    }
                })?;
            }
        }

        naga::TypeInner::Pointer { .. } => {
            panic!("this should never happen bc of canonicalization")
        }
    }

    Ok(())
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
                            Err(ConstArraySizeEvalError::NegativeLength(v))
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

pub fn validate_params(
    params: &ShaderParam,
    ty: Handle<Type>,
    module: &naga::Module,
) -> Result<(), ParametersValidationError> {
    let ty = &module.types[ty];

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
            if list.len() > evaluated_size as usize {
                return Err(ParametersValidationError::ListTooLong {
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
                return Err(ParametersValidationError::ListTooLong {
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
                return Err(ParametersValidationError::ListTooLong {
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
    fn to_string(&self, module: &naga::Module) -> String;
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
    fn to_string(&self, module: &naga::Module) -> String {
        match self {
            naga::TypeInner::Scalar { kind, width } => kind.to_string(*width),
            naga::TypeInner::Vector { size, kind, width } => {
                format!("vec{}<{}>", *size as u8, kind.to_string(*width))
            }
            naga::TypeInner::Matrix { .. } => "matrix".to_string(),
            naga::TypeInner::Atomic { .. } => "atomic".to_string(),
            naga::TypeInner::Pointer { .. } => "pointer".to_string(),
            naga::TypeInner::ValuePointer { .. } => "value pointer".to_string(),
            naga::TypeInner::Array { .. } => "array".to_string(),
            naga::TypeInner::Struct { .. } => "struct".to_string(),
            naga::TypeInner::Image {
                dim,
                arrayed,
                class,
            } => match (dim, arrayed, class) {
                (naga::ImageDimension::D1, false, naga::ImageClass::Sampled { kind, .. }) => {
                    format!("texture_1d<{}>", kind.to_string(4))
                }
                (naga::ImageDimension::D2, false, naga::ImageClass::Sampled { kind, .. }) => {
                    format!("texture_2d<{}>", kind.to_string(4))
                }
                (naga::ImageDimension::D2, true, naga::ImageClass::Sampled { kind, .. }) => {
                    format!("texture_2d_array<{}>", kind.to_string(4))
                }
                (naga::ImageDimension::D3, false, naga::ImageClass::Sampled { kind, .. }) => {
                    format!("texture_3d<{}>", kind.to_string(4))
                }
                (naga::ImageDimension::Cube, false, naga::ImageClass::Sampled { kind, .. }) => {
                    format!("texture_cube<{}>", kind.to_string(4))
                }
                (naga::ImageDimension::Cube, true, naga::ImageClass::Sampled { kind, .. }) => {
                    format!("texture_cube_array<{}>", kind.to_string(4))
                }
                _ => format!("{:?}", self),
            },
            naga::TypeInner::Sampler { .. } => "sampler".to_string(),
            naga::TypeInner::AccelerationStructure => "acceleration structure".to_string(),
            naga::TypeInner::RayQuery => "ray query".to_string(),
            naga::TypeInner::BindingArray { base, size } => {
                let size: Option<&naga::Constant> = match size {
                    ArraySize::Constant(size) => Some(&module.constants[*size]),
                    ArraySize::Dynamic => None, // TODO: not sure how to handle this
                };
                let base: &naga::Type = &module.types[*base];
                format!(
                    "binding_array<{}, {}>",
                    base.inner.to_string(module),
                    size.map(|t| t.inner.to_string(module))
                        .unwrap_or("_".to_string())
                )
            }
        }
    }
}

trait ConstantInnerExt {
    fn to_string(&self, module: &naga::Module) -> String;
}

impl ConstantInnerExt for naga::ConstantInner {
    fn to_string(&self, _module: &naga::Module) -> String {
        match self {
            ConstantInner::Scalar { value, .. } => match value {
                naga::ScalarValue::Sint(v) => format!("{}", v),
                naga::ScalarValue::Uint(v) => format!("{}", v),
                naga::ScalarValue::Float(v) => format!("{}", v),
                naga::ScalarValue::Bool(true) => "true".to_string(),
                naga::ScalarValue::Bool(false) => "false".to_string(),
            },
            ConstantInner::Composite { .. } => format!("{:?}", self),
        }
    }
}

trait ScalarKindExt {
    fn to_string(&self, width: u8) -> String;
}

impl ScalarKindExt for naga::ScalarKind {
    fn to_string(&self, width: u8) -> String {
        match self {
            ScalarKind::Sint => format!("i{}", width * 8),
            ScalarKind::Uint => format!("u{}", width * 8),
            ScalarKind::Float => format!("f{}", width * 8),
            ScalarKind::Bool => "bool".to_string(),
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
