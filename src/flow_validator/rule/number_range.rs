use tucana::shared::{DataTypeNumberRangeRuleConfig, Value, value::Kind};

use super::violation::{
    DataTypeRuleError, DataTypeRuleViolation, NumberInRangeRuleViolation,
    RegexRuleTypeNotAcceptedViolation,
};

/// # Number Range Validation
///
/// This function validates if a numeric value falls within a specified range and follows step constraints.
///
/// ## Process:
/// 1. Extracts the numeric value from the input (if it is a number)
/// 2. Checks if the number is within the specified range (from/to)
/// 3. If steps are specified, verifies the number is divisible by the step value
///
/// ## Error Handling:
/// - Returns a `RegexRuleTypeNotAcceptedViolation` if the value is not a number
/// - Returns a `NumberInRangeRuleViolation` if the number is outside the specified range
/// - Returns a `NumberInRangeRuleViolation` if the number doesn't conform to the step constraint
///
pub fn apply_number_range(
    rule: DataTypeNumberRangeRuleConfig,
    body: &Value,
    key: &str,
) -> Result<(), DataTypeRuleError> {
    let kind = match &body.kind {
        Some(kind) => kind,
        None => return Ok(()),
    };

    let result = match kind {
        Kind::NumberValue(n) => n.clone(),
        _ => {
            return Err(DataTypeRuleError {
                violations: vec![DataTypeRuleViolation::RegexTypeNotAccepted(
                    RegexRuleTypeNotAcceptedViolation {
                        type_not_accepted: format!("{:?}", kind),
                    },
                )],
            });
        }
    };

    if result < rule.from as f64 || result > rule.to as f64 {
        return Err(DataTypeRuleError {
            violations: vec![DataTypeRuleViolation::NumberInRange(
                NumberInRangeRuleViolation {
                    key: String::from(key),
                },
            )],
        });
    }

    if let Some(modulo) = rule.steps {
        if modulo == 0 {
            return Ok(());
        }

        if result % modulo as f64 != 0.0 {
            return Err(DataTypeRuleError {
                violations: vec![DataTypeRuleViolation::NumberInRange(
                    NumberInRangeRuleViolation {
                        key: String::from(key),
                    },
                )],
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn number_as_value(number: f64) -> Value {
        Value {
            kind: Some(Kind::NumberValue(number)),
        }
    }

    #[test]
    fn test_apply_number_range() {
        let rule = DataTypeNumberRangeRuleConfig {
            from: 1,
            to: 10,
            steps: None,
        };

        assert!(apply_number_range(rule, &number_as_value(-2.0), "test").is_err());
        assert!(apply_number_range(rule, &number_as_value(2.0), "test").is_ok());
        assert!(apply_number_range(rule, &number_as_value(3.0), "test").is_ok());
        assert!(apply_number_range(rule, &number_as_value(11.0), "test").is_err());
        assert!(apply_number_range(rule, &number_as_value(12.0), "test").is_err());
    }

    #[test]
    fn test_apply_number_range_with_steps() {
        let rule = DataTypeNumberRangeRuleConfig {
            from: 1,
            to: 10,
            steps: Some(2),
        };

        assert!(apply_number_range(rule, &number_as_value(2.0), "test").is_ok());
        assert!(apply_number_range(rule, &number_as_value(4.0), "test").is_ok());
        assert!(apply_number_range(rule, &number_as_value(6.0), "test").is_ok());
        assert!(apply_number_range(rule, &number_as_value(8.0), "test").is_ok());
        assert!(apply_number_range(rule, &number_as_value(10.0), "test").is_ok());
        assert!(apply_number_range(rule, &number_as_value(1.0), "test").is_err());
        assert!(apply_number_range(rule, &number_as_value(3.0), "test").is_err());
        assert!(apply_number_range(rule, &number_as_value(5.0), "test").is_err());
        assert!(apply_number_range(rule, &number_as_value(7.0), "test").is_err());
        assert!(apply_number_range(rule, &number_as_value(9.0), "test").is_err());
        assert!(apply_number_range(rule, &number_as_value(11.0), "test").is_err());
        assert!(apply_number_range(rule, &number_as_value(12.0), "test").is_err());
        assert!(apply_number_range(rule, &number_as_value(-12.0), "test").is_err());
    }

    #[test]
    fn test_apply_number_range_with_falty_steps() {
        let rule = DataTypeNumberRangeRuleConfig {
            from: 1,
            to: 10,
            steps: Some(0),
        };

        assert!(apply_number_range(rule, &number_as_value(-12.0), "test").is_err());
        assert!(apply_number_range(rule, &number_as_value(12.0), "test").is_err());
        assert!(apply_number_range(rule, &number_as_value(6.0), "test").is_ok());
    }
}
