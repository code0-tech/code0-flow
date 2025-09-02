use tucana::shared::{DataTypeRegexRuleConfig, Value, value::Kind};
use super::violation::{
    DataTypeRuleError, DataTypeRuleViolation, RegexRuleTypeNotAcceptedViolation, RegexRuleViolation,
};

/// # Regex Pattern Validation
///
/// This function validates if a value matches a specified regex pattern.
///
/// ## Process:
/// 1. Converts the input value to a string representation (if possible)
/// 2. Compiles the regex pattern from the rule
/// 3. Checks if the string representation matches the regex pattern
///
/// ## Error Handling:
/// - Returns a `RegexRuleTypeNotAcceptedViolation` if the value type cannot be converted to a string
///   (e.g., arrays, objects)
/// - Returns a `RegexRuleViolation` if the string representation does not match the specified pattern
///
pub fn apply_regex(rule: DataTypeRegexRuleConfig, body: &Value) -> Result<(), DataTypeRuleError> {
    let kind = match &body.kind {
        Some(kind) => kind,
        None => return Ok(()),
    };

    let result = match kind {
        Kind::BoolValue(b) => b.to_string(),
        Kind::NumberValue(n) => n.to_string(),
        Kind::StringValue(s) => s.clone(),
        Kind::NullValue(_) => "null".to_string(),
        Kind::StructValue(s) => {
            return Err(DataTypeRuleError {
                violations: vec![DataTypeRuleViolation::RegexTypeNotAccepted(
                    RegexRuleTypeNotAcceptedViolation {
                        type_not_accepted: format!("StructValue({:?})", s),
                    },
                )],
            });
        }
        Kind::ListValue(l) => {
            return Err(DataTypeRuleError {
                violations: vec![DataTypeRuleViolation::RegexTypeNotAccepted(
                    RegexRuleTypeNotAcceptedViolation {
                        type_not_accepted: format!("ListValue({:?})", l),
                    },
                )],
            });
        }
    };

    let regex = regex::Regex::new(rule.pattern.as_str()).unwrap();

    if !regex.is_match(&result) {
        return Err(DataTypeRuleError {
            violations: vec![DataTypeRuleViolation::Regex(RegexRuleViolation {
                missing_regex: rule.pattern.clone(),
            })],
        });
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tucana::shared::{ListValue, Struct};

    #[test]
    fn test_apply_regex_with_matching_string() {
        let rule = DataTypeRegexRuleConfig {
            pattern: String::from("^[a-z]+$"),
        };
        let value = Value {
            kind: Some(Kind::StringValue(String::from("abcde"))),
        };

        assert!(apply_regex(rule, &value).is_ok());
    }

    #[test]
    fn test_apply_regex_with_non_matching_string() {
        let rule = DataTypeRegexRuleConfig {
            pattern: String::from("^[a-z]+$"),
        };
        let value = Value {
            kind: Some(Kind::StringValue(String::from("123"))),
        };

        let result = apply_regex(rule, &value);
        assert!(result.is_err());

        if let Err(DataTypeRuleError { violations }) = result {
            assert_eq!(violations.len(), 1);
            match &violations[0] {
                DataTypeRuleViolation::Regex(violation) => {
                    assert_eq!(violation.missing_regex, "^[a-z]+$");
                }
                _ => panic!("Expected RegexRuleViolation"),
            }
        }
    }

    #[test]
    fn test_apply_regex_with_matching_boolean() {
        let rule = DataTypeRegexRuleConfig {
            pattern: String::from("^true$"),
        };
        let value = Value {
            kind: Some(Kind::BoolValue(true)),
        };

        assert!(apply_regex(rule, &value).is_ok());
    }

    #[test]
    fn test_apply_regex_with_non_matching_boolean() {
        let rule = DataTypeRegexRuleConfig {
            pattern: String::from("^false$"),
        };
        let value = Value {
            kind: Some(Kind::BoolValue(true)),
        };

        assert!(apply_regex(rule, &value).is_err());
    }

    #[test]
    fn test_apply_regex_with_matching_number() {
        let rule = DataTypeRegexRuleConfig {
            pattern: String::from("^42$"),
        };
        let value = Value {
            kind: Some(Kind::NumberValue(42.0)),
        };

        assert!(apply_regex(rule, &value).is_ok());
    }

    #[test]
    fn test_apply_regex_with_non_matching_number() {
        let rule = DataTypeRegexRuleConfig {
            pattern: String::from("^[0-9]+$"),
        };
        let value = Value {
            kind: Some(Kind::NumberValue(3.14)),
        };

        assert!(apply_regex(rule, &value).is_err());
    }

    #[test]
    fn test_apply_regex_with_array() {
        let rule = DataTypeRegexRuleConfig {
            pattern: String::from(".*"),
        };
        let value = Value {
            kind: Some(Kind::ListValue(ListValue { values: vec![] })),
        };

        let result = apply_regex(rule, &value);
        assert!(result.is_err());

        if let Err(DataTypeRuleError { violations }) = result {
            assert_eq!(violations.len(), 1);
            match &violations[0] {
                DataTypeRuleViolation::RegexTypeNotAccepted(violation) => {
                    assert!(violation.type_not_accepted.contains("ListValue"));
                }
                _ => panic!("Expected RegexRuleTypeNotAcceptedViolation"),
            }
        }
    }

    #[test]
    fn test_apply_regex_with_object() {
        let rule = DataTypeRegexRuleConfig {
            pattern: String::from(".*"),
        };
        let value = Value {
            kind: Some(Kind::StructValue(Struct {
                fields: Default::default(),
            })),
        };

        let result = apply_regex(rule, &value);
        assert!(result.is_err());

        if let Err(DataTypeRuleError { violations }) = result {
            assert_eq!(violations.len(), 1);
            match &violations[0] {
                DataTypeRuleViolation::RegexTypeNotAccepted(violation) => {
                    assert!(violation.type_not_accepted.contains("StructValue"));
                }
                _ => panic!("Expected RegexRuleTypeNotAcceptedViolation"),
            }
        }
    }

    #[test]
    fn test_apply_regex_with_null_kind() {
        let rule = DataTypeRegexRuleConfig {
            pattern: String::from(".*"),
        };
        let value = Value { kind: None };

        assert!(apply_regex(rule, &value).is_ok());
    }

    #[test]
    fn test_apply_regex_complex_pattern() {
        let rule = DataTypeRegexRuleConfig {
            pattern: String::from(r"^\d{3}-\d{2}-\d{4}$"), // SSN pattern
        };
        let value = Value {
            kind: Some(Kind::StringValue(String::from("123-45-6789"))),
        };

        assert!(apply_regex(rule.clone(), &value).is_ok());

        let invalid_value = Value {
            kind: Some(Kind::StringValue(String::from("123-456-789"))),
        };

        assert!(apply_regex(rule, &invalid_value).is_err());
    }

    #[test]
    fn test_apply_regex_email_pattern() {
        let rule = DataTypeRegexRuleConfig {
            pattern: String::from(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"),
        };
        let value = Value {
            kind: Some(Kind::StringValue(String::from("test@example.com"))),
        };

        assert!(apply_regex(rule.clone(), &value).is_ok());

        let invalid_value = Value {
            kind: Some(Kind::StringValue(String::from("invalid-email"))),
        };

        assert!(apply_regex(rule, &invalid_value).is_err());
    }
}
