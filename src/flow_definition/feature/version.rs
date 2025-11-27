use tucana::shared::{DefinitionDataType, FlowType, RuntimeFunctionDefinition};

pub trait HasVersion {
    fn version(&self) -> &String;

    fn is_accepted(&self, filter: &Option<String>) -> bool {
        filter
            .as_ref()
            .is_none_or(|v| self.version() == v)
    }
}

impl HasVersion for DefinitionDataType {
    fn version(&self) -> &String {
        &self.version
    }
}

impl HasVersion for FlowType {
    fn version(&self) -> &String {
        &self.version
    }
}

impl HasVersion for RuntimeFunctionDefinition {
    fn version(&self) -> &String {
        &self.version
    }
}
