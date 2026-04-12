pub mod version;

use serde::Deserialize;
use tucana::shared::{DefinitionDataType, FlowType, FunctionDefinition, RuntimeFunctionDefinition};

#[derive(Deserialize, Debug, Clone)]
pub struct Feature {
    pub name: String,
    pub data_types: Vec<DefinitionDataType>,
    pub flow_types: Vec<FlowType>,
    pub runtime_functions: Vec<RuntimeFunctionDefinition>,
    pub functions: Vec<FunctionDefinition>,
}
