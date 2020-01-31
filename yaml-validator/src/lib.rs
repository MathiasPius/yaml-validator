use error::{OptionalField, PathContext};
use std::convert::TryFrom;
use std::ops::Index;
use yaml_rust::{
    yaml::{Array, Hash},
    Yaml, YamlLoader,
};

#[cfg(test)]
mod tests;

mod error;
pub use error::{StatefulError, YamlSchemaError};
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
struct DataHash {
    pub items: Option<Box<PropertyType>>,
}

#[derive(Debug, PartialEq, Eq)]
struct DataArray {
    pub items: Option<Box<PropertyType>>,
}

#[derive(Debug, PartialEq, Eq)]
struct DataObject {
    pub items: Vec<Property>,
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
    fn get_field(&'a self, field: &'static str)
        -> Result<&'a Yaml, StatefulError<YamlSchemaError>>;
    fn verify_content(
        &'a self,
        field: &'static str,
        typename: &'static str,
    ) -> Result<(), StatefulError<YamlSchemaError>>;
    fn unwrap_bool(&'a self, field: &'static str) -> Result<bool, StatefulError<YamlSchemaError>>;
    fn unwrap_i64(&'a self, field: &'static str) -> Result<i64, StatefulError<YamlSchemaError>>;
    fn unwrap_str(&'a self, field: &'static str)
        -> Result<&'a str, StatefulError<YamlSchemaError>>;
    fn unwrap_hash(
        &'a self,
        field: &'static str,
    ) -> Result<&'a Hash, StatefulError<YamlSchemaError>>;
    fn unwrap_vec(
        &'a self,
        field: &'static str,
    ) -> Result<&'a Array, StatefulError<YamlSchemaError>>;
}

impl<'a> HashAccessor<'a> for Hash {
    fn get_field(
        &'a self,
        field: &'static str,
    ) -> Result<&'a Yaml, StatefulError<YamlSchemaError>> {
        self.get(&Yaml::String(field.into()))
            .ok_or_else(|| YamlSchemaError::MissingField(field).into())
            .prepend(field.into())
    }

    fn verify_content(
        &'a self,
        field: &'static str,
        expected_type: &'static str,
    ) -> Result<(), StatefulError<YamlSchemaError>> {
        let typename = self.unwrap_str(field)?;
        if typename != expected_type {
            return Err(
                YamlSchemaError::TypeMismatch(field, expected_type, typename.into()).into(),
            )
            .prepend(field.into());
        }

        Ok(())
    }

    fn unwrap_bool(&'a self, field: &'static str) -> Result<bool, StatefulError<YamlSchemaError>> {
        let value = self.get_field(field)?;
        value
            .as_bool()
            .ok_or_else(|| YamlSchemaError::WrongType("boolean", type_to_str(value)).into())
            .prepend(field.into())
    }

    fn unwrap_i64(&'a self, field: &'static str) -> Result<i64, StatefulError<YamlSchemaError>> {
        let value = self.get_field(field)?;
        value
            .as_i64()
            .ok_or_else(|| YamlSchemaError::WrongType("i64", type_to_str(value)).into())
            .prepend(field.into())
    }

    fn unwrap_str(
        &'a self,
        field: &'static str,
    ) -> Result<&'a str, StatefulError<YamlSchemaError>> {
        let value = self.get_field(field)?;
        value
            .as_str()
            .ok_or_else(|| YamlSchemaError::WrongType("string", type_to_str(value)).into())
            .prepend(field.into())
    }

    fn unwrap_hash(
        &'a self,
        field: &'static str,
    ) -> Result<&'a Hash, StatefulError<YamlSchemaError>> {
        let value = self.get_field(field)?;
        value
            .as_hash()
            .ok_or_else(|| YamlSchemaError::WrongType("hash", type_to_str(value)).into())
            .prepend(field.into())
    }

    fn unwrap_vec(
        &'a self,
        field: &'static str,
    ) -> Result<&'a Array, StatefulError<YamlSchemaError>> {
        let value = self.get_field(field)?;
        value
            .as_vec()
            .ok_or_else(|| YamlSchemaError::WrongType("array", type_to_str(value)).into())
            .prepend(field.into())
    }
}

impl TryFrom<Yaml> for DataString {
    type Error = StatefulError<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let yaml = yaml.into_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("string descriptor is not an object").into()
        })?;

        yaml.verify_content("type", "string")?;
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
    type Error = StatefulError<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let yaml = yaml.into_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("integer descriptor is not an object").into()
        })?;

        yaml.verify_content("type", "integer")?;
        let min = yaml.unwrap_i64("min").into_optional()?;
        let max = yaml.unwrap_i64("max").into_optional()?;

        Ok(DataInteger { min, max })
    }
}

impl TryFrom<Yaml> for DataReference {
    type Error = StatefulError<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let yaml = yaml.into_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("reference descriptor is not an object").into()
        })?;

        let uri = yaml.unwrap_str("$ref")?.into();

        Ok(DataReference { uri })
    }
}

impl TryFrom<Yaml> for DataHash {
    type Error = StatefulError<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let hash = yaml.as_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("hash descriptor is not an object").into()
        })?;

        hash.verify_content("type", "hash")?;

        let items = yaml.index("items");

        Ok(DataHash {
            items: PropertyType::try_from(items.clone())
                .map(|i| Some(Box::new(i)))
                .prepend("items".into())?,
        })
    }
}

impl TryFrom<Yaml> for DataArray {
    type Error = StatefulError<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let hash = yaml.as_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("array descriptor is not an object").into()
        })?;

        hash.verify_content("type", "array")?;
        let items = yaml.index("items");

        Ok(DataArray {
            items: PropertyType::try_from(items.clone())
                .map(|i| Some(Box::new(i)))
                .prepend("items".into())?,
        })
    }
}

impl TryFrom<Yaml> for DataObject {
    type Error = StatefulError<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let yaml = yaml.into_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("object descriptor is not an object").into()
        })?;

        yaml.verify_content("type", "object")?;

        let items = yaml.unwrap_vec("items")?;

        let mut parsed_items = Vec::with_capacity(items.len());
        for item in items.into_iter() {
            parsed_items.push(Property::try_from(item.clone())?);
        }

        Ok(DataObject {
            items: parsed_items,
        })
    }
}

impl TryFrom<Yaml> for PropertyType {
    type Error = StatefulError<YamlSchemaError>;
    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let hash = yaml.as_hash().ok_or_else(|| {
            YamlSchemaError::SchemaParsingError("propertytype is not an object").into()
        })?;

        match hash.unwrap_str("type")? {
            "integer" => Ok(PropertyType::Integer(DataInteger::try_from(yaml)?)),
            "string" => Ok(PropertyType::String(DataString::try_from(yaml)?)),
            "array" => Ok(PropertyType::Array(DataArray::try_from(yaml)?)),
            "hash" => Ok(PropertyType::Hash(DataHash::try_from(yaml)?)),
            "object" => Ok(PropertyType::Object(DataObject::try_from(yaml)?)),
            "reference" => Ok(PropertyType::Reference(DataReference::try_from(yaml)?)),
            unknown_type => Err(YamlSchemaError::UnknownType(unknown_type.into()).into()),
        }
    }
}

impl TryFrom<Yaml> for Property {
    type Error = StatefulError<YamlSchemaError>;
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
        if let Yaml::Integer(value) = value {
            if let Some(ref min) = self.min {
                if value < min {
                    return Err(YamlValidationError::IntegerValidationError(
                        IntegerValidationError::TooSmall(*min, *value),
                    )
                    .into());
                }
            }

            if let Some(ref max) = self.max {
                if value > max {
                    return Err(YamlValidationError::IntegerValidationError(
                        IntegerValidationError::TooBig(*max, *value),
                    )
                    .into());
                }
            }

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

impl<'a> YamlValidator<'a> for DataHash {
    fn validate(
        &'a self,
        value: &'a Yaml,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
        if let Yaml::Hash(dict) = value {
            if let Some(ref item) = self.items {
                for element in dict.iter() {
                    item.validate(element.1, context).prepend(format!(
                        ".{}",
                        element.0.as_str().unwrap_or("<non-string field>")
                    ))?;
                }
            }
            Ok(())
        } else {
            Err(YamlValidationError::WrongType("hash", value).into())
        }
    }
}

impl<'a> YamlValidator<'a> for DataArray {
    fn validate(
        &'a self,
        value: &'a Yaml,
        context: Option<&'a YamlContext>,
    ) -> ValidationResult<'a> {
        if let Yaml::Array(elements) = value {
            if let Some(ref items) = self.items {
                for (i, element) in elements.iter().enumerate() {
                    items
                        .validate(element, context)
                        .prepend(format!("[{}]", i))?;
                }
            }

            Ok(())
        } else {
            Err(YamlValidationError::WrongType("array", value).into())
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
        DataObject::validate(&self.items, value, context)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum PropertyType {
    Integer(DataInteger),
    String(DataString),
    Array(DataArray),
    Hash(DataHash),
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
            PropertyType::Array(p) => p.validate(value, context),
            PropertyType::Hash(p) => p.validate(value, context),
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
    pub fn validate_str<'a>(
        &'a self,
        yaml: &'a str,
        context: Option<&'a YamlContext>,
    ) -> Result<(), String> {
        let docs = YamlLoader::load_from_str(yaml)
            .map_err(|e| format!("{}", YamlValidationError::YamlScanError(e)))?;
        for doc in docs {
            self.validate(&doc, context).map_err(|e| format!("{}", e))?;
        }
        Ok(())
    }
}

impl TryFrom<Yaml> for YamlSchema {
    type Error = StatefulError<YamlSchemaError>;

    fn try_from(yaml: Yaml) -> Result<YamlSchema, Self::Error> {
        if let Some(yaml) = yaml.as_hash() {
            let uri = yaml
                .unwrap_str("uri")
                .into_optional()?
                .and_then(|uri| Some(uri.into()));

            let schema = yaml.unwrap_vec("schema")?;

            let mut parsed_fields = Vec::with_capacity(schema.len());
            for field in schema {
                parsed_fields.push(Property::try_from(field.clone())?);
            }

            return Ok(YamlSchema {
                uri,
                schema: parsed_fields,
            });
        }

        Ok(YamlSchema {
            uri: None,
            schema: vec![],
        })
    }
}

impl std::str::FromStr for YamlSchema {
    type Err = StatefulError<YamlSchemaError>;
    fn from_str(schema: &str) -> std::result::Result<YamlSchema, Self::Err> {
        let mut docs = YamlLoader::load_from_str(schema)
            .map_err(|e| YamlSchemaError::YamlScanError(e).into())?;
        let first = docs.pop().ok_or_else(|| YamlSchemaError::NoSchema.into())?;

        if docs.is_empty() {
            Ok(YamlSchema::try_from(first)?)
        } else {
            Err(YamlSchemaError::MultipleSchemas.into())
        }
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
