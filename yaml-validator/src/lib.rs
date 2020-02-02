use std::convert::TryFrom;
use yaml_rust::Yaml;

mod error;
#[cfg(test)]
mod tests;
mod utils;

use error::{add_path_index, add_path_name, incomplete, optional, SchemaError, SchemaErrorKind};
use utils::YamlUtils;

trait Validate<'yaml, 'schema: 'yaml> {
    fn validate(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>>;
}

#[derive(Debug, Default)]
struct SchemaObject<'schema> {
    items: Vec<Property<'schema>>,
}

#[derive(Debug, Default)]
struct SchemaArray<'schema> {
    items: Option<Box<PropertyType<'schema>>>,
}

#[derive(Debug, Default)]
struct SchemaString {}

#[derive(Debug, Default)]
struct SchemaInteger {}

#[derive(Debug)]
enum PropertyType<'schema> {
    Object(SchemaObject<'schema>),
    Array(SchemaArray<'schema>),
    String(SchemaString),
    Integer(SchemaInteger),
}

#[derive(Debug)]
struct Property<'schema> {
    name: &'schema str,
    schematype: Option<PropertyType<'schema>>,
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaObject<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&["items"], &["name", "type"])?;

        let items = yaml.lookup("items", "array", Yaml::as_vec)?;

        let (items, errs): (Vec<_>, Vec<_>) = items
            .iter()
            .enumerate()
            .map(|(i, property)| {
                Property::try_from(property)
                    .map_err(add_path_name("items"))
                    .map_err(add_path_index(i))
            })
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

impl<'schema> TryFrom<&'schema Yaml> for SchemaArray<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&[], &["items", "name", "type"])?;

        // I'm using Option::from here because I don't actually want to transform
        // the resulting yaml object into a specific type, but need the yaml itself
        // to be passed into PropertyType::try_from
        yaml.lookup("items", "yaml", Option::from)
            .map(|inner| {
                yaml.lookup("items", "hash", Yaml::as_hash)
                    .map_err(add_path_name("items"))?;

                Ok(SchemaArray {
                    items: Some(Box::new(
                        PropertyType::try_from(inner).map_err(add_path_name("items"))?,
                    )),
                })
            })
            .or_else(optional(Ok(SchemaArray { items: None })))?
    }
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaString {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&[], &["name", "type"])?;

        Ok(SchemaString {})
    }
}

impl<'schema> TryFrom<&'schema Yaml> for SchemaInteger {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&[], &["name", "type"])?;

        Ok(SchemaInteger {})
    }
}

impl<'schema> TryFrom<&'schema Yaml> for PropertyType<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        let typename = yaml.lookup("type", "string", Yaml::as_str)?;
        dbg!(typename);
        match typename {
            "object" => Ok(PropertyType::Object(SchemaObject::try_from(yaml)?)),
            "string" => Ok(PropertyType::String(SchemaString::try_from(yaml)?)),
            "integer" => Ok(PropertyType::Integer(SchemaInteger::try_from(yaml)?)),
            "array" => Ok(PropertyType::Array(SchemaArray::try_from(yaml)?)),
            unknown_type => Err(SchemaErrorKind::UnknownType { unknown_type }.into()),
        }
    }
}

impl<'schema> TryFrom<&'schema Yaml> for Property<'schema> {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {
        yaml.strict_contents(&["name"], &["type"])
            .map(Option::from)
            .or_else(incomplete(None))?;

        let name = yaml.lookup("name", "string", Yaml::as_str)?;
        Ok(Property {
            name,
            schematype: PropertyType::try_from(yaml)
                .map(Option::from)
                .or_else(optional(None))?,
        })
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaString {
    fn validate(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>> {
        yaml.as_type("string", Yaml::as_str).and_then(|_| Ok(()))
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaInteger {
    fn validate(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>> {
        yaml.as_type("integer", Yaml::as_i64).and_then(|_| Ok(()))
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaObject<'schema> {
    fn validate(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>> {
        yaml.as_type("hash", Yaml::as_hash).and_then(|_| Ok(()))?;

        let items: Vec<&'schema str> = self.items.iter().map(|item| item.name).collect();
        yaml.strict_contents(&items, &[])?;

        let mut errors: Vec<SchemaError<'yaml>> = self
            .items
            .iter()
            .map(|schema_item| {
                let item: &Yaml = yaml
                    .lookup(schema_item.name, "yaml", Option::from)
                    .map_err(add_path_name(schema_item.name))?;

                schema_item
                    .validate(item)
                    .map_err(add_path_name(schema_item.name))?;
                Ok(())
            })
            .filter_map(Result::err)
            .collect();

        if errors.is_empty() {
            Ok(())
        } else if errors.len() == 1 {
            Err(errors.pop().unwrap())
        } else {
            Err(SchemaErrorKind::Multiple { errors }.into())
        }
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaArray<'schema> {
    fn validate(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>> {
        let items = yaml.as_type("array", Yaml::as_vec)?;

        if let Some(schema) = &self.items {
            let mut errors: Vec<SchemaError<'yaml>> = items
                .iter()
                .enumerate()
                .map(|(i, item)| schema.validate(item).map_err(add_path_index(i)))
                .filter(Result::is_err)
                .map(Result::unwrap_err)
                .collect();

            return if errors.is_empty() {
                Ok(())
            } else if errors.len() == 1 {
                Err(errors.pop().unwrap())
            } else {
                Err(SchemaErrorKind::Multiple { errors }.into())
            };
        }

        Ok(())
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for Property<'schema> {
    fn validate(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>> {
        if let Some(schematype) = &self.schematype {
            return schematype.validate(yaml);
        }

        Ok(())
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for PropertyType<'schema> {
    fn validate(&self, yaml: &'yaml Yaml) -> Result<(), SchemaError<'yaml>> {
        match self {
            PropertyType::Integer(p) => p.validate(yaml),
            PropertyType::String(p) => p.validate(yaml),
            PropertyType::Object(p) => p.validate(yaml),
            PropertyType::Array(p) => p.validate(yaml),
        }
    }
}
