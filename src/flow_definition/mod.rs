mod error;
mod feature;

use crate::flow_definition::error::ReaderError;
use crate::flow_definition::feature::Feature;
use crate::flow_definition::feature::version::HasVersion;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tucana::shared::{
    DefinitionDataType, FlowType, FunctionDefinition, Module, ModuleConfigurationDefinition,
    RuntimeFlowType, RuntimeFunctionDefinition, Translation,
};
use walkdir::WalkDir;

pub struct Reader {
    should_break: bool,
    accepted_features: Vec<String>,
    accepted_version: Option<String>,
    path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ModuleConfiguration {
    pub identifier: String,
    pub name: Vec<Translation>,
    pub description: Vec<Translation>,
    pub documentation: String,
    pub author: String,
    pub icon: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,
}

#[derive(Clone, Debug, Default)]
struct LoadedModule {
    name: String,
    config: ModuleConfiguration,
    data_types: Vec<DefinitionDataType>,
    flow_types: Vec<FlowType>,
    runtime_flow_types: Vec<RuntimeFlowType>,
    functions: Vec<FunctionDefinition>,
    runtime_functions: Vec<RuntimeFunctionDefinition>,
    configurations: Vec<ModuleConfigurationDefinition>,
}

impl LoadedModule {
    fn into_module(mut self) -> Module {
        if self.config.identifier.is_empty() {
            self.config.identifier = self.name.clone();
        }

        Module {
            identifier: self.config.identifier,
            name: self.config.name,
            description: self.config.description,
            documentation: self.config.documentation,
            author: self.config.author,
            icon: self.config.icon,
            version: self.config.version,
            flow_types: self.flow_types,
            runtime_flow_types: self.runtime_flow_types,
            function_definitions: self.functions,
            runtime_function_definitions: self.runtime_functions,
            definition_data_types: self.data_types,
            configurations: self.configurations,
        }
    }
}

impl Reader {
    pub fn configure(
        path: String,
        should_break: bool,
        accepted_features: Vec<String>,
        accepted_version: Option<String>,
    ) -> Self {
        Self {
            should_break,
            accepted_features,
            accepted_version,
            path,
        }
    }

    pub fn read_features(&self) -> Result<Vec<Feature>, ReaderError> {
        let modules = self
            .read_loaded_modules()
            .map_err(|err| ReaderError::ReadFeatureError {
                path: self.path.clone(),
                source: Box::new(err),
            })?;

        Ok(modules
            .into_iter()
            .map(|module| Feature {
                name: module.name,
                data_types: module.data_types,
                flow_types: module.flow_types,
                runtime_functions: module.runtime_functions,
                functions: module.functions,
            })
            .collect())
    }

    pub fn read_modules(&self) -> Result<Vec<Module>, ReaderError> {
        let modules = self
            .read_loaded_modules()
            .map_err(|err| ReaderError::ReadFeatureError {
                path: self.path.clone(),
                source: Box::new(err),
            })?;

        Ok(modules.into_iter().map(LoadedModule::into_module).collect())
    }

    fn read_loaded_modules(&self) -> Result<Vec<LoadedModule>, ReaderError> {
        let root = Path::new(&self.path);
        if !root.exists() || !root.is_dir() {
            return Err(ReaderError::ReadDirectoryError {
                path: root.to_path_buf(),
                error: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Definition path {} does not exist", root.display()),
                ),
            });
        }

        let mut modules = Vec::new();
        for module_dir in find_module_directories(root) {
            let module_name = module_name_from_paths(root, &module_dir);
            if !self.feature_allowed(&module_name, &module_dir) {
                continue;
            }

            let mut module = LoadedModule {
                name: module_name,
                ..Default::default()
            };

            let module_file = module_dir.join("module.json");
            if module_file.is_file() {
                match read_json_file::<ModuleConfiguration>(&module_file) {
                    Ok(config) => module.config = config,
                    Err(err) if self.should_break => return Err(err),
                    Err(err) => log::warn!(
                        "Skipping invalid module definition {}: {:?}",
                        module_file.display(),
                        err
                    ),
                }
            }

            let entries =
                fs::read_dir(&module_dir).map_err(|error| ReaderError::ReadDirectoryError {
                    path: module_dir.clone(),
                    error,
                })?;

            for entry in entries {
                let entry = entry.map_err(ReaderError::DirectoryEntryError)?;
                let file_type = entry
                    .file_type()
                    .map_err(ReaderError::DirectoryEntryError)?;
                if !file_type.is_dir() {
                    continue;
                }

                let path = entry.path();
                let dir_name = entry.file_name().to_string_lossy().to_string();

                match dir_name.as_str() {
                    "flow_type" | "flow_types" => {
                        module
                            .flow_types
                            .extend(load_json_dir::<FlowType>(&path, self.should_break)?);
                    }
                    "runtime_flow_type" | "runtime_flow_types" => {
                        module
                            .runtime_flow_types
                            .extend(load_json_dir::<RuntimeFlowType>(&path, self.should_break)?);
                    }
                    "data_type" | "data_types" => {
                        module
                            .data_types
                            .extend(load_json_dir::<DefinitionDataType>(
                                &path,
                                self.should_break,
                            )?);
                    }
                    "runtime_definition" | "runtime_definitions" | "runtime_functions" => {
                        module.runtime_functions.extend(
                            load_json_dir::<RuntimeFunctionDefinition>(&path, self.should_break)?,
                        );
                    }
                    "function" | "functions" => {
                        module.functions.extend(load_json_dir::<FunctionDefinition>(
                            &path,
                            self.should_break,
                        )?);
                    }
                    "configuration" | "configurations" => {
                        module.configurations.extend(
                            load_json_dir::<ModuleConfigurationDefinition>(
                                &path,
                                self.should_break,
                            )?,
                        );
                    }
                    _ => {}
                }
            }

            module
                .data_types
                .retain(|item| item.is_accepted(&self.accepted_version));
            module
                .flow_types
                .retain(|item| item.is_accepted(&self.accepted_version));
            module
                .runtime_flow_types
                .retain(|item| item.is_accepted(&self.accepted_version));
            module
                .functions
                .retain(|item| item.is_accepted(&self.accepted_version));
            module
                .runtime_functions
                .retain(|item| item.is_accepted(&self.accepted_version));

            modules.push(module);
        }

        Ok(modules)
    }

    fn feature_allowed(&self, module_name: &str, module_path: &Path) -> bool {
        if self.accepted_features.is_empty() {
            return true;
        }

        let short_name = module_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        self.accepted_features
            .iter()
            .any(|feature| feature == module_name || feature == short_name)
    }
}

fn read_json_file<T: DeserializeOwned>(path: &Path) -> Result<T, ReaderError> {
    let content = fs::read_to_string(path).map_err(|error| ReaderError::ReadFileError {
        path: path.to_path_buf(),
        error,
    })?;

    serde_json::from_str::<T>(&content).map_err(|error| ReaderError::JsonError {
        path: path.to_path_buf(),
        error,
    })
}

fn load_json_dir<T: DeserializeOwned>(
    dir: &Path,
    should_break: bool,
) -> Result<Vec<T>, ReaderError> {
    let mut items = Vec::new();
    for file in WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        })
    {
        match read_json_file::<T>(&file.as_path()) {
            Ok(item) => items.push(item),
            Err(err) if should_break => return Err(err),
            Err(err) => log::warn!("Skipping invalid definition {}: {:?}", file.display(), err),
        }
    }

    Ok(items)
}

fn find_module_directories(root: &Path) -> Vec<PathBuf> {
    let mut modules = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_dir())
        .map(|entry| entry.into_path())
        .filter(|path| looks_like_module(path))
        .collect::<Vec<_>>();
    modules.sort();
    modules
}

fn looks_like_module(path: &Path) -> bool {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return false,
    };

    entries.flatten().any(|entry| {
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => return false,
        };

        let name = entry.file_name().to_string_lossy().to_string();
        name == "module.json" || (file_type.is_dir() && is_definition_dir(&name))
    })
}

fn is_definition_dir(name: &str) -> bool {
    matches!(
        name,
        "flow_type"
            | "flow_types"
            | "runtime_flow_type"
            | "runtime_flow_types"
            | "data_type"
            | "data_types"
            | "runtime_definition"
            | "runtime_definitions"
            | "runtime_functions"
            | "function"
            | "functions"
            | "configuration"
            | "configurations"
    )
}

fn module_name_from_paths(root: &Path, module_path: &Path) -> String {
    let relative = module_path
        .strip_prefix(root)
        .ok()
        .and_then(|path| path.to_str())
        .unwrap_or_default();

    if relative.is_empty() || relative == "." {
        module_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("module")
            .to_string()
    } else {
        relative.to_string()
    }
}
