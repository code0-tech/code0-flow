use crate::{
    flow_definition::Reader,
    flow_service::{auth::get_authorization_metadata, retry::create_channel_with_retry},
};
use tonic::{Extensions, Request, transport::Channel};
use tucana::{
    aquila::{ModuleUpdateRequest, module_service_client::ModuleServiceClient},
    shared::Module,
};

pub mod auth;
pub mod retry;

pub struct FlowUpdateService {
    modules: Vec<Module>,
    channel: Channel,
    aquila_token: String,
    definition_source: Option<String>,
}

impl FlowUpdateService {
    /// Create a new FlowUpdateService instance from an Aquila URL and a definition path.
    ///
    /// This reads the definition files from the given path as modules and initializes the
    /// service with those module definitions.
    pub async fn from_url(aquila_url: String, definition_path: &str, aquila_token: String) -> Self {
        let reader = Reader::configure(definition_path.to_string(), true, vec![], None);
        let modules = match reader.read_modules() {
            Ok(modules) => modules,
            Err(error) => {
                log::error!("Error occurred while reading definitions: {:?}", error);
                panic!("Error occurred while reading definitions")
            }
        };

        let channel = create_channel_with_retry("Aquila", aquila_url).await;

        Self {
            modules,
            channel,
            aquila_token,
            definition_source: None,
        }
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
