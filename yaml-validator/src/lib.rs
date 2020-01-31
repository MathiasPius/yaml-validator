use std::convert::TryFrom;
use std::ops::Index;
use yaml_rust::Yaml;

mod error;
#[cfg(test)]
mod tests;
use error::{SchemaError, SchemaErrorKind};

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

fn lookup<'schema, F, T>(
    yaml: &'schema Yaml,
    field: &'static str,
    expected: &'static str,
    cast: F,
) -> Result<T, SchemaError<'schema>>
where
    F: FnOnce(&'schema Yaml) -> Option<T>,
{
    let value = yaml.index(field);
    match value {
        Yaml::BadValue => Err(SchemaErrorKind::FieldMissing { field }.into()),
        content => Ok(cast(content).ok_or_else(|| {
            SchemaErrorKind::WrongType {
                expected,
                actual: type_to_str(value),
            }
            .into()
        })?),
    }
}

#[derive(Debug)]
struct SchemaObject<'schema> {
    items: Vec<Property<'schema>>,
}

#[derive(Debug)]
enum PropertyType<'schema> {
    Object(SchemaObject<'schema>),
}

#[derive(Debug)]
struct Property<'schema> {
    name: &'schema str,
    schematype: PropertyType<'schema>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaObject<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        let _hash = yaml.as_hash().ok_or_else(|| {
            SchemaErrorKind::DescriptorNotHash {
                expected: "hash",
                actual: type_to_str(yaml),
            }
            .into()
        })?;

        let _items = lookup(yaml, "items", "hash", |y| y.as_hash())?;

        Ok(SchemaObject { items: vec![] })
    }
}

impl<'schema> TryFrom<&'schema Yaml> for PropertyType<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        let _hash = yaml.as_hash().ok_or_else(|| {
            SchemaErrorKind::DescriptorNotHash {
                expected: "object",
                actual: type_to_str(yaml),
            }
            .into()
        })?;

        let typename = lookup(yaml, "type", "string", |y| y.as_str())?;

        match typename {
            "object" => Ok(PropertyType::Object(SchemaObject::try_from(yaml)?)),
            unknown_type => Err(SchemaErrorKind::UnknownType { unknown_type }.into()),
        }
    }
}
