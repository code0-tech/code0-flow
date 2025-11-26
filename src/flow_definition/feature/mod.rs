use serde::Deserialize;
use tucana::shared::{DefinitionDataType, FlowType, RuntimeFunctionDefinition};

#[derive(Deserialize, Debug, Clone)]
pub struct Feature {
    pub name: String,
    pub data_types: Vec<DefinitionDataType>,
    pub flow_types: Vec<FlowType>,
    pub functions: Vec<RuntimeFunctionDefinition>,
}
