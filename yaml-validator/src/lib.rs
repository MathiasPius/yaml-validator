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

fn as_type<'schema, F, T>(
    yaml: &'schema Yaml,
    expected: &'static str,
    cast: F,
) -> Result<T, SchemaError<'schema>>
where
    F: FnOnce(&'schema Yaml) -> Option<T>,
{
    Ok(cast(yaml).ok_or_else(|| {
        SchemaErrorKind::WrongType {
            expected,
            actual: type_to_str(yaml),
        }
        .into()
    })?)
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
        Yaml::Null => Err(SchemaErrorKind::FieldMissing { field }.into()),
        content => as_type(content, expected, cast),
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
        as_type(yaml, "hash", Yaml::as_hash)?;

        let items = lookup(yaml, "items", "vec", Yaml::as_vec)?;

        let (items, errs): (Vec<_>, Vec<_>) = items
            .into_iter()
            .map(|field| Property::try_from(field))
            .partition(Result::is_err);

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

impl<'schema> TryFrom<&'schema Yaml> for PropertyType<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        as_type(yaml, "hash", Yaml::as_hash)?;
        let typename = lookup(yaml, "type", "string", Yaml::as_str)?;

        match typename {
            "hash" => Ok(PropertyType::Object(SchemaObject::try_from(yaml)?)),
            unknown_type => Err(SchemaErrorKind::UnknownType { unknown_type }.into()),
        }
    }
}

impl<'schema> TryFrom<&'schema Yaml> for Property<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        as_type(yaml, "hash", Yaml::as_hash)?;

        Ok(Property {
            name: lookup(yaml, "name", "string", Yaml::as_str)?,
            schematype: PropertyType::try_from(yaml)?,
        })
    }
}
