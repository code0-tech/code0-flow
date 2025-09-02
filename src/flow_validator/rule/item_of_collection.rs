use super::violation::{DataTypeRuleError, DataTypeRuleViolation, ItemOfCollectionRuleViolation};
use tucana::shared::{DataTypeItemOfCollectionRuleConfig, Value};

/// # Item of Collection Validation
///
/// This function validates if a value is contained within a predefined collection of allowed items.
///
/// ## Process:
/// 1. Checks if the provided value is present in the collection of allowed items
///
/// ## Error Handling:
/// - Returns an `ItemOfCollectionRuleViolation` if the value is not found in the collection
///
pub fn apply_item_of_collection(
    rule: DataTypeItemOfCollectionRuleConfig,
    body: &Value,
    key: &str,
) -> Result<(), DataTypeRuleError> {
    if !rule.items.contains(body) {
        return Err(DataTypeRuleError {
            violations: vec![DataTypeRuleViolation::ItemOfCollection(
                ItemOfCollectionRuleViolation {
                    collection_name: String::from(key),
                },
            )],
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tucana::shared::{ListValue, NullValue, Struct, Value};

    #[test]
    fn test_apply_item_of_collection_success() {
        let value = Value {
            kind: Some(tucana::shared::value::Kind::StringValue(
                "allowed_value".to_string(),
            )),
        };
        let items = vec![
            Value {
                kind: Some(tucana::shared::value::Kind::StringValue(
                    "allowed_value".to_string(),
                )),
            },
            Value {
                kind: Some(tucana::shared::value::Kind::StringValue(
                    "another_allowed_value".to_string(),
                )),
            },
        ];

        let rule = DataTypeItemOfCollectionRuleConfig { items };
        let result = apply_item_of_collection(rule, &value, "test_field");

        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_item_of_collection_failure() {
        let value = Value {
            kind: Some(tucana::shared::value::Kind::StringValue(
                "disallowed_value".to_string(),
            )),
        };
        let items = vec![
            Value {
                kind: Some(tucana::shared::value::Kind::StringValue(
                    "allowed_value".to_string(),
                )),
            },
            Value {
                kind: Some(tucana::shared::value::Kind::StringValue(
                    "another_allowed_value".to_string(),
                )),
            },
        ];

        let rule = DataTypeItemOfCollectionRuleConfig { items };
        let result = apply_item_of_collection(rule, &value, "test_field");

        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.violations.len(), 1);
            match &error.violations[0] {
                DataTypeRuleViolation::ItemOfCollection(violation) => {
                    assert_eq!(violation.collection_name, "test_field");
                }
                _ => panic!("Expected ItemOfCollection violation"),
            }
        }
    }

    #[test]
    fn test_apply_item_of_collection_with_different_value_types() {
        let value = Value {
            kind: Some(tucana::shared::value::Kind::NumberValue(42.0)),
        };
        let items = vec![
            Value {
                kind: Some(tucana::shared::value::Kind::StringValue(
                    "allowed_value".to_string(),
                )),
            },
            Value {
                kind: Some(tucana::shared::value::Kind::NumberValue(42.0)),
            },
        ];

        let rule = DataTypeItemOfCollectionRuleConfig { items };
        let result = apply_item_of_collection(rule, &value, "test_field");

        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_item_of_collection_with_complex_values() {
        let mut fields = HashMap::new();
        fields.insert(
            "name".to_string(),
            Value {
                kind: Some(tucana::shared::value::Kind::StringValue("test".to_string())),
            },
        );
        fields.insert(
            "age".to_string(),
            Value {
                kind: Some(tucana::shared::value::Kind::NumberValue(30.0)),
            },
        );
        let struct_value = Value {
            kind: Some(tucana::shared::value::Kind::StructValue(Struct { fields })),
        };

        let list_values = vec![
            Value {
                kind: Some(tucana::shared::value::Kind::StringValue("one".to_string())),
            },
            Value {
                kind: Some(tucana::shared::value::Kind::NumberValue(2.0)),
            },
        ];
        let list_value = Value {
            kind: Some(tucana::shared::value::Kind::ListValue(ListValue {
                values: list_values,
            })),
        };

        let null_value = Value {
            kind: Some(tucana::shared::value::Kind::NullValue(
                NullValue::NullValue as i32,
            )),
        };

        let items = vec![struct_value.clone(), list_value.clone(), null_value.clone()];
        let rule = DataTypeItemOfCollectionRuleConfig { items };

        assert!(apply_item_of_collection(rule.clone(), &struct_value, "test_field").is_ok());
        assert!(apply_item_of_collection(rule.clone(), &list_value, "test_field").is_ok());
        assert!(apply_item_of_collection(rule, &null_value, "test_field").is_ok());

        let mut different_fields = HashMap::new();
        different_fields.insert(
            "different".to_string(),
            Value {
                kind: Some(tucana::shared::value::Kind::BoolValue(true)),
            },
        );
        let different_struct = Value {
            kind: Some(tucana::shared::value::Kind::StructValue(Struct {
                fields: different_fields,
            })),
        };

        let rule_for_failure = DataTypeItemOfCollectionRuleConfig {
            items: vec![struct_value],
        };
        assert!(
            apply_item_of_collection(rule_for_failure, &different_struct, "test_field").is_err()
        );
    }
}
