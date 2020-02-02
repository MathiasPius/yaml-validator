use crate::error::{SchemaError, SchemaErrorKind};
use std::ops::Index;
use yaml_rust::Yaml;

pub trait YamlUtils {
    fn type_to_str(&self) -> &'static str;

    fn as_type<'schema, F, T>(
        &'schema self,
        expected: &'static str,
        cast: F,
    ) -> Result<T, SchemaError<'schema>>
    where
        F: FnOnce(&'schema Yaml) -> Option<T>;

    fn lookup<'schema, F, T>(
        &'schema self,
        field: &'schema str,
        expected: &'static str,
        cast: F,
    ) -> Result<T, SchemaError<'schema>>
    where
        F: FnOnce(&'schema Yaml) -> Option<T>;
}

impl YamlUtils for Yaml {
    fn type_to_str(&self) -> &'static str {
        match self {
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
        &'schema self,
        expected: &'static str,
        cast: F,
    ) -> Result<T, SchemaError<'schema>>
    where
        F: FnOnce(&'schema Yaml) -> Option<T>,
    {
        Ok(cast(self).ok_or_else(|| {
            SchemaErrorKind::WrongType {
                expected,
                actual: self.type_to_str(),
            }
            .into()
        })?)
    }

    fn lookup<'schema, F, T>(
        &'schema self,
        field: &'schema str,
        expected: &'static str,
        cast: F,
    ) -> Result<T, SchemaError<'schema>>
    where
        F: FnOnce(&'schema Yaml) -> Option<T>,
    {
        let value = self.index(field);
        match value {
            Yaml::BadValue => Err(SchemaErrorKind::FieldMissing { field }.into()),
            Yaml::Null => Err(SchemaErrorKind::FieldMissing { field }.into()),
            content => content.as_type(expected, cast),
        }
    }
}
