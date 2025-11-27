use tucana::shared::{DefinitionDataType, FlowType, RuntimeFunctionDefinition, Version};

pub trait HasVersion {
    fn version(&self) -> &Option<Version>;
    fn version_mut(&mut self) -> &mut Option<Version>;

    fn normalize_version(&mut self) {
        self.version_mut().get_or_insert(Version {
            major: 0,
            minor: 0,
            patch: 0,
        });
    }

    fn is_accepted(&self, filter: &Option<Version>) -> bool {
        filter
            .as_ref()
            .map_or(true, |v| self.version().as_ref() == Some(v))
    }
}

impl HasVersion for DefinitionDataType {
    fn version(&self) -> &Option<Version> {
        &self.version
    }

    fn version_mut(&mut self) -> &mut Option<Version> {
        &mut self.version
    }
}

impl HasVersion for FlowType {
    fn version(&self) -> &Option<Version> {
        &self.version
    }

    fn version_mut(&mut self) -> &mut Option<Version> {
        &mut self.version
    }
}

impl HasVersion for RuntimeFunctionDefinition {
    fn version(&self) -> &Option<Version> {
        &self.version
    }

    fn version_mut(&mut self) -> &mut Option<Version> {
        &mut self.version
    }
}