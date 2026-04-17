use crate::{
    flow_definition::Reader,
    flow_service::{auth::get_authorization_metadata, retry::create_channel_with_retry},
};
use tonic::{Extensions, Request, transport::Channel};
use tucana::{
    aquila::{
        DataTypeUpdateRequest, FlowTypeUpdateRequest, FunctionDefinitionUpdateRequest,
        RuntimeFunctionDefinitionUpdateRequest, data_type_service_client::DataTypeServiceClient,
        flow_type_service_client::FlowTypeServiceClient,
        function_definition_service_client::FunctionDefinitionServiceClient,
        runtime_function_definition_service_client::RuntimeFunctionDefinitionServiceClient,
    },
    shared::{
        DefinitionDataType as DataType, FlowType, FunctionDefinition, RuntimeFunctionDefinition,
    },
};

pub mod auth;
pub mod retry;

pub struct FlowUpdateService {
    data_types: Vec<DataType>,
    runtime_functions: Vec<RuntimeFunctionDefinition>,
    functions: Vec<FunctionDefinition>,
    flow_types: Vec<FlowType>,
    channel: Channel,
    aquila_token: String,
    definition_source: Option<String>,
}

impl FlowUpdateService {
    /// Create a new FlowUpdateService instance from an Aquila URL and a definition path.
    ///
    /// This will read the definition files from the given path and initialize the service with the data types, runtime function definitions, function definitions, and flow types.
    pub async fn from_url(aquila_url: String, definition_path: &str, aquila_token: String) -> Self {
        let mut data_types = Vec::new();
        let mut runtime_functions = Vec::new();
        let mut functions = Vec::new();
        let mut flow_types = Vec::new();

        let reader = Reader::configure(definition_path.to_string(), true, vec![], None);

        let features = match reader.read_features() {
            Ok(features) => features,
            Err(error) => {
                log::error!("Error occurred while reading definitions: {:?}", error);
                panic!("Error occurred while reading definitions")
            }
        };

        for feature in features {
            data_types.append(&mut feature.data_types.clone());
            flow_types.append(&mut feature.flow_types.clone());
            runtime_functions.append(&mut feature.runtime_functions.clone());
            functions.append(&mut feature.functions.clone());
        }

        let channel = create_channel_with_retry("Aquila", aquila_url).await;

        Self {
            data_types,
            runtime_functions,
            functions,
            flow_types,
            channel,
            aquila_token,
            definition_source: None,
        }
    }

    pub fn with_definition_source(mut self, source: String) -> Self {
        self.definition_source = Some(source);
        self
    }

    pub fn with_flow_types(mut self, flow_types: Vec<FlowType>) -> Self {
        self.flow_types = flow_types;
        self
    }

    pub fn with_data_types(mut self, data_types: Vec<DataType>) -> Self {
        self.data_types = data_types;
        self
    }

    pub fn with_runtime_functions(
        mut self,
        runtime_functions: Vec<RuntimeFunctionDefinition>,
    ) -> Self {
        self.runtime_functions = runtime_functions;
        self
    }

    pub fn with_functions(mut self, functions: Vec<FunctionDefinition>) -> Self {
        self.functions = functions;
        self
    }

    pub async fn send(&mut self) {
        let _ = self.send_with_status().await;
    }

    pub async fn send_with_status(&mut self) -> bool {
        let data_types_success = self.update_data_types().await;
        let runtime_functions_success = self.update_runtime_functions().await;
        let functions_success = self.update_functions().await;
        let flow_types_success = self.update_flow_types().await;
        data_types_success && runtime_functions_success && functions_success && flow_types_success
    }

    async fn update_data_types(&mut self) -> bool {
        if self.data_types.is_empty() {
            log::info!("No DataTypes present.");
            return true;
        }

        if let Some(source) = &self.definition_source {
            self.data_types = self
                .data_types
                .clone()
                .into_iter()
                .map(|mut x| {
                    x.definition_source = source.to_string();
                    x
                })
                .collect()
        };

        log::info!("Updating {} DataTypes.", self.data_types.len());
        let mut client = DataTypeServiceClient::new(self.channel.clone());
        let request = Request::from_parts(
            get_authorization_metadata(&self.aquila_token),
            Extensions::new(),
            DataTypeUpdateRequest {
                data_types: self.data_types.clone(),
            },
        );

        match client.update(request).await {
            Ok(response) => {
                let res = response.into_inner();
                log::info!(
                    "Was the update of the DataTypes accepted by Sagittarius? {}",
                    res.success
                );

                res.success
            }
            Err(err) => {
                log::error!("Failed to update data types: {:?}", err);
                false
            }
        }
    }

    async fn update_functions(&mut self) -> bool {
        if self.functions.is_empty() {
            log::info!("No FunctionDefinitions present.");
            return true;
        }

        if let Some(source) = &self.definition_source {
            self.functions = self
                .functions
                .clone()
                .into_iter()
                .map(|mut x| {
                    x.definition_source = source.to_string();
                    x
                })
                .collect()
        };

        log::info!("Updating {} FunctionDefinitions.", self.functions.len());
        let mut client = FunctionDefinitionServiceClient::new(self.channel.clone());
        let request = Request::from_parts(
            get_authorization_metadata(&self.aquila_token),
            Extensions::new(),
            FunctionDefinitionUpdateRequest {
                functions: self.functions.clone(),
            },
        );

        match client.update(request).await {
            Ok(response) => {
                let res = response.into_inner();
                log::info!(
                    "Was the update of the FunctionDefinitions accepted by Sagittarius? {}",
                    res.success
                );
                res.success
            }
            Err(err) => {
                log::error!("Failed to update function definitions: {:?}", err);
                false
            }
        }
    }

    async fn update_runtime_functions(&mut self) -> bool {
        if self.runtime_functions.is_empty() {
            log::info!("No RuntimeFunctionDefinitions present.");
            return true;
        }

        if let Some(source) = &self.definition_source {
            self.runtime_functions = self
                .runtime_functions
                .clone()
                .into_iter()
                .map(|mut x| {
                    x.definition_source = source.to_string();
                    x
                })
                .collect()
        };

        log::info!(
            "Updating {} RuntimeFunctionDefinitions.",
            self.runtime_functions.len()
        );
        let mut client = RuntimeFunctionDefinitionServiceClient::new(self.channel.clone());
        let request = Request::from_parts(
            get_authorization_metadata(&self.aquila_token),
            Extensions::new(),
            RuntimeFunctionDefinitionUpdateRequest {
                runtime_functions: self.runtime_functions.clone(),
            },
        );

        match client.update(request).await {
            Ok(response) => {
                let res = response.into_inner();
                log::info!(
                    "Was the update of the RuntimeFunctionDefinitions accepted by Sagittarius? {}",
                    res.success
                );
                res.success
            }
            Err(err) => {
                log::error!("Failed to update runtime function definitions: {:?}", err);
                false
            }
        }
    }

    async fn update_flow_types(&mut self) -> bool {
        if self.flow_types.is_empty() {
            log::info!("No FlowTypes present.");
            return true;
        }

        self.flow_types = self
            .flow_types
            .clone()
            .into_iter()
            .map(|mut x| {
                x.definition_source = self.definition_source.clone();
                x
            })
            .collect();

        log::info!("Updating {} FlowTypes.", self.flow_types.len());
        let mut client = FlowTypeServiceClient::new(self.channel.clone());
        let request = Request::from_parts(
            get_authorization_metadata(&self.aquila_token),
            Extensions::new(),
            FlowTypeUpdateRequest {
                flow_types: self.flow_types.clone(),
            },
        );

        match client.update(request).await {
            Ok(response) => {
                let res = response.into_inner();
                log::info!(
                    "Was the update of the FlowTypes accepted by Sagittarius? {}",
                    res.success
                );
                res.success
            }
            Err(err) => {
                log::error!("Failed to update flow types: {:?}", err);
                false
            }
        }
    }
}
