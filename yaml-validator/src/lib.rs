use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[cfg(test)]
mod tests;

mod error;
pub use error::YamlSchemaError;
use error::{ValidationResult, *};

trait YamlValidator<'a> {
    fn validate(
        &'a self,
        value: &'a Value,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a>;
}

#[serde(deny_unknown_fields)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataNumber {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i128>,
}

impl<'a> YamlValidator<'a> for DataNumber {
    fn validate(&'a self, value: &'a Value, _: Option<&'a YamlContext>) -> ValidationResult<'a> {
        if let Value::Number(_) = value {
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("number", value).into())
        }
    }
}

#[serde(deny_unknown_fields)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataString {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
}

impl<'a> YamlValidator<'a> for DataString {
    fn validate(&'a self, value: &'a Value, _: Option<&'a YamlContext>) -> ValidationResult<'a> {
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

impl TryFrom<Yaml> for DataString {
    type Error = StatefulResult<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let mut yaml = yaml.into_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("datastring is not an object").into()
        })?;

        let typefield = yaml.remove(&Yaml::String("type".into())).ok_or_else(|| {
            YamlSchemaError::SchemaParsingError(
                "attempting to parse element as string, but object contains no 'type' field",
            )
            .into()
        })?;
        let typename = typefield.as_str().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError(
                "attempting to parse element as string, but object's type field is not a string",
            )
            .into()
        })?;

        if typename != "string" {
            return Err(YamlSchemaError::SchemaParsingError(
                "attempting to parse element as string, but object's type field is not 'string'",
            )
            .into());
        }

        let max_length = if let Some(max_length) = yaml.remove(&Yaml::String("max_length".into())) {
            Some(max_length.as_i64().ok_or_else(|| {
                YamlSchemaError::SchemaParsingError("max_length must be an integer").into()
            })? as usize)
        } else {
            None
        };

        let min_length = if let Some(min_length) = yaml.remove(&Yaml::String("min_length".into())) {
            Some(min_length.as_i64().ok_or_else(|| {
                YamlSchemaError::SchemaParsingError("min_length must be an integer").into()
            })? as usize)
        } else {
            None
        };

        if !yaml.is_empty() {
            return Err(YamlSchemaError::SchemaParsingError(
                "string element contains superfluous elements",
            )
            .into());
        }

        Ok(DataString {
            max_length,
            min_length,
        })
    }
}

#[serde(deny_unknown_fields)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataReference {
    pub uri: String,
}

impl<'a> YamlValidator<'a> for DataReference {
    fn validate(
        &'a self,
        value: &'a Value,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
        if let Some(ctx) = context {
            if let Some(schema) = ctx.lookup(&self.uri) {
                return DataObject::validate(&schema.schema, value, context);
            }
            return Err(YamlValidationError::MissingSchema(&self.uri).into());
        }
        Err(YamlValidationError::MissingContext.into())
    }
}

#[serde(deny_unknown_fields)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataDictionary {
    pub value: Option<Box<PropertyType>>,
}

impl<'a> YamlValidator<'a> for DataDictionary {
    fn validate(
        &'a self,
        value: &'a Value,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
        if let Value::Mapping(dict) = value {
            for item in dict.iter() {
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

#[serde(deny_unknown_fields)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataList {
    pub inner: Box<PropertyType>,
}

impl<'a> YamlValidator<'a> for DataList {
    fn validate(
        &'a self,
        value: &'a Value,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
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

#[serde(deny_unknown_fields)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DataObject {
    pub fields: Vec<Property>,
}

impl DataObject {
    pub fn validate<'a>(
        properties: &'a [Property],
        value: &'a Value,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
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
    fn validate(
        &'a self,
        value: &'a Value,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
        DataObject::validate(&self.fields, value, context)
    }
}

#[serde(deny_unknown_fields)]
#[serde(rename_all = "lowercase", tag = "type")]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum PropertyType {
    #[serde(rename = "number")]
    Number(DataNumber),
    #[serde(rename = "string")]
    String(DataString),
    #[serde(rename = "list")]
    List(DataList),
    #[serde(rename = "dictionary")]
    Dictionary(DataDictionary),
    #[serde(rename = "object")]
    Object(DataObject),
    #[serde(rename = "reference")]
    Reference(DataReference),
}

impl<'a> YamlValidator<'a> for PropertyType {
    fn validate(
        &'a self,
        value: &'a Value,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
        match self {
            PropertyType::Number(p) => p.validate(value, context),
            PropertyType::String(p) => p.validate(value, context),
            PropertyType::List(p) => p.validate(value, context),
            PropertyType::Dictionary(p) => p.validate(value, context),
            PropertyType::Object(p) => p.validate(value, context),
            PropertyType::Reference(p) => p.validate(value, context),
        }
    }
}

#[serde(rename_all = "lowercase")]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Property {
    pub name: String,
    #[serde(flatten)]
    pub datatype: PropertyType,
}

/// Struct containing a list of internal properties as defined in its top-level `schema` field
#[derive(Serialize, Deserialize, Debug)]
pub struct YamlSchema {
    uri: Option<String>,
    schema: Vec<Property>,
}

impl YamlSchema {
    /// Validate a single yaml document against this schema
    /// # Examples
    /// This example specifies a single schema, and validates two separate yaml documents against it
    /// ```rust
    /// # use yaml_validator::YamlSchema;
    /// # use std::str::FromStr;
    /// #
    /// let schema = YamlSchema::from_str(r#"
    /// ---
    /// schema:
    ///   - name: firstname
    ///     type: string
    /// "#).unwrap();
    ///
    /// assert!(schema.validate_str("firstname: John", None).is_ok());
    /// assert!(!schema.validate_str("lastname: Smith", None).is_ok())
    /// ```
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

impl TryFrom<Yaml> for YamlSchema {
    type Error = yaml_rust::ScanError;

    fn try_from(yaml: Yaml) -> Result<YamlSchema, Self::Error> {
        if let Some(mut yaml) = yaml.into_hash() {
            let uri = yaml
                .remove(&Yaml::String("uri".to_owned()))
                .and_then(|uri| uri.into_string());

            let schema = yaml
                .remove(&Yaml::String("schema".to_owned()))
                .map(|_| vec![])
                .unwrap();

            return Ok(YamlSchema { uri, schema });
        }

        Ok(YamlSchema {
            uri: None,
            schema: vec![],
        })
    }
}

/// Can I add comments to implementations?
impl std::str::FromStr for YamlSchema {
    type Err = serde_yaml::Error;
    fn from_str(schema: &str) -> std::result::Result<YamlSchema, Self::Err> {
        serde_yaml::from_str(schema)
    }
}

impl<'a> YamlValidator<'a> for YamlSchema {
    fn validate(
        &'a self,
        value: &'a Value,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
        DataObject::validate(&self.schema, value, context).prepend("$".into())
    }
}

/// Context containing a list of schemas
#[derive(Debug, Default)]
pub struct YamlContext {
    schemas: Vec<YamlSchema>,
}

impl YamlContext {
    /// Take ownership of a vector of schemas and use those to produce a context
    /// # Examples
    /// ```rust
    /// # use yaml_validator::{YamlSchema, YamlContext};
    /// # use std::str::FromStr;
    /// #
    /// let person = YamlSchema::from_str(r#"
    /// uri: example/person
    /// schema:
    ///   - name: firstname
    ///     type: string
    /// "#).unwrap();
    ///
    /// let context = YamlContext::from_schemas(vec![person]);
    /// ```
    pub fn from_schemas(schemas: Vec<YamlSchema>) -> Self {
        YamlContext { schemas }
    }

    /// Move a new schema into an existing context
    /// ```rust
    /// # use yaml_validator::{YamlSchema, YamlContext};
    /// # use std::str::FromStr;
    /// #
    /// # let person = YamlSchema::from_str(r#"
    /// # uri: example/person
    /// # schema:
    /// #   - name: firstname
    /// #     type: string
    /// # "#).unwrap();
    /// #
    /// # let mut context = YamlContext::from_schemas(vec![person]);
    /// #
    /// let phonebook = YamlSchema::from_str(r#"
    /// schema:
    ///   - name: people
    ///     type: reference
    ///     uri: example/person
    /// "#).unwrap();
    ///
    /// context.add_schema(phonebook);
    /// ```
    pub fn add_schema(&mut self, schema: YamlSchema) {
        self.schemas.push(schema);
    }

    /// Lookup a schema by uri within a YamlContext
    /// # Examples
    /// ```rust
    /// # use yaml_validator::{YamlSchema, YamlContext};
    /// # use std::str::FromStr;
    /// #
    /// let person = YamlSchema::from_str(r#"
    /// uri: example/person
    /// schema:
    ///   - name: firstname
    ///     type: string
    /// "#).unwrap();
    ///
    /// let context = YamlContext::from_schemas(vec![person]);
    ///
    /// assert!(context.lookup("example/person").is_some())
    /// ```
    pub fn lookup(&self, uri: &str) -> Option<&YamlSchema> {
        for schema in self.schemas.iter() {
            if let Some(ref schema_uri) = schema.uri {
                if schema_uri == uri {
                    return Some(&schema);
                }
            }
        }
        None
    }

    /// Returns an immutable list of the schemas currently available within the YamlContext
    pub fn schemas(&self) -> &Vec<YamlSchema> {
        &self.schemas
    }
}
