use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[cfg(test)]
mod tests;

pub mod error;
use error::{Result, *};

pub trait YamlValidator<'a> {
    fn validate(&'a self, value: &'a Value, context: Option<&'a YamlContext>) -> Result<'a>;
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
    fn validate(&'a self, value: &'a Value, _: Option<&'a YamlContext>) -> Result<'a> {
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
    fn validate(&'a self, value: &'a Value, _: Option<&'a YamlContext>) -> Result<'a> {
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
struct DataReference {
    pub uri: String,
}

impl<'a> YamlValidator<'a> for DataReference {
    fn validate(&'a self, value: &'a Value, context: Option<&'a YamlContext>) -> Result<'a> {
        if let Some(ctx) = context {
            if let Some(schema) = ctx.lookup(&self.uri) {
                return DataObject::validate(&schema.schema, value, context);
            }
            return Err(YamlValidationError::MissingSchema(&self.uri).into());
        }
        Err(YamlValidationError::MissingContext.into())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataDictionary {
    pub key: Option<Box<PropertyType>>,
    pub value: Option<Box<PropertyType>>,
}

impl<'a> YamlValidator<'a> for DataDictionary {
    fn validate(&'a self, value: &'a Value, context: Option<&'a YamlContext>) -> Result<'a> {
        if let Value::Mapping(dict) = value {
            for item in dict.iter() {
                if let Some(ref key) = self.key {
                    key.validate(item.0, context).prepend(format!(
                        ".{}",
                        item.0.as_str().unwrap_or("<non-string field>")
                    ))?;
                }

                if let Some(ref value) = self.value {
                    value.validate(item.1, context).prepend(format!(
                        ".{}",
                        item.0.as_str().unwrap_or("<non-string field>")
                    ))?;
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
    fn validate(&'a self, value: &'a Value, context: Option<&'a YamlContext>) -> Result<'a> {
        if let serde_yaml::Value::Sequence(items) = value {
            for (i, item) in items.iter().enumerate() {
                self.inner
                    .validate(item, context)
                    .prepend(format!("[{}]", i))?;
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

impl DataObject {
    pub fn validate<'a>(
        properties: &'a Vec<Property>,
        value: &'a Value,
        context: Option<&'a YamlContext>,
    ) -> Result<'a> {
        if let Value::Mapping(ref obj) = value {
            for prop in properties.iter() {
                if let Some(field) = obj.get(&serde_yaml::to_value(&prop.name).unwrap()) {
                    prop.datatype
                        .validate(field, context)
                        .prepend(format!(".{}", prop.name))?
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

impl<'a> YamlValidator<'a> for DataObject {
    fn validate(&'a self, value: &'a Value, context: Option<&'a YamlContext>) -> Result<'a> {
        DataObject::validate(&self.fields, value, context)
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
    #[serde(rename = "reference")]
    DataReference(DataReference),
}

impl<'a> YamlValidator<'a> for PropertyType {
    fn validate(&'a self, value: &'a Value, context: Option<&'a YamlContext>) -> Result<'a> {
        match self {
            PropertyType::DataNumber(p) => p.validate(value, context),
            PropertyType::DataString(p) => p.validate(value, context),
            PropertyType::DataList(p) => p.validate(value, context),
            PropertyType::DataDictionary(p) => p.validate(value, context),
            PropertyType::DataObject(p) => p.validate(value, context),
            PropertyType::DataReference(p) => p.validate(value, context),
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
    uri: Option<String>,
    schema: Vec<Property>,
}

impl YamlSchema {
    pub fn from_str(schema: &str) -> YamlSchema {
        serde_yaml::from_str(schema).expect("failed to parse string as yaml")
    }

    pub fn validate_str(
        &self,
        yaml: &str,
        context: Option<&YamlContext>,
    ) -> std::result::Result<(), String> {
        match self.validate(
            &serde_yaml::from_str(yaml).expect("failed to parse string as yaml"),
            context,
        ) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("{}", e)),
        }
    }
}

impl<'a> YamlValidator<'a> for YamlSchema {
    fn validate(&'a self, value: &'a Value, context: Option<&'a YamlContext>) -> Result<'a> {
        DataObject::validate(&self.schema, value, context).prepend("$".into())
    }
}

#[derive(Debug)]
pub struct YamlContext {
    schemas: Vec<YamlSchema>,
}

impl YamlContext {
    pub fn new() -> Self {
        YamlContext { schemas: vec![] }
    }

    pub fn from_schemas(schemas: Vec<YamlSchema>) -> Self {
        YamlContext {
            schemas: schemas.into(),
        }
    }

    pub fn add_schema(&mut self, schema: YamlSchema) {
        self.schemas.push(schema);
    }

    pub fn lookup(&self, uri: &str) -> Option<&YamlSchema> {
        for schema in self.schemas.iter() {
            if let Some(ref schema_uri) = schema.uri {
                if schema_uri == uri {
                    return Some(&schema);
                }
            }
        }
        return None;
    }

    pub fn schemas(&self) -> &Vec<YamlSchema> {
        &self.schemas
    }
}
