use error::{OptionalField, PathContext};
use std::convert::TryFrom;
use yaml_rust::{
    yaml::{Array, Hash},
    Yaml, YamlLoader,
};

#[cfg(test)]
mod tests;

mod error;
pub use error::YamlSchemaError;
use error::{ValidationResult, *};

#[derive(Debug, PartialEq, Eq)]
struct DataInteger {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

#[derive(Debug, PartialEq, Eq)]
struct DataString {
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
}

#[derive(Debug, PartialEq, Eq)]
struct DataReference {
    pub uri: String,
}

#[derive(Debug, PartialEq, Eq)]
struct DataDictionary {
    pub value: Option<Box<PropertyType>>,
}

#[derive(Debug, PartialEq, Eq)]
struct DataList {
    pub inner: Option<Box<PropertyType>>,
}

#[derive(Debug, PartialEq, Eq)]
struct DataObject {
    pub fields: Vec<Property>,
}

trait YamlValidator<'a> {
    fn validate(
        &'a self,
        value: &'a Yaml,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a>;
}

fn type_to_str(yaml: &Yaml) -> &'static str {
    match yaml {
        Yaml::Real(_) => "float",
        Yaml::Integer(_) => "integer",
        Yaml::String(_) => "string",
        Yaml::Boolean(_) => "boolean",
        Yaml::Array(_) => "array",
        Yaml::Hash(_) => "hash",
        Yaml::Alias(_) => "alias",
        Yaml::Null => "null",
        Yaml::BadValue => "bad_value",
    }
}

trait HashAccessor<'a> {
    fn get_field(
        &'a self,
        field: &'static str,
    ) -> Result<&'a Yaml, StatefulResult<YamlSchemaError>>;
    fn verify_type(&'a self, typename: &'static str)
        -> Result<(), StatefulResult<YamlSchemaError>>;
    fn unwrap_bool(&'a self, field: &'static str) -> Result<bool, StatefulResult<YamlSchemaError>>;
    fn unwrap_i64(&'a self, field: &'static str) -> Result<i64, StatefulResult<YamlSchemaError>>;
    fn unwrap_str(
        &'a self,
        field: &'static str,
    ) -> Result<&'a str, StatefulResult<YamlSchemaError>>;
    fn unwrap_hash(
        &'a self,
        field: &'static str,
    ) -> Result<&'a Hash, StatefulResult<YamlSchemaError>>;
    fn unwrap_vec(
        &'a self,
        field: &'static str,
    ) -> Result<&'a Array, StatefulResult<YamlSchemaError>>;
}

impl<'a> HashAccessor<'a> for Hash {
    fn get_field(
        &'a self,
        field: &'static str,
    ) -> Result<&'a Yaml, StatefulResult<YamlSchemaError>> {
        self.get(&Yaml::String(field.into()))
            .ok_or_else(|| YamlSchemaError::MissingField(field).into())
            .prepend(field.into())
    }

    fn verify_type(
        &'a self,
        expected_type: &'static str,
    ) -> Result<(), StatefulResult<YamlSchemaError>> {
        let typename = self.unwrap_str("type")?;
        if typename != expected_type {
            return Err(YamlSchemaError::TypeMismatch(expected_type, typename.into()).into())
                .prepend("type".into());
        }

        Ok(())
    }

    fn unwrap_bool(&'a self, field: &'static str) -> Result<bool, StatefulResult<YamlSchemaError>> {
        let value = self.get_field(field)?;
        value
            .as_bool()
            .ok_or_else(|| YamlSchemaError::WrongType("boolean", type_to_str(value)).into())
            .prepend(field.into())
    }

    fn unwrap_i64(&'a self, field: &'static str) -> Result<i64, StatefulResult<YamlSchemaError>> {
        let value = self.get_field(field)?;
        value
            .as_i64()
            .ok_or_else(|| YamlSchemaError::WrongType("i64", type_to_str(value)).into())
            .prepend(field.into())
    }

    fn unwrap_str(
        &'a self,
        field: &'static str,
    ) -> Result<&'a str, StatefulResult<YamlSchemaError>> {
        let value = self.get_field(field)?;
        value
            .as_str()
            .ok_or_else(|| YamlSchemaError::WrongType("string", type_to_str(value)).into())
            .prepend(field.into())
    }

    fn unwrap_hash(
        &'a self,
        field: &'static str,
    ) -> Result<&'a Hash, StatefulResult<YamlSchemaError>> {
        let value = self.get_field(field)?;
        value
            .as_hash()
            .ok_or_else(|| YamlSchemaError::WrongType("hash", type_to_str(value)).into())
            .prepend(field.into())
    }

    fn unwrap_vec(
        &'a self,
        field: &'static str,
    ) -> Result<&'a Array, StatefulResult<YamlSchemaError>> {
        let value = self.get_field(field)?;
        value
            .as_vec()
            .ok_or_else(|| YamlSchemaError::WrongType("array", type_to_str(value)).into())
            .prepend(field.into())
    }
}

impl TryFrom<Yaml> for DataString {
    type Error = StatefulResult<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let yaml = yaml.into_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("datastring is not an object").into()
        })?;

        yaml.verify_type("string")?;
        let max_length = yaml
            .unwrap_i64("max_length")
            .into_optional()?
            .map(|i| i as usize);
        let min_length = yaml
            .unwrap_i64("min_length")
            .into_optional()?
            .map(|i| i as usize);

        Ok(DataString {
            max_length,
            min_length,
        })
    }
}

impl TryFrom<Yaml> for DataInteger {
    type Error = StatefulResult<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let yaml = yaml.into_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("datastring is not an object").into()
        })?;

        yaml.verify_type("integer")?;

        let min = yaml.unwrap_i64("min").into_optional()?;

        let max = yaml.unwrap_i64("max").into_optional()?;

        Ok(DataInteger { min, max })
    }
}

impl TryFrom<Yaml> for DataReference {
    type Error = StatefulResult<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let yaml = yaml.into_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("datareference is not an object").into()
        })?;

        yaml.verify_type("reference")?;

        let uri = yaml.unwrap_str("uri")?.into();

        Ok(DataReference { uri })
    }
}

impl TryFrom<Yaml> for DataDictionary {
    type Error = StatefulResult<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let yaml = yaml.into_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("datadictionary is not an object").into()
        })?;

        yaml.verify_type("dictionary")?;

        let _value = yaml.unwrap_hash("value").into_optional()?;

        Ok(DataDictionary { value: None })
    }
}

impl TryFrom<Yaml> for DataList {
    type Error = StatefulResult<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let yaml = yaml.into_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("datalist is not an object").into()
        })?;

        yaml.verify_type("list")?;

        let _value = yaml.unwrap_hash("value").into_optional()?;

        Ok(DataList { inner: None })
    }
}

impl TryFrom<Yaml> for DataObject {
    type Error = StatefulResult<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let yaml = yaml.into_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("datalist is not an object").into()
        })?;

        yaml.verify_type("object")?;

        let _fields = yaml.unwrap_hash("fields").into_optional()?;

        Ok(DataObject { fields: vec![] })
    }
}

impl TryFrom<Yaml> for PropertyType {
    type Error = StatefulResult<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let hash = yaml.as_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("propertytype is not an object").into()
        })?;

        match hash.unwrap_str("type")? {
            "integer" => Ok(PropertyType::Integer(DataInteger::try_from(yaml)?)),
            "string" => Ok(PropertyType::String(DataString::try_from(yaml)?)),
            "list" => Ok(PropertyType::List(DataList::try_from(yaml)?)),
            "dictionary" => Ok(PropertyType::Dictionary(DataDictionary::try_from(yaml)?)),
            "object" => Ok(PropertyType::Object(DataObject::try_from(yaml)?)),
            "reference" => Ok(PropertyType::Reference(DataReference::try_from(yaml)?)),
            unknown_type => Err(YamlSchemaError::UnknownType(unknown_type.into()).into()),
        }
    }
}

impl TryFrom<Yaml> for Property {
    type Error = StatefulResult<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let hash = yaml.as_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("property is not an object").into()
        })?;

        Ok(Property {
            name: hash.unwrap_str("name")?.into(),
            datatype: PropertyType::try_from(yaml)?,
        })
    }
}

impl<'a> YamlValidator<'a> for DataInteger {
    fn validate(&'a self, value: &'a Yaml, _: Option<&'a YamlContext>) -> ValidationResult<'a> {
        if let Yaml::Integer(_) = value {
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("integer", value).into())
        }
    }
}

impl<'a> YamlValidator<'a> for DataString {
    fn validate(&'a self, value: &'a Yaml, _: Option<&'a YamlContext>) -> ValidationResult<'a> {
        if let Yaml::String(inner) = value {
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

impl<'a> YamlValidator<'a> for DataReference {
    fn validate(
        &'a self,
        value: &'a Yaml,
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

impl<'a> YamlValidator<'a> for DataDictionary {
    fn validate(
        &'a self,
        value: &'a Yaml,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
        if let Yaml::Hash(dict) = value {
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

impl<'a> YamlValidator<'a> for DataList {
    fn validate(
        &'a self,
        value: &'a Yaml,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
        if let Yaml::Array(items) = value {
            for (i, item) in items.iter().enumerate() {
                self.inner
                    .as_ref()
                    .unwrap()
                    .validate(item, context)
                    .prepend(format!("[{}]", i))?;
            }
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("list", value).into())
        }
    }
}

impl DataObject {
    pub fn validate<'a>(
        properties: &'a [Property],
        value: &'a Yaml,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
        if let Yaml::Hash(ref obj) = value {
            for prop in properties.iter() {
                if let Some(field) = obj.get(&Yaml::from_str(&prop.name)) {
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
        value: &'a Yaml,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
        DataObject::validate(&self.fields, value, context)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum PropertyType {
    Integer(DataInteger),
    String(DataString),
    List(DataList),
    Dictionary(DataDictionary),
    Object(DataObject),
    Reference(DataReference),
}

impl<'a> YamlValidator<'a> for PropertyType {
    fn validate(
        &'a self,
        value: &'a Yaml,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
        match self {
            PropertyType::Integer(p) => p.validate(value, context),
            PropertyType::String(p) => p.validate(value, context),
            PropertyType::List(p) => p.validate(value, context),
            PropertyType::Dictionary(p) => p.validate(value, context),
            PropertyType::Object(p) => p.validate(value, context),
            PropertyType::Reference(p) => p.validate(value, context),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Property {
    pub name: String,
    pub datatype: PropertyType,
}

/// Struct containing a list of internal properties as defined in its top-level `schema` field
#[derive(Debug)]
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
        match self.validate(&Yaml::from_str(yaml), context) {
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

impl std::str::FromStr for YamlSchema {
    type Err = YamlSchemaError;
    fn from_str(schema: &str) -> std::result::Result<YamlSchema, Self::Err> {
        YamlSchema::try_from(Yaml::from_str(schema)).map_err(|e| e.into())
    }
}

impl<'a> YamlValidator<'a> for YamlSchema {
    fn validate(
        &'a self,
        value: &'a Yaml,
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
