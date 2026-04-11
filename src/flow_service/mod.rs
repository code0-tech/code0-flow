use crate::{
    flow_definition::Reader,
    flow_service::{auth::get_authorization_metadata, retry::create_channel_with_retry},
};
use tonic::{Extensions, Request, transport::Channel};
use tucana::{
    aquila::{
        DataTypeUpdateRequest, FlowTypeUpdateRequest, RuntimeFunctionDefinitionUpdateRequest,
        data_type_service_client::DataTypeServiceClient,
        flow_type_service_client::FlowTypeServiceClient,
        runtime_function_definition_service_client::RuntimeFunctionDefinitionServiceClient,
    },
    shared::{DefinitionDataType as DataType, FlowType, RuntimeFunctionDefinition},
};

pub mod auth;
pub mod retry;

pub struct FlowUpdateService {
    data_types: Vec<DataType>,
    runtime_definitions: Vec<RuntimeFunctionDefinition>,
    flow_types: Vec<FlowType>,
    channel: Channel,
    aquila_token: String,
}

impl FlowUpdateService {
    /// Create a new FlowUpdateService instance from an Aquila URL and a definition path.
    ///
    /// This will read the definition files from the given path and initialize the service with the data types, runtime definitions, and flow types.
    pub async fn from_url(aquila_url: String, definition_path: &str, aquila_token: String) -> Self {
        let mut data_types = Vec::new();
        let mut runtime_definitions = Vec::new();
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
            runtime_definitions.append(&mut feature.functions.clone());
        }

        let channel = create_channel_with_retry("Aquila", aquila_url).await;

        Self {
            data_types,
            runtime_definitions,
            flow_types,
            channel,
            aquila_token,
        }
    }

    pub fn with_flow_types(mut self, flow_types: Vec<FlowType>) -> Self {
        self.flow_types = flow_types;
        self
    }

    pub fn with_data_types(mut self, data_types: Vec<DataType>) -> Self {
        self.data_types = data_types;
        self
    }

    pub fn with_runtime_definitions(
        mut self,
        runtime_definitions: Vec<RuntimeFunctionDefinition>,
    ) -> Self {
        self.runtime_definitions = runtime_definitions;
        self
    }

    pub async fn send(&self) {
        self.update_data_types().await;
        self.update_runtime_definitions().await;
        self.update_flow_types().await;
    }

    async fn update_data_types(&self) {
        if self.data_types.is_empty() {
            log::info!("No data types to update");
            return;
        }

        log::info!("Updating the current DataTypes!");
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
                log::info!(
                    "Was the update of the DataTypes accepted by Sagittarius? {}",
                    response.into_inner().success
                );
            }
            Err(err) => {
                log::error!("Failed to update data types: {:?}", err);
            }
        }
    }

    async fn update_runtime_definitions(&self) {
        if self.runtime_definitions.is_empty() {
            log::info!("No runtime definitions to update");
            return;
        }

        log::info!("Updating the current RuntimeDefinitions!");
        let mut client = RuntimeFunctionDefinitionServiceClient::new(self.channel.clone());
        let request = Request::from_parts(
            get_authorization_metadata(&self.aquila_token),
            Extensions::new(),
            RuntimeFunctionDefinitionUpdateRequest {
                runtime_functions: self.runtime_definitions.clone(),
            },
        );

        match client.update(request).await {
            Ok(response) => {
                log::info!(
                    "Was the update of the RuntimeFunctionDefinitions accepted by Sagittarius? {}",
                    response.into_inner().success
                );
            }
            Err(err) => {
                log::error!("Failed to update runtime function definitions: {:?}", err);
            }
        }
    }

    async fn update_flow_types(&self) {
        if self.flow_types.is_empty() {
            log::info!("No FlowTypes to update!");
            return;
        }

        log::info!("Updating the current FlowTypes!");
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
                log::info!(
                    "Was the update of the FlowTypes accepted by Sagittarius? {}",
                    response.into_inner().success
                );
            }
            Err(err) => {
                log::error!("Failed to update flow types: {:?}", err);
            }
        }
    }
}
