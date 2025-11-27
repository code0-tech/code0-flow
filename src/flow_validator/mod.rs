mod rule;

use rule::{
    contains_key::apply_contains_key,
    contains_type::apply_contains_type,
    item_of_collection::apply_item_of_collection,
    number_range::apply_number_range,
    regex::apply_regex,
    violation::{DataTypeNotFoundRuleViolation, DataTypeRuleError, DataTypeRuleViolation},
};

use tucana::shared::{ExecutionDataType, ValidationFlow, Value, execution_data_type_rule::Config};
pub struct VerificationResult;

pub fn verify_flow(flow: ValidationFlow, body: Value) -> Result<(), DataTypeRuleError> {
    let input_type = match &flow.input_type_identifier {
        Some(r) => r.clone(),
        None => return Ok(()), //Returns directly because no rule is given. The body is ok and will not be concidered
    };

    let data_type = match flow
        .data_types
        .iter()
        .find(|dt| dt.identifier == input_type)
    {
        Some(dt) => dt.clone(),
        None => {
            return Err(DataTypeRuleError {
                violations: vec![DataTypeRuleViolation::DataTypeNotFound(
                    DataTypeNotFoundRuleViolation {
                        data_type: input_type,
                    },
                )],
            });
        }
    };

    verify_data_type_rules(body, data_type, &flow.data_types)
}

//Verifies the rules on the datatype of the body thats given
fn verify_data_type_rules(
    body: Value,
    data_type: ExecutionDataType,
    availabe_data_types: &Vec<ExecutionDataType>,
) -> Result<(), DataTypeRuleError> {
    let mut violations: Vec<DataTypeRuleViolation> = Vec::new();
    for rule in data_type.rules {
        let rule_config = match rule.config {
            None => continue,
            Some(config) => config,
        };

        match rule_config {
            Config::NumberRange(config) => {
                match apply_number_range(config, &body, &String::from("value")) {
                    Ok(_) => continue,
                    Err(violation) => {
                        violations.extend(violation.violations);
                        continue;
                    }
                }
            }
            Config::ItemOfCollection(config) => {
                match apply_item_of_collection(config, &body, "key") {
                    Ok(_) => continue,
                    Err(violation) => {
                        violations.extend(violation.violations);
                        continue;
                    }
                }
            }
            Config::ContainsType(config) => {
                match apply_contains_type(config, availabe_data_types, &body) {
                    Ok(_) => continue,
                    Err(violation) => {
                        violations.extend(violation.violations);
                        continue;
                    }
                }
            }
            Config::Regex(config) => {
                match apply_regex(config, &body) {
                    Ok(_) => continue,
                    Err(violation) => {
                        violations.extend(violation.violations);
                        continue;
                    }
                };
            }
            Config::ContainsKey(config) => {
                match apply_contains_key(config, &body, availabe_data_types) {
                    Ok(_) => continue,
                    Err(violation) => {
                        violations.extend(violation.violations);
                        continue;
                    }
                };
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(DataTypeRuleError { violations })
    }
}

fn get_data_type_by_id(
    data_types: &Vec<ExecutionDataType>,
    identifier: &String,
) -> Option<ExecutionDataType> {
    data_types
        .iter()
        .find(|data_type| &data_type.identifier == identifier)
        .cloned()
}
