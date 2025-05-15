use std::collections::HashMap;

use proc_macro_error::{Level, abort, diagnostic, emit_error};
use serde::Serialize;
use syn::{FnArg, Pat, PatType, Type, ext::IdentExt, spanned::Spanned};

use crate::{LinkCall, LinkEnum, LinkFunc, LinkItem, LinkItemArg, LinkItemReturnType, WrapperType};

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
            items: data
                .module_items
                .iter()
                .filter_map(|item| match item {
                    LinkItem::Func(func) => Some(func),
                    _ => None,
                })
                .map(SdkItem::new)
                .collect::<Option<Vec<_>>>()?,
            enums: data
                .module_items
                .iter()
                .filter_map(|item| match item {
                    LinkItem::Enum(link_enum) => Some(link_enum),
                    _ => None,
                })
                .map(SdkEnum::new)
                .collect::<Option<Vec<_>>>()?,
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
        let mut params = item
            .inputs
            .iter()
            .map(SdkItemParam::new)
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Option<Vec<_>>>()?;

        if item.printfness.is_some() {
            params.push(SdkItemParam {
                name: "string".to_string(),
                r#type: SdkType::StringPtr,
            });
        }

        Some(Self {
            name: item.ident.unraw().to_string(),
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
            name: name.ident.unraw().to_string(),
            r#type: if let WrapperType::Convert(wrapper) = &arg.wrapper {
                SdkType::Named {
                    name: get_type_name(wrapper)?,
                }
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
    Named { name: String },
}

impl SdkType {
    pub fn from_type(ty: &Type) -> Option<Self> {
        let type_name = get_type_name(ty)?;

        Some(match &*type_name {
            "bool" => Self::Bool,
            "i32" | "u32" | "c_uchar" | "usize" => Self::Int,
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
            LinkItemReturnType::Type {
                return_type,
                wrapper,
                ..
            } => {
                if let WrapperType::Convert(wrapper) = wrapper {
                    Some(Self::Named {
                        name: get_type_name(wrapper)?,
                    })
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
    variants: HashMap<String, i64>,
}

impl SdkEnum {
    pub fn new(item: &LinkEnum) -> Option<Self> {
        Some(Self {
            name: item.name.unraw().to_string(),
            underlying_type: SdkType::from_type(&item.underlying_type)?,
            variants: item
                .variants
                .iter()
                .map(|(variant, value)| {
                    let parsed = value.base10_parse::<i64>();
                    let parsed = match parsed {
                        Ok(val) => val,
                        Err(err) => {
                            emit_error!(err.span(), "Failed to parse enum value: {}", err);
                            return None;
                        }
                    };
                    Some((variant.unraw().to_string(), parsed))
                })
                .collect::<Option<HashMap<_, _>>>()?,
        })
    }
}

fn get_type_name(ty: &Type) -> Option<String> {
    let path = if let Type::Path(path) = ty {
        path
    } else {
        emit_error!(
            ty,
            "This type is not allowed because it can't be represented in the API description format"
        );
        return None;
    };

    let name = path
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");

    Some(name)
}
