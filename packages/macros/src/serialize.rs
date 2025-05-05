use std::collections::HashMap;

use proc_macro_error::{abort, diagnostic, emit_error, Level};
use serde::Serialize;
use syn::{spanned::Spanned, FnArg, Pat, PatType, Type};

use crate::{LinkCall, LinkFunc, LinkItemArg, LinkItemReturnType, WrapperType};

#[derive(Serialize)]
pub struct SdkModule {
    name: String,
    items: Vec<SdkItem>,
    enums: Vec<SdkEnum>,
}

impl SdkModule {
    pub fn new(data: &LinkCall) -> Option<Self> {
        Some(Self {
            name: data.module_name.value(),
            items: data.module_items.iter()
                .map(SdkItem::new)
                .collect::<Vec<_>>()
                .into_iter()
                .collect::<Option<Vec<_>>>()?,
            enums: vec![],
        })
    }
}

#[derive(Serialize)]
pub struct SdkItem {
    name: String,
    params: Vec<SdkItemParam>,
    returns: Option<SdkType>,
}

impl SdkItem {
    pub fn new(item: &LinkFunc) -> Option<Self> {
        let mut params = item.inputs.iter()
            .map(SdkItemParam::new)
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Option<Vec<_>>>()?;

        if item.printfness.is_some() {
            params.push(SdkItemParam { name: "string".to_string(), r#type: SdkType::StringPtr });
        }

        Some(Self {
            name: item.ident.to_string().replace("r#", ""),
            params,
            returns: SdkType::from_return_type(&item.output)?,
        })
    }
}

#[derive(Serialize)]
pub struct SdkItemParam {
    name: String,
    r#type: SdkType,
}

impl SdkItemParam {
    pub fn new(arg: &LinkItemArg) -> Option<Self> {
        let fn_arg = if let FnArg::Typed(fn_arg) = &arg.fn_arg {
            fn_arg
        } else {
            emit_error!(arg.fn_arg, "`self` parameters aren't supported");
            return None;
        };

        let name = if let Pat::Ident(ident) = &*fn_arg.pat {
            ident
        } else {
            emit_error!(fn_arg.pat, "Argument destructuring is not supported");
            return None;
        };

        Some(Self {
            name: name.ident.to_string().replace("r#", ""),
            r#type: if let WrapperType::Convert(wrapper) = &arg.wrapper {
                SdkType::Named { name: get_type_name(wrapper)? }
            } else {
                SdkType::from_type(&fn_arg.ty)?
            },
        })
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum SdkType {
    Bool,
    Int,
    Long,
    Float,
    Double,
    StringPtr,
    Named { name: String }
}

impl SdkType {
    pub fn from_type(ty: &Type) -> Option<Self> {
        let type_name = get_type_name(ty)?;

        Some(match &*type_name {
            "bool" => Self::Bool,
            "i32" | "u32" => Self::Int,
            "i64" | "u64" => Self::Long,
            "f32" | "c_float" => Self::Float,
            "f64" | "c_double" => Self::Double,
            "CStr" => Self::StringPtr,
            _ => {
                emit_error!(ty, "This type is not supported");
                return None;
            }
        })
    }

    pub fn from_return_type(output: &LinkItemReturnType) -> Option<Option<Self>> {
        Some(match output {
            LinkItemReturnType::Default => None,
            LinkItemReturnType::Type { return_type, wrapper, .. } => {
                if let WrapperType::Convert(wrapper) = wrapper {
                    Some(Self::Named { name: get_type_name(wrapper)? })
                } else {
                    Self::from_type(return_type)
                }
            }
        })
    }
}

#[derive(Serialize)]
pub struct SdkEnum {
    name: String,
    underlying_type: SdkType,
    variants: HashMap<String, f64>,
}

fn get_type_name(ty: &Type) -> Option<String> {
    let path = if let Type::Path(path) = ty {
        path
    } else {
        emit_error!(ty, "This type is not allowed because it can't be represented in the API description format");
        return None;
    };

    let name = path.path.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");

    Some(name)
}