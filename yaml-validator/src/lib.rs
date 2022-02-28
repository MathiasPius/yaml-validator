#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::convert::TryFrom;
pub use yaml_rust;
use yaml_rust::Yaml;

mod breadcrumb;
mod errors;
mod modifiers;
mod types;
mod utils;
use modifiers::*;
use types::*;

pub use errors::schema::{SchemaError, SchemaErrorKind};
use errors::ValidationError;

use crate::types::bool::SchemaBool;
use utils::{CondenseErrors, OptionalLookup, YamlUtils};

/// Validation trait implemented by all types, as well as the [Schema](crate::Schema) type
pub trait Validate<'yaml, 'schema: 'yaml> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), ValidationError<'yaml>>;
}

/// Contains a number of schemas that may or may not be dependent on each other.
#[derive(Debug, Default)]
pub struct Context<'schema> {
    schemas: BTreeMap<&'schema str, Schema<'schema>>,
}

impl<'schema> Context<'schema> {
    /// Get a reference to a single schema within the context to use for validation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use yaml_rust::YamlLoader;
    /// # use std::convert::TryFrom;
    /// # use yaml_validator::{Validate, Context};
    /// #
    /// let schemas = vec![
    ///     YamlLoader::load_from_str(r#"
    ///         uri: just-a-number
    ///         schema:
    ///             type: integer
    ///     "#).unwrap().remove(0)
    /// ];
    ///
    /// let context = Context::try_from(&schemas[..]).unwrap();
    /// let document = YamlLoader::load_from_str("10").unwrap().remove(0);
    ///
    /// context.get_schema("just-a-number").unwrap()
    ///     .validate(&context, &document).unwrap();
    /// ```
    pub fn get_schema(&self, uri: &str) -> Option<&Schema<'schema>> {
        self.schemas.get(uri)
    }
}

/// A context can only be created from a vector of Yaml documents, all of which must fit the schema layout.
impl<'schema> TryFrom<&'schema [Yaml]> for Context<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(documents: &'schema [Yaml]) -> Result<Self, Self::Error> {
        let schemas = SchemaError::condense_errors(&mut documents.iter().map(Schema::try_from))?;

        Ok(Context {
            schemas: schemas
                .into_iter()
                .map(|schema| (schema.uri, schema))
                .collect(),
        })
    }
}

#[derive(Debug)]
enum PropertyType<'schema> {
    Object(SchemaObject<'schema>),
    Array(SchemaArray<'schema>),
    Hash(SchemaHash<'schema>),
    String(SchemaString),
    Integer(SchemaInteger),
    Real(SchemaReal),
    Bool(SchemaBool),
    Reference(SchemaReference<'schema>),
    Not(SchemaNot<'schema>),
    OneOf(SchemaOneOf<'schema>),
    AllOf(SchemaAllOf<'schema>),
    AnyOf(SchemaAnyOf<'schema>),
}

impl<'schema> TryFrom<&'schema Yaml> for PropertyType<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        if yaml.as_hash().is_none() {
            return Err(SchemaErrorKind::WrongType {
                expected: "hash",
                actual: yaml.type_to_str(),
            }
            .into());
        }

        if let Some(uri) = yaml
            .lookup("$ref", "string", Yaml::as_str)
            .into_optional()
            .map_err(SchemaError::from)?
        {
            return Ok(PropertyType::Reference(SchemaReference { uri }));
        }

        if yaml
            .lookup("not", "hash", Option::from)
            .into_optional()
            .map_err(SchemaError::from)?
            .is_some()
        {
            return Ok(PropertyType::Not(SchemaNot::try_from(yaml)?));
        }

        if yaml
            .lookup("oneOf", "hash", Option::from)
            .into_optional()
            .map_err(SchemaError::from)?
            .is_some()
        {
            return Ok(PropertyType::OneOf(SchemaOneOf::try_from(yaml)?));
        }

        if yaml
            .lookup("allOf", "hash", Option::from)
            .into_optional()
            .map_err(SchemaError::from)?
            .is_some()
        {
            return Ok(PropertyType::AllOf(SchemaAllOf::try_from(yaml)?));
        }

        if yaml
            .lookup("anyOf", "hash", Option::from)
            .into_optional()
            .map_err(SchemaError::from)?
            .is_some()
        {
            return Ok(PropertyType::AnyOf(SchemaAnyOf::try_from(yaml)?));
        }

        let typename = yaml.lookup("type", "string", Yaml::as_str)?;

        match typename {
            "object" => Ok(PropertyType::Object(SchemaObject::try_from(yaml)?)),
            "string" => Ok(PropertyType::String(SchemaString::try_from(yaml)?)),
            "integer" => Ok(PropertyType::Integer(SchemaInteger::try_from(yaml)?)),
            "real" => Ok(PropertyType::Real(SchemaReal::try_from(yaml)?)),
            "array" => Ok(PropertyType::Array(SchemaArray::try_from(yaml)?)),
            "hash" => Ok(PropertyType::Hash(SchemaHash::try_from(yaml)?)),
            "boolean" => Ok(PropertyType::Bool(SchemaBool::try_from(yaml)?)),
            unknown_type => Err(SchemaErrorKind::UnknownType { unknown_type }.into()),
        }
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for PropertyType<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), ValidationError<'yaml>> {
        match self {
            PropertyType::Integer(p) => p.validate(ctx, yaml),
            PropertyType::Real(p) => p.validate(ctx, yaml),
            PropertyType::String(p) => p.validate(ctx, yaml),
            PropertyType::Object(p) => p.validate(ctx, yaml),
            PropertyType::Array(p) => p.validate(ctx, yaml),
            PropertyType::Hash(p) => p.validate(ctx, yaml),
            PropertyType::Reference(p) => p.validate(ctx, yaml),
            PropertyType::Not(p) => p.validate(ctx, yaml),
            PropertyType::OneOf(p) => p.validate(ctx, yaml),
            PropertyType::AllOf(p) => p.validate(ctx, yaml),
            PropertyType::AnyOf(p) => p.validate(ctx, yaml),
            PropertyType::Bool(p) => p.validate(ctx, yaml),
        }
    }
}

/// A single schema unit used for validation.
#[derive(Debug)]
pub struct Schema<'schema> {
    uri: &'schema str,
    schema: PropertyType<'schema>,
}

impl<'schema> TryFrom<&'schema Yaml> for Schema<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&["uri", "schema"], &[])?;

        let uri = yaml.lookup("uri", "string", Yaml::as_str)?;
        let schema = PropertyType::try_from(yaml.lookup("schema", "yaml", Option::from)?)
            .map_err(SchemaError::add_path_name(uri))?;

        Ok(Schema { uri, schema })
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for Schema<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), ValidationError<'yaml>> {
        self.schema.validate(ctx, yaml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_simple;
    use crate::Context;
    use yaml_rust::YamlLoader;

    #[test]
    fn from_yaml() {
        let yaml = YamlLoader::load_from_str(
            r#"---
uri: test
schema:
  type: integer
---
uri: another
schema:
  $ref: test
"#,
        )
        .unwrap();

        let context = Context::try_from(&yaml[..]).unwrap();
        let schema = context.get_schema("another").unwrap();
        dbg!(&context);
        dbg!(&schema);
        schema.validate(&context, &load_simple("20")).unwrap();
    }
}
