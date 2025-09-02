use super::violation::ContainsKeyRuleViolation;
use super::violation::DataTypeRuleError;
use super::violation::DataTypeRuleViolation;
use super::violation::MissingDataTypeRuleDefinition;
use tucana::shared::ExecutionDataType;
use tucana::shared::ExecutionDataTypeContainsKeyRuleConfig;
use tucana::shared::Value;
use tucana::shared::helper::path::expect_kind;
use tucana::shared::value::Kind;
use crate::flow_validator::{get_data_type_by_id, verify_data_type_rules};

/// # Data Type Validation Behavior
///
/// This function checks if a specific key exists in the JSON body and validates
/// if its value matches the expected data type.
///
/// ## Process:
/// 1. Searches for the specified key in the JSON body
/// 2. If the key is found, retrieves the associated data type definition from the flow
/// 3. Validates that the value matches the expected data type
///
/// ## Error Handling:
/// - Returns a `ContainsKeyRuleViolation` if the specified key is not found in the body
/// - Returns a `MissingDataTypeRuleDefinition` if the referenced data type doesn't exist
/// - Returns validation errors if the value doesn't match the expected data type
pub fn apply_contains_key(
    rule: ExecutionDataTypeContainsKeyRuleConfig,
    body: &Value,
    available_data_types: &Vec<ExecutionDataType>,
) -> Result<(), DataTypeRuleError> {
    let identifier = rule.data_type_identifier;

    if let Some(Kind::StructValue(_)) = &body.kind {
        let value = match expect_kind(&identifier, &body) {
            Some(value) => Value {
                kind: Some(value.to_owned()),
            },
            None => {
                let error = ContainsKeyRuleViolation {
                    missing_key: identifier,
                };

                return Err(DataTypeRuleError {
                    violations: vec![DataTypeRuleViolation::ContainsKey(error)],
                });
            }
        };

        let data_type = match get_data_type_by_id(&available_data_types, &identifier) {
            Some(data_type) => data_type,
            None => {
                let error = MissingDataTypeRuleDefinition {
                    missing_type: identifier,
                };

                return Err(DataTypeRuleError {
                    violations: vec![DataTypeRuleViolation::MissingDataType(error)],
                });
            }
        };

        return verify_data_type_rules(value, data_type, available_data_types);
    } else {
        return Err(DataTypeRuleError {
            violations: vec![DataTypeRuleViolation::ContainsKey(
                ContainsKeyRuleViolation {
                    missing_key: identifier,
                },
            )],
        });
    }
}
