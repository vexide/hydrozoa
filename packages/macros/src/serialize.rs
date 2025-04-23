use std::collections::HashMap;

use proc_macro_error::{abort, diagnostic, emit_error, Level};
use serde::Serialize;
use syn::{spanned::Spanned, FnArg, Pat, PatType, Type};

use crate::{LinkCall, LinkItem, LinkItemArg, LinkItemReturnType};

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
    pub fn new(item: &LinkItem) -> Option<Self> {
        Some(Self {
            name: item.ident.to_string().replace("r#", ""),
            params: item.inputs.iter()
                .map(SdkItemParam::new)
                .collect::<Vec<_>>()
                .into_iter()
                .collect::<Option<Vec<_>>>()?,
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
            r#type: SdkType::new(&fn_arg.ty)?,
        })
    }
}

#[derive(Serialize)]
enum SdkType {
    Bool,
    Int,
    Long,
    Float,
    Double,
    Enum(String)
}

impl SdkType {
    pub fn new(ty: &Type) -> Option<Self> {
        let path = if let Type::Path(path) = ty {
            path
        } else {
            emit_error!(ty, "This type is not allowed because it can't be represented in the API description format");
            return None;
        };

        let last_segment = path.path.segments.last().unwrap().ident.to_string();

        Some(match &*last_segment {
            "bool" => Self::Bool,
            "i32" | "u32" => Self::Int,
            "i64" | "u64" => Self::Long,
            "f32" | "c_float" => Self::Float,
            "f64" | "c_double" => Self::Double,
            _ => {
                emit_error!(path, "This type is not supported");
                return None;
            }
        })
    }

    pub fn from_return_type(output: &LinkItemReturnType) -> Option<Option<Self>> {
        match output {
            LinkItemReturnType::Default => Some(None),
            LinkItemReturnType::Type { return_type, .. } => Some(Self::new(return_type)),
        }
    }
}

#[derive(Serialize)]
pub struct SdkEnum {
    name: String,
    underlying_type: SdkType,
    variants: HashMap<String, f64>,
}