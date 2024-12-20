use std::collections::HashMap;
use std::path::Path;

use crate::models::sc_abi::endpoint::{Endpoint, Mutability};
use crate::models::sc_abi::field::Field;
use crate::models::sc_abi::output::Output;
use crate::utils::string_mutation::StringMutation;

#[derive(Debug)]
pub struct EndpointProps {
    pub mutability: String,
    pub name: String,
    pub import_export_name: String,
    pub file_name: String,
    pub inputs: Vec<EndpointIO>,
    pub outputs: Vec<EndpointIO>,
}

#[derive(Clone, Debug)]
pub struct EndpointPath {
    pub folder: String,
    pub page_name: String,
}

#[derive(Debug)]
pub struct EndpointIO {
    pub getter: String,
    pub setter: String,
    pub type_: String,
    pub initial_value: String,
    pub fields: Vec<Field>,
    pub args_serializer: String,
}

pub fn get_endpoints_props(
    custom_structs: &HashMap<String, Vec<Field>>,
    endpoints: &Vec<Endpoint>,
) -> HashMap<(String, String), Vec<EndpointProps>> {
    let mut pages_props: HashMap<(String, String), Vec<EndpointProps>> = Default::default();

    for endpoint in endpoints {
        let endpoint_name = endpoint
            .name()
            .to_string()
            .snake_to_camel_case()
            .capitalize_first_letter();
        let import_export_name = match endpoint.mutability() {
            Mutability::Mutable => endpoint_name,
            Mutability::Readonly => {
                format!("use{}", endpoint_name)
            }
        };
        let endpoint_file_name = endpoint.name().to_string().snake_to_kebab_case();
        let endpoint_inputs = get_endpoint_inputs(custom_structs, endpoint.inputs());
        let endpoint_outputs = get_endpoint_outputs(custom_structs, endpoint.outputs());
        let endpoint_path = get_endpoint_path(endpoint.docs());

        if pages_props.contains_key(&(
            endpoint_path.folder.clone(),
            endpoint_path.page_name.clone(),
        )) {
            let EndpointPath { folder, page_name } = endpoint_path.clone();

            if let Some(_endpoint_props) = pages_props.get_mut(&(folder, page_name)) {
                _endpoint_props.push(EndpointProps {
                    mutability: match endpoint.mutability() {
                        Mutability::Mutable => "mutable".to_string(),
                        Mutability::Readonly => "readonly".to_string(),
                    },
                    name: endpoint.name().to_string(),
                    import_export_name,
                    file_name: endpoint_file_name,
                    inputs: endpoint_inputs,
                    outputs: endpoint_outputs,
                });
            }
        } else {
            let EndpointPath { folder, page_name } = endpoint_path.clone();
            let mut endpoints_props = Vec::<EndpointProps>::new();

            endpoints_props.push(EndpointProps {
                mutability: match endpoint.mutability() {
                    Mutability::Mutable => "mutable".to_string(),
                    Mutability::Readonly => "readonly".to_string(),
                },
                name: endpoint.name().to_string(),
                import_export_name,
                file_name: endpoint_file_name,
                inputs: endpoint_inputs,
                outputs: endpoint_outputs,
            });

            pages_props.insert((folder, page_name), endpoints_props);
        }
    }

    pages_props
}

fn get_endpoint_path(docs: &Option<Vec<String>>) -> EndpointPath {
    let mut folder = String::new();
    let mut page_name = String::new();

    if let Some(_docs) = docs {
        for string in _docs {
            if string.contains("path:") {
                let path_str = string
                    .replace(" ", "")
                    .trim_start_matches("path:")
                    .to_string();

                let path = Path::new(&path_str);

                if let Some(parent) = path.parent() {
                    let parent = parent.to_str().unwrap_or("").to_string();

                    folder = if parent == "/" {
                        "/pages".to_string()
                    } else {
                        parent
                    };
                }

                if let Some(file_name) = path.file_name() {
                    let file_name = file_name.to_str().unwrap_or("").to_string();

                    page_name = if file_name == "pages" {
                        "dashboard".to_string()
                    } else {
                        file_name
                    };
                }

                if !folder.is_empty() && !page_name.is_empty() {
                    return EndpointPath { folder, page_name };
                }
            }
        }
    }

    EndpointPath {
        folder: "".to_string(),
        page_name: "".to_string(),
    }
}

fn get_endpoint_inputs(
    custom_structs: &HashMap<String, Vec<Field>>,
    inputs: &Vec<Field>,
) -> Vec<EndpointIO> {
    let mut endpoint_inputs = Vec::<EndpointIO>::new();

    for (name, fields) in custom_structs {
        for input in inputs {
            match input.type_().as_str() {
                "Address" => endpoint_inputs.push(EndpointIO {
                    getter: input.name().to_string().snake_to_camel_case(),
                    setter: format!(
                        "set{}",
                        input
                            .name()
                            .to_string()
                            .snake_to_camel_case()
                            .capitalize_first_letter()
                    ),
                    type_: "string".to_string(),
                    initial_value: "\"\"".to_string(),
                    fields: Vec::new(),
                    args_serializer: "AddressValue".to_string(),
                }),
                "TokenIdentifier" => endpoint_inputs.push(EndpointIO {
                    getter: input.name().to_string().snake_to_camel_case(),
                    setter: format!(
                        "set{}",
                        input
                            .name()
                            .to_string()
                            .snake_to_camel_case()
                            .capitalize_first_letter()
                    ),
                    type_: "string".to_string(),
                    initial_value: "\"\"".to_string(),
                    fields: Vec::new(),
                    args_serializer: "TokenIdentifierValue".to_string(),
                }),
                "usize" | "u8" | "u16" | "u32" | "u64" | "BigUint" => {
                    endpoint_inputs.push(EndpointIO {
                        getter: input.name().to_string().snake_to_camel_case(),
                        setter: format!(
                            "set{}",
                            input
                                .name()
                                .to_string()
                                .snake_to_camel_case()
                                .capitalize_first_letter()
                        ),
                        type_: "number".to_string(),
                        initial_value: "0".to_string(),
                        fields: Vec::new(),
                        args_serializer: match input.type_().as_str() {
                            "usize" => "U32Value".to_string(),
                            "u8" => "U8Value".to_string(),
                            "u16" => "U16Value".to_string(),
                            "u32" => "U32Value".to_string(),
                            "u64" => "U64Value".to_string(),
                            "BigUint" => "BigUIntValue".to_string(),
                            _ => "".to_string(),
                        },
                    })
                }
                "isize" | "i8" | "i16" | "i32" | "i64" | "BigInt" => {
                    endpoint_inputs.push(EndpointIO {
                        getter: input.name().to_string().snake_to_camel_case(),
                        setter: format!(
                            "set{}",
                            input
                                .name()
                                .to_string()
                                .snake_to_camel_case()
                                .capitalize_first_letter()
                        ),
                        type_: "number".to_string(),
                        initial_value: "0".to_string(),
                        fields: Vec::new(),
                        args_serializer: match input.type_().as_str() {
                            "isize" => "I32Value".to_string(),
                            "i8" => "I8Value".to_string(),
                            "i16" => "I16Value".to_string(),
                            "i32" => "I32Value".to_string(),
                            "i64" => "I64Value".to_string(),
                            "BigInt" => "BigIntValue".to_string(),
                            _ => "".to_string(),
                        },
                    })
                }
                "bytes" => endpoint_inputs.push(EndpointIO {
                    getter: input.name().to_string().snake_to_camel_case(),
                    setter: format!(
                        "set{}",
                        input
                            .name()
                            .to_string()
                            .snake_to_camel_case()
                            .capitalize_first_letter()
                    ),
                    type_: "string".to_string(),
                    initial_value: "\"\"".to_string(),
                    fields: Vec::new(),
                    args_serializer: "BytesValue".to_string(),
                }),
                "utf-8 string" => endpoint_inputs.push(EndpointIO {
                    getter: input.name().to_string().snake_to_camel_case(),
                    setter: format!(
                        "set{}",
                        input
                            .name()
                            .to_string()
                            .snake_to_camel_case()
                            .capitalize_first_letter()
                    ),
                    type_: "string".to_string(),
                    initial_value: "\"\"".to_string(),
                    fields: Vec::new(),
                    args_serializer: "StringValue".to_string(),
                }),
                "bool" => endpoint_inputs.push(EndpointIO {
                    getter: input.name().to_string().snake_to_camel_case(),
                    setter: format!(
                        "set{}",
                        input
                            .name()
                            .to_string()
                            .snake_to_camel_case()
                            .capitalize_first_letter()
                    ),
                    type_: "boolean".to_string(),
                    initial_value: "false".to_string(),
                    fields: Vec::new(),
                    args_serializer: "BooleanValue".to_string(),
                }),
                "List<Address>" | "variadic<Address>" => endpoint_inputs.push(EndpointIO {
                    getter: input.name().to_string().snake_to_camel_case(),
                    setter: format!(
                        "set{}",
                        input
                            .name()
                            .to_string()
                            .snake_to_camel_case()
                            .capitalize_first_letter()
                    ),
                    type_: "string[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: "AddressValue".to_string(),
                }),
                "List<TokenIdentifier>" | "variadic<TokenIdentifier>" => {
                    endpoint_inputs.push(EndpointIO {
                        getter: input.name().to_string().snake_to_camel_case(),
                        setter: format!(
                            "set{}",
                            input
                                .name()
                                .to_string()
                                .snake_to_camel_case()
                                .capitalize_first_letter()
                        ),
                        type_: "string[]".to_string(),
                        initial_value: "[]".to_string(),
                        fields: Vec::new(),
                        args_serializer: "TokenIdentifierValue".to_string(),
                    })
                }
                "List<usize>" | "List<u8>" | "List<u16>" | "List<u32>" | "List<u64>"
                | "List<BigUint>" => endpoint_inputs.push(EndpointIO {
                    getter: input.name().to_string().snake_to_camel_case(),
                    setter: format!(
                        "set{}",
                        input
                            .name()
                            .to_string()
                            .snake_to_camel_case()
                            .capitalize_first_letter()
                    ),
                    type_: "number[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: match input
                        .type_()
                        .strip_prefix("List<")
                        .unwrap()
                        .trim_end_matches(">")
                    {
                        "usize" => "U32Value".to_string(),
                        "u8" => "U8Value".to_string(),
                        "u16" => "U16Value".to_string(),
                        "u32" => "U32Value".to_string(),
                        "u64" => "U64Value".to_string(),
                        "BigUint" => "BigUIntValue".to_string(),
                        _ => "".to_string(),
                    },
                }),
                "variadic<usize>" | "variadic<u8>" | "variadic<u16>" | "variadic<u32>"
                | "variadic<u64>" | "variadic<BigUint>" => endpoint_inputs.push(EndpointIO {
                    getter: input.name().to_string().snake_to_camel_case(),
                    setter: format!(
                        "set{}",
                        input
                            .name()
                            .to_string()
                            .snake_to_camel_case()
                            .capitalize_first_letter()
                    ),
                    type_: "number[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: match input
                        .type_()
                        .strip_prefix("List<")
                        .unwrap()
                        .trim_end_matches(">")
                    {
                        "usize" => "U32Value".to_string(),
                        "u8" => "U8Value".to_string(),
                        "u16" => "U16Value".to_string(),
                        "u32" => "U32Value".to_string(),
                        "u64" => "U64Value".to_string(),
                        "BigUint" => "BigUIntValue".to_string(),
                        _ => "".to_string(),
                    },
                }),
                "List<isize>" | "List<i8>" | "List<i16>" | "List<i32>" | "List<i64>"
                | "List<BigInt>" => endpoint_inputs.push(EndpointIO {
                    getter: input.name().to_string().snake_to_camel_case(),
                    setter: format!(
                        "set{}",
                        input
                            .name()
                            .to_string()
                            .snake_to_camel_case()
                            .capitalize_first_letter()
                    ),
                    type_: "number[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: match input
                        .type_()
                        .strip_prefix("List<")
                        .unwrap()
                        .trim_end_matches(">")
                    {
                        "isize" => "I32Value".to_string(),
                        "i8" => "I8Value".to_string(),
                        "i16" => "I16Value".to_string(),
                        "i32" => "I32Value".to_string(),
                        "i64" => "I64Value".to_string(),
                        "BigInt" => "BigIntValue".to_string(),
                        _ => "".to_string(),
                    },
                }),
                "variadic<isize>" | "variadic<i8>" | "variadic<i16>" | "variadic<i32>"
                | "variadic<i64>" | "variadic<BigInt>" => endpoint_inputs.push(EndpointIO {
                    getter: input.name().to_string().snake_to_camel_case(),
                    setter: format!(
                        "set{}",
                        input
                            .name()
                            .to_string()
                            .snake_to_camel_case()
                            .capitalize_first_letter()
                    ),
                    type_: "number[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: match input
                        .type_()
                        .strip_prefix("List<")
                        .unwrap()
                        .trim_end_matches(">")
                    {
                        "isize" => "I32Value".to_string(),
                        "i8" => "I8Value".to_string(),
                        "i16" => "I16Value".to_string(),
                        "i32" => "I32Value".to_string(),
                        "i64" => "I64Value".to_string(),
                        "BigInt" => "BigIntValue".to_string(),
                        _ => "".to_string(),
                    },
                }),
                "List<bytes>" | "variadic<bytes>" => endpoint_inputs.push(EndpointIO {
                    getter: input.name().to_string().snake_to_camel_case(),
                    setter: format!(
                        "set{}",
                        input
                            .name()
                            .to_string()
                            .snake_to_camel_case()
                            .capitalize_first_letter()
                    ),
                    type_: "string[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: "BytesValue".to_string(),
                }),
                "List<utf-8 string>" | "variadic<utf-8 string>" => {
                    endpoint_inputs.push(EndpointIO {
                        getter: input.name().to_string().snake_to_camel_case(),
                        setter: format!(
                            "set{}",
                            input
                                .name()
                                .to_string()
                                .snake_to_camel_case()
                                .capitalize_first_letter()
                        ),
                        type_: "string[]".to_string(),
                        initial_value: "[]".to_string(),
                        fields: Vec::new(),
                        args_serializer: "StringValue".to_string(),
                    })
                }
                "List<bool>" | "variadic<bool>" => endpoint_inputs.push(EndpointIO {
                    getter: input.name().to_string().snake_to_camel_case(),
                    setter: format!(
                        "set{}",
                        input
                            .name()
                            .to_string()
                            .snake_to_camel_case()
                            .capitalize_first_letter()
                    ),
                    type_: "boolean[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: "BooleanValue".to_string(),
                }),
                _ => {
                    if input.type_().as_str() == name {
                        endpoint_inputs.push(EndpointIO {
                            getter: name.to_string().snake_to_camel_case(),
                            setter: format!(
                                "set{}",
                                name.to_string()
                                    .snake_to_camel_case()
                                    .capitalize_first_letter()
                            ),
                            type_: name.to_string(),
                            initial_value: "{}".to_string(),
                            fields: fields.clone(),
                            args_serializer: "".to_string(),
                        })
                    }
                }
            }
        }
    }

    endpoint_inputs
}

fn get_endpoint_outputs(
    custom_structs: &HashMap<String, Vec<Field>>,
    outputs: &Vec<Output>,
) -> Vec<EndpointIO> {
    let mut endpoint_outputs = Vec::<EndpointIO>::new();

    for (name, fields) in custom_structs {
        for output in outputs {
            match output.type_().as_str() {
                "Address" => endpoint_outputs.push(EndpointIO {
                    getter: "address".to_string(),
                    setter: "setAddress".to_string(),
                    type_: "string".to_string(),
                    initial_value: "\"\"".to_string(),
                    fields: Vec::new(),
                    args_serializer: "".to_string(),
                }),
                "TokenIdentifier" => endpoint_outputs.push(EndpointIO {
                    getter: "tokenIdentifier".to_string(),
                    setter: "setTokenIdentifier".to_string(),
                    type_: "string".to_string(),
                    initial_value: "\"\"".to_string(),
                    fields: Vec::new(),
                    args_serializer: "".to_string(),
                }),
                "usize" | "u8" | "u16" | "u32" | "u64" | "BigUint" => {
                    endpoint_outputs.push(EndpointIO {
                        getter: "numb".to_string(),
                        setter: "setNumb".to_string(),
                        type_: "number".to_string(),
                        initial_value: "0".to_string(),
                        fields: Vec::new(),
                        args_serializer: "".to_string(),
                    })
                }
                "isize" | "i8" | "i16" | "i32" | "i64" | "BigInt" => {
                    endpoint_outputs.push(EndpointIO {
                        getter: "numb".to_string(),
                        setter: "setNumb".to_string(),
                        type_: "number".to_string(),
                        initial_value: "0".to_string(),
                        fields: Vec::new(),
                        args_serializer: "".to_string(),
                    })
                }
                "bytes" | "utf-8 string" => endpoint_outputs.push(EndpointIO {
                    getter: "str".to_string(),
                    setter: "setStr".to_string(),
                    type_: "string".to_string(),
                    initial_value: "\"\"".to_string(),
                    fields: Vec::new(),
                    args_serializer: "".to_string(),
                }),
                "bool" => endpoint_outputs.push(EndpointIO {
                    getter: "bool".to_string(),
                    setter: "setBool".to_string(),
                    type_: "boolean".to_string(),
                    initial_value: "false".to_string(),
                    fields: Vec::new(),
                    args_serializer: "".to_string(),
                }),
                "List<Address>" | "variadic<Address>" => endpoint_outputs.push(EndpointIO {
                    getter: "arrayAddress".to_string(),
                    setter: "setArrayAddress".to_string(),
                    type_: "string[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: "".to_string(),
                }),
                "List<usize>" | "List<u8>" | "List<u16>" | "List<u32>" | "List<u64>"
                | "List<BigUint>" => endpoint_outputs.push(EndpointIO {
                    getter: "arrayNumber".to_string(),
                    setter: "setArrayNumber".to_string(),
                    type_: "number[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: "".to_string(),
                }),
                "variadic<usize>" | "variadic<u8>" | "variadic<u16>" | "variadic<u32>"
                | "variadic<u64>" | "variadic<BigUint>" => endpoint_outputs.push(EndpointIO {
                    getter: "arrayNumber".to_string(),
                    setter: "setArrayNumber".to_string(),
                    type_: "number[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: "".to_string(),
                }),
                "List<isize>" | "List<i8>" | "List<i16>" | "List<i32>" | "List<i64>"
                | "List<BigInt>" => endpoint_outputs.push(EndpointIO {
                    getter: "arrayNumber".to_string(),
                    setter: "setArrayNumber".to_string(),
                    type_: "number[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: "".to_string(),
                }),
                "variadic<isize>" | "variadic<i8>" | "variadic<i16>" | "variadic<i32>"
                | "variadic<i64>" | "variadic<BigInt>" => endpoint_outputs.push(EndpointIO {
                    getter: "arrayNumber".to_string(),
                    setter: "setArrayNumber".to_string(),
                    type_: "number[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: "".to_string(),
                }),
                "List<bytes>"
                | "List<utf-8 string>"
                | "variadic<bytes>"
                | "variadic<utf-8 string>" => endpoint_outputs.push(EndpointIO {
                    getter: "arrayString".to_string(),
                    setter: "setArrayString".to_string(),
                    type_: "string[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: "".to_string(),
                }),
                "List<bool>" | "variadic<bool>" => endpoint_outputs.push(EndpointIO {
                    getter: "arrayBool".to_string(),
                    setter: "setArrayBool".to_string(),
                    type_: "boolean[]".to_string(),
                    initial_value: "[]".to_string(),
                    fields: Vec::new(),
                    args_serializer: "".to_string(),
                }),
                _ => {
                    if output.type_().as_str() == name {
                        endpoint_outputs.push(EndpointIO {
                            getter: name.to_string().snake_to_camel_case(),
                            setter: format!(
                                "set{}",
                                name.to_string()
                                    .snake_to_camel_case()
                                    .capitalize_first_letter()
                            ),
                            type_: name.to_string(),
                            initial_value: "{}".to_string(),
                            fields: fields.clone(),
                            args_serializer: "".to_string(),
                        })
                    }
                }
            }
        }
    }

    endpoint_outputs
}
