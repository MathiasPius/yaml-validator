use std::collections::BTreeMap;
use std::convert::TryFrom;
pub use yaml_rust;
use yaml_rust::Yaml;

mod error;
mod types;
mod utils;
use types::*;

pub use error::{SchemaError, SchemaErrorKind};
use error::{add_path_name, optional};

use utils::YamlUtils;

pub trait Validate<'yaml, 'schema: 'yaml> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>>;
}

#[derive(Debug, Default)]
pub struct Context<'schema> {
    schemas: BTreeMap<&'schema str, Schema<'schema>>,
}

impl<'schema> Context<'schema> {
    pub fn get_schema(&self, uri: &str) -> Option<&Schema<'schema>> {
        self.schemas.get(uri)
    }
}

impl<'schema> TryFrom<&'schema Vec<Yaml>> for Context<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(documents: &'schema Vec<Yaml>) -> Result<Self, Self::Error> {
        let (schemas, errs): (Vec<_>, Vec<_>) = documents
            .iter()
            .map(Schema::try_from)
            .partition(Result::is_ok);

        if !errs.is_empty() {
            let mut errors: Vec<SchemaError<'schema>> =
                errs.into_iter().map(Result::unwrap_err).collect();
            if errors.len() == 1 {
                return Err(errors.pop().unwrap());
            } else {
                return Err(SchemaErrorKind::Multiple { errors }.into());
            }
        }

        Ok(Context {
            schemas: schemas
                .into_iter()
                .map(Result::unwrap)
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
    Reference(SchemaReference<'schema>),
}

impl<'schema> TryFrom<&'schema Yaml> for PropertyType<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        let reference = yaml
            .lookup("$ref", "string", Yaml::as_str)
            .map(Option::from)
            .or_else(optional(None))?;

        if let Some(uri) = reference {
            return Ok(PropertyType::Reference(SchemaReference { uri }));
        }

        let typename = yaml.lookup("type", "string", Yaml::as_str)?;

        match typename {
            "object" => Ok(PropertyType::Object(SchemaObject::try_from(yaml)?)),
            "string" => Ok(PropertyType::String(SchemaString::try_from(yaml)?)),
            "integer" => Ok(PropertyType::Integer(SchemaInteger::try_from(yaml)?)),
            "array" => Ok(PropertyType::Array(SchemaArray::try_from(yaml)?)),
            "hash" => Ok(PropertyType::Hash(SchemaHash::try_from(yaml)?)),
            unknown_type => Err(SchemaErrorKind::UnknownType { unknown_type }.into()),
        }
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for PropertyType<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        match self {
            PropertyType::Integer(p) => p.validate(ctx, yaml),
            PropertyType::String(p) => p.validate(ctx, yaml),
            PropertyType::Object(p) => p.validate(ctx, yaml),
            PropertyType::Array(p) => p.validate(ctx, yaml),
            PropertyType::Hash(p) => p.validate(ctx, yaml),
            PropertyType::Reference(p) => p.validate(ctx, yaml),
        }
    }
}

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
            .map_err(add_path_name(uri))?;

        Ok(Schema { uri, schema })
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for Schema<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        self.schema.validate(ctx, yaml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_simple;
    use yaml_rust::YamlLoader;
    use crate::Context;

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

        let context = Context::try_from(&yaml).unwrap();
        let schema = context.get_schema("another").unwrap();
        dbg!(&context);
        dbg!(&schema);
        schema.validate(&context, &load_simple("20")).unwrap();
    }
}
