pub struct DataTypeRuleError {
    pub violations: Vec<DataTypeRuleViolation>,
}

pub enum DataTypeRuleViolation {
    MissingDataType(MissingDataTypeRuleDefinition),
    ContainsKey(ContainsKeyRuleViolation),
    Regex(RegexRuleViolation),
    RegexTypeNotAccepted(RegexRuleTypeNotAcceptedViolation),
    DataTypeNotFound(DataTypeNotFoundRuleViolation),
    NumberInRange(NumberInRangeRuleViolation),
    ItemOfCollection(ItemOfCollectionRuleViolation),
    InvalidFormat(InvalidFormatRuleViolation),
    GenericKeyNotAllowed(GenericKeyNotAllowedRuleViolation),
    DataTypeIdentifierNotPresent(DataTypeIdentifierNotPresentRuleViolation),
}

pub struct MissingDataTypeRuleDefinition {
    pub missing_type: String,
}

pub struct ContainsKeyRuleViolation {
    pub missing_key: String,
}

pub struct RegexRuleViolation {
    pub missing_regex: String,
}

pub struct RegexRuleTypeNotAcceptedViolation {
    pub type_not_accepted: String,
}

pub struct DataTypeNotFoundRuleViolation {
    pub data_type: String,
}

pub struct NumberInRangeRuleViolation {
    pub key: String,
}

pub struct ItemOfCollectionRuleViolation {
    pub collection_name: String,
}

pub struct InvalidFormatRuleViolation {
    pub expected_format: String,
    pub value: String,
}

pub struct GenericKeyNotAllowedRuleViolation {
    pub key: String,
}

pub struct DataTypeIdentifierNotPresentRuleViolation {
    pub identifier: String,
}

impl DataTypeRuleError {
    pub fn to_string(&self) -> String {
        let mut violations = Vec::new();

        for violation in &self.violations {
            match violation {
                DataTypeRuleViolation::ContainsKey(v) => {
                    violations.push(serde_json::json!({
                        "type": "ContainsKey",
                        "explanation": format!("Missing required key: '{}'", v.missing_key),
                        "details": {
                            "missing_key": v.missing_key
                        }
                    }));
                }
                DataTypeRuleViolation::Regex(v) => {
                    violations.push(serde_json::json!({
                        "type": "Regex",
                        "explanation": format!("Failed to match regex pattern: '{}'", v.missing_regex),
                        "details": {
                            "missing_regex": v.missing_regex
                        }
                    }));
                }
                DataTypeRuleViolation::MissingDataType(v) => {
                    violations.push(serde_json::json!({
                        "type": "MissingDataType",
                        "explanation": format!("Missing required data type: '{}'", v.missing_type),
                        "details": {
                            "missing_type": v.missing_type
                        }
                    }));
                }
                DataTypeRuleViolation::RegexTypeNotAccepted(v) => {
                    violations.push(serde_json::json!({
                        "type": "RegexTypeNotAccepted",
                        "explanation": format!("Regex pattern does not match data type: '{}'", v.type_not_accepted),
                        "details": {
                            "type_not_accepted": v.type_not_accepted
                        }
                    }));
                }
                DataTypeRuleViolation::DataTypeNotFound(v) => {
                    violations.push(serde_json::json!({
                        "type": "DataTypeNotFound",
                        "explanation": format!("Data type not found: '{}'", v.data_type),
                        "details": {
                            "data_type": v.data_type
                        }
                    }));
                }
                DataTypeRuleViolation::NumberInRange(v) => {
                    violations.push(serde_json::json!({
                        "type": "NumberInRange",
                        "explanation": format!("Number not in valid range for key: '{}'", v.key),
                        "details": {
                            "key": v.key
                        }
                    }));
                }
                DataTypeRuleViolation::ItemOfCollection(v) => {
                    violations.push(serde_json::json!({
                        "type": "ItemOfCollection",
                        "explanation": format!("Item is not a valid member of collection: '{}'", v.collection_name),
                        "details": {
                            "collection_name": v.collection_name
                        }
                    }));
                }
                DataTypeRuleViolation::InvalidFormat(v) => {
                    violations.push(serde_json::json!({
                        "type": "InvalidFormat",
                        "explanation": format!("Invalid format. Expected: '{}', Got: '{}'", v.expected_format, v.value),
                        "details": {
                            "expected_format": v.expected_format,
                            "value": v.value
                        }
                    }));
                }
                DataTypeRuleViolation::GenericKeyNotAllowed(v) => {
                    violations.push(serde_json::json!({
                        "type": "GenericKeyNotAllowed",
                        "explanation": format!("Generic key not allowed: '{}'", v.key),
                        "details": {
                            "key": v.key
                        }
                    }));
                }
                DataTypeRuleViolation::DataTypeIdentifierNotPresent(v) => {
                    violations.push(serde_json::json!({
                        "type": "DataTypeIdentifierNotPresent",
                        "explanation": format!("Data type identifier not present: '{}'", v.identifier),
                        "details": {
                            "identifier": v.identifier
                        }
                    }));
                }
            }
        }

        serde_json::json!({
            "error": "DataTypeRuleError",
            "violation_count": self.violations.len(),
            "violations": violations
        })
        .to_string()
    }
}
