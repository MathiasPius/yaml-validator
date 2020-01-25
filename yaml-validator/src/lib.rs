use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[cfg(test)]
mod tests;

pub mod error;
use error::{Result, *};

pub trait YamlValidator<'a> {
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
            Err(YamlValidationError::WrongType("number", value).into())
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
            Err(YamlValidationError::WrongType("string", value).into())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataDictionary {
    pub key: Option<Box<PropertyType>>,
    pub value: Option<Box<PropertyType>>,
}

impl<'a> YamlValidator<'a> for DataDictionary {
    fn validate(&'a self, value: &'a Value) -> Result<'a> {
        if let Value::Mapping(dict) = value {
            for item in dict.iter() {
                if let Some(ref key) = self.key {
                    key.validate(item.0).prepend(format!(".{}", item.0.as_str().unwrap_or("<non-string field>")))?;
                }

                if let Some(ref value) = self.value {
                    value.validate(item.1).prepend(format!(".{}", item.0.as_str().unwrap_or("<non-string field>")))?;
                }
            }
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("dictionary", value).into())
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
            for (i, item) in items.iter().enumerate() {
                self.inner.validate(item).prepend(format!("[{}]", i))?;
            }
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("list", value).into())
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
                    prop.datatype.validate(field).prepend(format!(".{}", prop.name))?
                } else {
                    return Err(YamlValidationError::MissingField(&prop.name).into());
                }
            }
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("object", value).into())
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
pub struct YamlSchema {
    schema: Vec<Property>,
}

impl YamlSchema {
    pub fn from_str(schema: &str) -> YamlSchema {
        serde_yaml::from_str(schema).expect("failed to parse string as yaml")
    }

    pub fn validate_str(&self, yaml: &str) -> std::result::Result<(), String> {
        match self.validate(&serde_yaml::from_str(yaml).expect("failed to parse string as yaml")) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("{}", e)),
        }
    }
}

impl<'a> YamlValidator<'a> for YamlSchema {
    fn validate(&'a self, value: &'a Value) -> Result<'a> {
        if let serde_yaml::Value::Mapping(map) = value {
            for prop in self.schema.iter() {
                if let Some(field) = map.get(&serde_yaml::to_value(&prop.name).unwrap()) {
                    prop.datatype.validate(field).prepend(prop.name.clone())?
                } else {
                    return Ok(());
                }
            }
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("resource definition", value).into())
        }
    }
}
