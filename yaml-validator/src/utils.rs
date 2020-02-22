use crate::error::{SchemaError, SchemaErrorKind};
use std::ops::Index;
use yaml_rust::{yaml::Hash, Yaml};

#[derive(Debug)]
pub enum Limit<T> {
    Inclusive(T),
    Exclusive(T),
}

#[cfg(test)]
pub(crate) fn load_simple(source: &'static str) -> Yaml {
    yaml_rust::YamlLoader::load_from_str(source)
        .unwrap()
        .remove(0)
}

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

    fn strict_contents<'schema>(
        &'schema self,
        required: &[&'schema str],
        optional: &[&'schema str],
    ) -> Result<&Hash, SchemaError<'schema>>;

    fn check_exclusive_fields<'schema>(
        &'schema self,
        exclusive_keys: &[&'static str],
    ) -> Result<(), SchemaError<'schema>>;
}

impl YamlUtils for Yaml {
    fn type_to_str(&self) -> &'static str {
        match self {
            Yaml::Real(_) => "real",
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

    fn strict_contents<'schema>(
        &'schema self,
        required: &[&'schema str],
        optional: &[&'schema str],
    ) -> Result<&Hash, SchemaError<'schema>> {
        let hash = self.as_type("hash", Yaml::as_hash)?;

        let missing = required
            .iter()
            .filter(|field| !hash.contains_key(&Yaml::String((**field).to_string())))
            .map(|field| SchemaErrorKind::FieldMissing { field: *field });

        let extra = hash
            .keys()
            .map(|field| field.as_type("string", Yaml::as_str).unwrap())
            .filter(|field| !required.contains(&field) && !optional.contains(&field))
            .map(|field| SchemaErrorKind::ExtraField { field });

        let mut errors: Vec<SchemaError<'schema>> =
            missing.chain(extra).map(SchemaErrorKind::into).collect();

        if errors.is_empty() {
            Ok(hash)
        } else if errors.len() == 1 {
            Err(errors.pop().unwrap())
        } else {
            Err(SchemaErrorKind::Multiple { errors }.into())
        }
    }

    fn check_exclusive_fields<'schema>(
        &'schema self,
        exclusive_keys: &[&'static str],
    ) -> Result<(), SchemaError<'schema>> {
        let hash = self.as_type("hash", Yaml::as_hash)?;

        let conflicts: Vec<&'static str> = exclusive_keys
            .into_iter()
            .filter(|field| hash.contains_key(&Yaml::String((**field).to_string())))
            .map(|f| *f)
            .collect();
        
        if conflicts.len() > 0 {
            return Err(SchemaErrorKind::MalformedField {
                error: format!("conflicting constraints: {} cannot be used at the same time", conflicts.join(", "))
            }.into());
        }

        Ok(())
    }
}
