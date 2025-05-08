# Code0-Flow - Flow Management Library

[![Crate](https://img.shields.io/crates/v/code0-flow.svg)](https://crates.io/crates/code0-flow)
[![Documentation](https://docs.rs/code0-flow/badge.svg)](https://docs.rs/code0-flow)

code0-flow is a Rust library developed by Code0 for managing flows within the FlowQueue and FlowStore. This libray is only for the internal usage of the execution block services (Aquila, Draco & Tarurs) and is not intendet to get used elsewhere.

## Features

- `flow_store` insert, delete & query Flows in the FlowStore
- `flow_definition` update the Adapter & Runtime definitions
- `flow_queue` send a Flow into the queue to be executed.

## FlowStore

### Keys

The key in the FlowStore in a Store will always have the pattern:

`flow_id::project_id::flow_type_identifier::<any_flow_type_specific_data>`

E.g. for REST:

`1::1::REST::test.code0.tech::GET`

This would be a REST Flow identifier

## FlowDefinition

```rust
// Define FlowType
let flow_type = FlowType {
    identifier: String::from("REST"),
    settings: vec![],
    input_type_identifier: Some(String::from("HTTP_REQUEST_OBJECT")),
    return_type_identifier: Some(String::from("HTTP_RESPONSE_OBJECT")),
    editable: true,
    name: vec![Translation {
        code: String::from("en-US"),
        content: String::from("Rest Endpoint"),
    }],
    description: vec![Translation {
        code: String::from("en-US"),
        content: String::from("A REST API is a web service that lets clients interact with data on a server using standard HTTP methods like GET, POST, PUT, and DELETE, usually returning results in JSON format."),
    }],
};

// Define DataTypes
let data_type = DataType {
    variant: 5,
    name: vec![Translation {
        code: String::from("en-US"),
        content: String::from("HTTP Headers"),
    }],
    identifier: String::from("HTTP_HEADER_MAP"),
    input_types: vec![],
    return_type: None,
    parent_type_identifier: Some(String::from("ARRAY")),
    rules: vec![DataTypeRule {
        config: Some(Config::ContainsType(DataTypeContainsTypeRuleConfig {
            data_type_identifier: String::from("HTTP_HEADER_ENTRY"),
        })),
    }],
}

// Send to get updated in Sagittarius
let update_client = code0_flow::flow_definition::FlowUpdateService::from_url(aquila_url)
           .with_data_types(vec![data_type])
           .with_flow_types(vec![flow_type]);

// Response --> true if successfull
update_client.send().await;
```
