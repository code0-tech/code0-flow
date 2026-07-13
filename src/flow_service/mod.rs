use crate::{
    flow_definition::Reader,
    flow_service::{auth::get_authorization_metadata, retry::create_channel_with_retry},
};
use tokio::time::Duration;
use tonic::{Extensions, Request, transport::Channel};
use tucana::{
    aquila::{ModuleUpdateRequest, module_service_client::ModuleServiceClient},
    shared::{Module, ModuleDefinition},
};

pub mod auth;
pub mod retry;

pub struct FlowUpdateService {
    modules: Vec<Module>,
    channel: Channel,
    aquila_token: String,
    definition_source: Option<String>,
}

pub struct ModuleDefinitionAppendix {
    pub module_identifier: String,
    pub definitions: Vec<ModuleDefinition>,
}

impl FlowUpdateService {
    /// Create a new FlowUpdateService instance from an Aquila URL and a definition path.
    ///
    /// This reads the definition files from the given path as modules and initializes the
    /// service with those module definitions.
    pub async fn from_url(
        aquila_url: String,
        definition_path: &str,
        aquila_token: String,
        connect_timeout: Duration,
        request_timeout: Duration,
    ) -> Self {
        let reader = Reader::configure(definition_path.to_string(), true, vec![], None);
        let modules = match reader.read_modules() {
            Ok(modules) => modules,
            Err(error) => {
                log::error!("Error occurred while reading definitions: {:?}", error);
                panic!("Error occurred while reading definitions")
            }
        };

        let channel =
            create_channel_with_retry("Aquila", aquila_url, connect_timeout, request_timeout).await;

        Self {
            modules,
            channel,
            aquila_token,
            definition_source: None,
        }
    }

    pub fn with_appendix(mut self, appendices: Vec<ModuleDefinitionAppendix>) -> Self {
        append_definitions_to_matching_modules(&mut self.modules, appendices);
        self
    }

    pub fn with_definition_source(mut self, source: String) -> Self {
        self.definition_source = Some(source);
        self
    }

    pub async fn send(&mut self) {
        let _ = self.send_with_status().await;
    }

    pub async fn send_with_status(&mut self) -> bool {
        self.update().await
    }

    async fn update(&mut self) -> bool {
        if self.modules.is_empty() {
            log::info!("No Modules are present, aborting update.");
            return true;
        }

        let mut modules = self.modules.clone();
        if let Some(source) = &self.definition_source {
            modules = modules
                .into_iter()
                .map(|module| apply_definition_source_to_module(module, source.clone()))
                .collect::<Vec<_>>();
        }

        log::info!("Updating {} Modules.", self.modules.len());
        let mut client = ModuleServiceClient::new(self.channel.clone());
        let request = Request::from_parts(
            get_authorization_metadata(&self.aquila_token),
            Extensions::new(),
            ModuleUpdateRequest { modules },
        );

        match client.update(request).await {
            Ok(response) => {
                let res = response.into_inner();

                match res.success {
                    true => log::info!("Module definition update has been successful"),
                    false => log::warn!("Module definition update has been unsuccessful"),
                };

                res.success
            }
            Err(err) => {
                log::error!("Module definition update failed. Reason: {:?}", err);
                false
            }
        }
    }
}

fn append_definitions_to_matching_modules(
    modules: &mut [Module],
    appendices: Vec<ModuleDefinitionAppendix>,
) {
    for appendix in appendices {
        for module in modules.iter_mut() {
            if module.identifier == appendix.module_identifier {
                module.definitions.extend(appendix.definitions.clone());
            }
        }
    }
}

fn apply_definition_source_to_module(mut module: Module, source: String) -> Module {
    for data_type in &mut module.definition_data_types {
        data_type.definition_source = source.clone();
    }
    for flow_type in &mut module.flow_types {
        flow_type.definition_source = Some(source.clone());
    }
    for runtime_flow_type in &mut module.runtime_flow_types {
        runtime_flow_type.definition_source = Some(source.clone());
    }
    for function in &mut module.function_definitions {
        function.definition_source = source.clone();
    }
    for runtime_function in &mut module.runtime_function_definitions {
        runtime_function.definition_source = source.clone();
    }

    module
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(identifier: &str) -> Module {
        Module {
            identifier: identifier.to_string(),
            name: Vec::new(),
            description: Vec::new(),
            documentation: String::new(),
            author: String::new(),
            icon: String::new(),
            version: String::new(),
            flow_types: Vec::new(),
            runtime_flow_types: Vec::new(),
            function_definitions: Vec::new(),
            runtime_function_definitions: Vec::new(),
            definition_data_types: Vec::new(),
            configurations: Vec::new(),
            definitions: Vec::new(),
            definition_source: identifier.to_string(),
        }
    }

    fn definition(identifier: &str) -> ModuleDefinition {
        ModuleDefinition {
            flow_type_identifier: vec![identifier.to_string()],
            value: None,
        }
    }

    #[test]
    fn appends_appendix_definitions_to_all_modules_with_same_identifier() {
        let mut modules = vec![module("shared"), module("other"), module("shared")];
        let appendices = vec![ModuleDefinitionAppendix {
            module_identifier: "shared".to_string(),
            definitions: vec![definition("flow")],
        }];

        append_definitions_to_matching_modules(&mut modules, appendices);

        assert_eq!(modules[0].definitions.len(), 1);
        assert_eq!(modules[1].definitions.len(), 0);
        assert_eq!(modules[2].definitions.len(), 1);
    }

    #[test]
    fn appends_multiple_matching_appendices() {
        let mut modules = vec![module("shared")];
        let appendices = vec![
            ModuleDefinitionAppendix {
                module_identifier: "shared".to_string(),
                definitions: vec![definition("flow-a")],
            },
            ModuleDefinitionAppendix {
                module_identifier: "shared".to_string(),
                definitions: vec![definition("flow-b")],
            },
            ModuleDefinitionAppendix {
                module_identifier: "other".to_string(),
                definitions: vec![definition("flow-c")],
            },
        ];

        append_definitions_to_matching_modules(&mut modules, appendices);

        assert_eq!(modules[0].definitions.len(), 2);
        assert_eq!(
            modules[0].definitions[0].flow_type_identifier,
            vec!["flow-a".to_string()]
        );
        assert_eq!(
            modules[0].definitions[1].flow_type_identifier,
            vec!["flow-b".to_string()]
        );
    }
}
