mod error;
mod feature;

use crate::flow_definition::error::ReaderError;
use crate::flow_definition::feature::Feature;
use crate::flow_definition::feature::version::HasVersion;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;
use tucana::shared::{DefinitionDataType, FlowType, RuntimeFunctionDefinition, Version};
use walkdir::WalkDir;

pub struct Reader {
    should_break: bool,
    accepted_features: Vec<String>,
    accepted_version: Option<Version>,
    path: String,
}

impl Reader {
    pub fn configure(
        path: String,
        should_break: bool,
        accepted_features: Vec<String>,
        accepted_version: Option<Version>,
    ) -> Self {
        Self {
            should_break,
            accepted_features,
            accepted_version,
            path,
        }
    }

    pub fn read_features(&self) -> Result<Vec<Feature>, ReaderError> {
        let definitions = Path::new(&self.path);

        match self.read_feature_content(definitions) {
            Ok(features) => {
                log::info!(
                    "Loaded {:?} feature/s",
                    &features
                        .iter()
                        .map(|f| f.name.clone())
                        .collect::<Vec<String>>()
                );

                log::debug!(
                    "Found FlowTypes {:?}",
                    &features
                        .iter()
                        .flat_map(|f| f.flow_types.iter().map(|t| t.identifier.clone()))
                        .collect::<Vec<String>>()
                );

                log::debug!(
                    "Found DataTypes {:?}",
                    &features
                        .iter()
                        .flat_map(|f| f.data_types.iter().map(|t| t.identifier.clone()))
                        .collect::<Vec<String>>()
                );

                log::debug!(
                    "Found Functions {:?}",
                    &features
                        .iter()
                        .flat_map(|f| f.functions.iter().map(|t| t.runtime_name.clone()))
                        .collect::<Vec<String>>()
                );

                Ok(features)
            }
            Err(err) => {
                log::error!("Failed to read feature/s from {}, {:?}", &self.path, err);
                Err(ReaderError::ReadFeatureError {
                    path: self.path.to_string(),
                    source: Box::new(err),
                })
            }
        }
    }

    fn read_feature_content(&self, dir: &Path) -> Result<Vec<Feature>, ReaderError> {
        let mut features: Vec<Feature> = Vec::new();

        let readdir = fs::read_dir(dir).map_err(|err| {
            log::error!("Failed to read directory {}: {:?}", dir.display(), err);
            ReaderError::ReadDirectoryError {
                path: dir.to_path_buf(),
                error: err,
            }
        })?;

        for entry_result in readdir {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(err) => {
                    log::error!("Failed to read directory entry: {:?}", err);
                    return Err(ReaderError::DirectoryEntryError(err));
                }
            };

            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let feature_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if !self.accepted_features.is_empty() && !self.accepted_features.contains(&feature_name)
            {
                log::info!("Skipping not accepted feature: {}", feature_name);
                continue;
            }

            let data_types = match self
                .load_definitions_for_feature::<DefinitionDataType>(&path, "data_type")?
            {
                Some(v) => v,
                None => continue,
            };

            let flow_types =
                match self.load_definitions_for_feature::<FlowType>(&path, "flow_type")? {
                    Some(v) => v,
                    None => continue,
                };

            let functions = match self.load_definitions_for_feature::<RuntimeFunctionDefinition>(
                &path,
                "runtime_definition",
            )? {
                Some(v) => v,
                None => continue,
            };

            let feature = Feature {
                name: feature_name,
                data_types,
                flow_types,
                functions,
            };

            features.push(feature);
        }

        Ok(features)
    }

    fn load_definitions_for_feature<T>(
        &self,
        feature_dir: &Path,
        sub_dir: &str,
    ) -> Result<Option<Vec<T>>, ReaderError>
    where
        T: DeserializeOwned + HasVersion,
    {
        let dir = feature_dir.join(sub_dir);

        let raw: Vec<T> = match self.collect_definitions::<T>(&dir) {
            Ok(v) => v,
            Err(err) => {
                if self.should_break {
                    return Err(ReaderError::ReadFeatureError {
                        path: dir.to_string_lossy().to_string(),
                        source: Box::new(err),
                    });
                } else {
                    // Skip this feature if we shouldn't break on error
                    return Ok(None);
                }
            }
        };

        let items = raw
            .into_iter()
            .map(|mut v| {
                v.normalize_version();
                v
            })
            .filter(|v| v.is_accepted(&self.accepted_version))
            .collect();

        Ok(Some(items))
    }

    fn collect_definitions<T>(&self, dir: &Path) -> Result<Vec<T>, ReaderError>
    where
        T: DeserializeOwned,
    {
        let mut definitions = Vec::new();

        if !dir.exists() {
            return Ok(definitions);
        }

        for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                let content = match fs::read_to_string(path) {
                    Ok(content) => content,
                    Err(err) => {
                        log::error!("Failed to read file {}: {}", path.display(), err);
                        return Err(ReaderError::ReadFileError {
                            path: path.to_path_buf(),
                            error: err,
                        });
                    }
                };

                match serde_json::from_str::<T>(&content) {
                    Ok(def) => definitions.push(def),
                    Err(e) => {
                        if self.should_break {
                            log::error!("Failed to parse JSON in file {}: {:?}", path.display(), e);
                            return Err(ReaderError::JsonError {
                                path: path.to_path_buf(),
                                error: e,
                            });
                        } else {
                            log::warn!("Skipping invalid JSON file {}: {:?}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(definitions)
    }
}
