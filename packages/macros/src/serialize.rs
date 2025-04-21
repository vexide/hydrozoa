use std::collections::HashMap;

use serde::Serialize;

#[derive(Serialize)]
struct SdkModule {
    name: String,
    items: Vec<SdkItem>,
    enums: Vec<SdkEnum>,
}

#[derive(Serialize)]
struct SdkItem {
    name: String,
    params: Vec<SdkItemParam>,
    returns: SdkType,
}

#[derive(Serialize)]
struct SdkItemParam {
    name: String,
    r#type: SdkType,
}

#[derive(Serialize)]
enum SdkType {
    Int,
    Long,
    Float,
    Double,
    Enum(String)
}

#[derive(Serialize)]
struct SdkEnum {
    name: String,
    underlying_type: SdkType,
    variants: HashMap<String, f64>,
}