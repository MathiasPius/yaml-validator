use std::convert::TryFrom;
use yaml_rust::Yaml;

mod error;
#[cfg(test)]
mod tests;
mod utils;

use error::{SchemaError, SchemaErrorKind};
use utils::YamlUtils;

trait Validate {
    fn validate<'yaml>(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>>;
}

#[derive(Debug, Default)]
struct SchemaObject<'schema> {
    items: Vec<Property<'schema>>,
}

#[derive(Debug, Default)]
struct SchemaString {}

#[derive(Debug, Default)]
struct SchemaInteger {}

#[derive(Debug)]
enum PropertyType<'schema> {
    Object(SchemaObject<'schema>),
    String(SchemaString),
    Integer(SchemaInteger),
}

#[derive(Debug)]
struct Property<'schema> {
    name: &'schema str,
    schematype: PropertyType<'schema>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaObject<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.as_type("hash", Yaml::as_hash)?;

        let items = yaml.lookup("items", "vec", Yaml::as_vec)?;

        let (items, errs): (Vec<_>, Vec<_>) = items
            .iter()
            .map(Property::try_from)
            .partition(Result::is_ok);

        if !errs.is_empty() {
            return Err(SchemaErrorKind::Multiple {
                errors: errs.into_iter().map(Result::unwrap_err).collect(),
            }
            .into());
        }

        Ok(SchemaObject {
            items: items.into_iter().map(Result::unwrap).collect(),
        })
    }
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaString {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.as_type("hash", Yaml::as_hash)?;

        Ok(SchemaString {})
    }
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaInteger {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.as_type("hash", Yaml::as_hash)?;

        Ok(SchemaInteger {})
    }
}

impl<'schema> TryFrom<&'schema Yaml> for PropertyType<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.as_type("hash", Yaml::as_hash)?;
        let typename = yaml.lookup("type", "string", Yaml::as_str)?;

        match typename {
            "hash" => Ok(PropertyType::Object(SchemaObject::try_from(yaml)?)),
            "string" => Ok(PropertyType::String(SchemaString::try_from(yaml)?)),
            "integer" => Ok(PropertyType::Integer(SchemaInteger::try_from(yaml)?)),
            unknown_type => Err(SchemaErrorKind::UnknownType { unknown_type }.into()),
        }
    }
}

impl<'schema> TryFrom<&'schema Yaml> for Property<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.as_type("hash", Yaml::as_hash)?;

        Ok(Property {
            name: yaml.lookup("name", "string", Yaml::as_str)?,
            schematype: PropertyType::try_from(yaml)?,
        })
    }
}

impl Validate for SchemaString {
    fn validate<'yaml>(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>> {
        yaml.as_type("string", Yaml::as_str).and_then(|_| Ok(()))
    }
}

impl Validate for SchemaInteger {
    fn validate<'yaml>(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>> {
        yaml.as_type("integer", Yaml::as_i64).and_then(|_| Ok(()))
    }
}

impl<'schema> Validate for SchemaObject<'schema> {
    fn validate<'yaml>(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>> {
        yaml.as_type("hash", Yaml::as_hash).and_then(|_| Ok(()))
    }
}

impl<'schema> Validate for Property<'schema> {
    fn validate<'yaml>(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>> {
        yaml.as_type("hash", Yaml::as_hash).and_then(|_| Ok(()))?;
        yaml.lookup("name", "string", Yaml::as_str)?;

        self.schematype.validate(yaml)
    }
}

impl<'schema> Validate for PropertyType<'schema> {
    fn validate<'yaml>(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>> {
        match self {
            PropertyType::Integer(p) => p.validate(yaml),
            PropertyType::String(p) => p.validate(yaml),
            PropertyType::Object(p) => p.validate(yaml),
        }
    }
}
