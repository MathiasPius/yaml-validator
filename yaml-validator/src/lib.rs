use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[cfg(test)]
mod tests;

pub mod error;
use error::{Result, *};

trait YamlValidator<'a> {
    fn validate(&'a self, value: &'a Value) -> Result<'a>;
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum DataSigned {
    Signed,
    Unsigned,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataNumber {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sign: Option<DataSigned>,
}

impl<'a> YamlValidator<'a> for DataNumber {
    fn validate(&'a self, value: &'a Value) -> Result<'a> {
        if let Value::Number(_) = value {
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("number", value))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataString {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
}

impl<'a> YamlValidator<'a> for DataString {
    fn validate(&'a self, value: &'a Value) -> Result<'a> {
        if let Value::String(inner) = value {
            if let Some(max_length) = self.max_length {
                if inner.len() > max_length {
                    return Err(StringValidationError::TooLong(max_length, inner.len()).into());
                }
            }

            if let Some(min_length) = self.min_length {
                if inner.len() < min_length {
                    return Err(StringValidationError::TooShort(min_length, inner.len()).into());
                }
            }

            Ok(())
        } else {
            Err(YamlValidationError::WrongType("string", value))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataDictionary {
    pub key: Box<PropertyType>,
    pub value: Box<PropertyType>,
}

impl<'a> YamlValidator<'a> for DataDictionary {
    fn validate(&'a self, value: &'a Value) -> Result<'a> {
        if let Value::Mapping(_) = value {
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("dictionary", value))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataList {
    pub inner: Box<PropertyType>,
}

impl<'a> YamlValidator<'a> for DataList {
    fn validate(&'a self, value: &'a Value) -> Result<'a> {
        if let serde_yaml::Value::Sequence(items) = value {
            for item in items.iter() {
                self.inner.validate(item)?;
            }
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("list", value))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataObject {
    pub fields: Vec<Property>,
}

impl<'a> YamlValidator<'a> for DataObject {
    fn validate(&'a self, value: &'a Value) -> Result<'a> {
        if let Value::Mapping(ref obj) = value {
            for prop in self.fields.iter() {
                if let Some(field) = obj.get(&serde_yaml::to_value(&prop.name).unwrap()) {
                    prop.datatype.validate(field)?
                } else {
                    return Err(YamlValidationError::MissingField(&prop.name));
                }
            }
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("object", value))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase", tag = "type")]
enum PropertyType {
    #[serde(rename = "number")]
    DataNumber(DataNumber),
    #[serde(rename = "string")]
    DataString(DataString),
    #[serde(rename = "list")]
    DataList(DataList),
    #[serde(rename = "dictionary")]
    DataDictionary(DataDictionary),
    #[serde(rename = "object")]
    DataObject(DataObject),
}

impl<'a> YamlValidator<'a> for PropertyType {
    fn validate(&'a self, value: &'a Value) -> Result<'a> {
        match self {
            PropertyType::DataNumber(p) => p.validate(value),
            PropertyType::DataString(p) => p.validate(value),
            PropertyType::DataList(p) => p.validate(value),
            PropertyType::DataDictionary(p) => p.validate(value),
            PropertyType::DataObject(p) => p.validate(value),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
struct Property {
    pub name: String,
    #[serde(flatten)]
    pub datatype: PropertyType,
}

#[derive(Serialize, Deserialize, Debug)]
struct YamlSchema {
    pub schema: Vec<Property>,
}

impl<'a> YamlValidator<'a> for YamlSchema {
    fn validate(&'a self, value: &'a Value) -> Result<'a> {
        if let serde_yaml::Value::Mapping(map) = value {
            for prop in self.schema.iter() {
                if let Some(field) = map.get(&serde_yaml::to_value(&prop.name).unwrap()) {
                    prop.datatype.validate(field)?
                } else {
                    return Ok(());
                }
            }
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("resource definition", value))
        }
    }
}
