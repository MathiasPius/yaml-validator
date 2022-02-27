use crate::errors::{GenericError, SchemaError, SchemaErrorKind};
use std::convert::TryInto;
use std::fmt::Display;
use std::ops::{Index, Sub};

use yaml_rust::{yaml::Hash, Yaml};

pub trait UnitValue: Sub + Copy + PartialOrd + Default + Display {
    const ZERO: Self;
    const UNIT: Self;
}

impl UnitValue for f64 {
    const ZERO: f64 = 0.0;
    const UNIT: f64 = std::f64::MIN_POSITIVE;
}

impl UnitValue for i64 {
    const ZERO: i64 = 0;
    const UNIT: i64 = 1;
}

#[derive(Debug)]
pub enum Limit<T: UnitValue>
where
    <T as Sub>::Output: UnitValue,
{
    Inclusive(T),
    Exclusive(T),
}

impl<T: UnitValue> Limit<T>
where
    <T as Sub>::Output: UnitValue,
{
    pub fn is_lesser(&self, value: &T) -> bool {
        match self {
            Limit::Inclusive(threshold) => value <= threshold,
            Limit::Exclusive(threshold) => value < threshold,
        }
    }

    pub fn is_greater(&self, value: &T) -> bool {
        match self {
            Limit::Inclusive(threshold) => value >= threshold,
            Limit::Exclusive(threshold) => value > threshold,
        }
    }

    pub fn has_span(&self, upper: &Self) -> bool {
        let zero = <<T as Sub>::Output as UnitValue>::ZERO;
        let unit = <<T as Sub>::Output as UnitValue>::UNIT;

        match (self, upper) {
            (Limit::Inclusive(lower), Limit::Inclusive(upper)) => (*upper - *lower) >= zero,
            (Limit::Exclusive(lower), Limit::Exclusive(upper)) => (*upper - *lower) > unit,
            (Limit::Exclusive(lower), Limit::Inclusive(upper)) => (*upper - *lower) >= zero,
            (Limit::Inclusive(lower), Limit::Exclusive(upper)) => (*upper - *lower) >= zero,
        }
    }
}

pub fn try_into_usize<'a, N: Default + PartialOrd + TryInto<usize>>(
    number: N,
) -> Result<usize, SchemaError<'a>> {
    if number < N::default() {
        return Err(SchemaErrorKind::MalformedField {
            error: "must be a non-negative integer value".into(),
        }
        .into());
    }

    N::try_into(number).map_err(|_| {
        SchemaErrorKind::MalformedField {
            error: "value does not fit in a usize on this system".into(),
        }
        .into()
    })
}

#[cfg(test)]
pub(crate) fn load_simple(source: &'static str) -> Yaml {
    yaml_rust::YamlLoader::load_from_str(source)
        .unwrap()
        .remove(0)
}

pub trait YamlUtils {
    fn type_to_str(&self) -> &'static str;

    fn as_type<'a, F, T>(&'a self, expected: &'static str, cast: F) -> Result<T, GenericError<'a>>
    where
        F: FnOnce(&'a Yaml) -> Option<T>;

    fn lookup<'a, F, T>(
        &'a self,
        field: &'a str,
        expected: &'static str,
        cast: F,
    ) -> Result<T, GenericError<'a>>
    where
        F: FnOnce(&'a Yaml) -> Option<T>;

    fn strict_contents<'schema>(
        &'schema self,
        required: &[&'schema str],
        optional: &[&'schema str],
    ) -> Result<&Hash, GenericError<'schema>>;

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

    fn as_type<'a, F, T>(&'a self, expected: &'static str, cast: F) -> Result<T, GenericError<'a>>
    where
        F: FnOnce(&'a Yaml) -> Option<T>,
    {
        cast(self).ok_or_else(|| GenericError::WrongType {
            expected,
            actual: self.type_to_str(),
        })
    }

    fn lookup<'a, F, T>(
        &'a self,
        field: &'a str,
        expected: &'static str,
        cast: F,
    ) -> Result<T, GenericError<'a>>
    where
        F: FnOnce(&'a Yaml) -> Option<T>,
    {
        let value = self.index(field);
        match value {
            Yaml::BadValue => Err(GenericError::FieldMissing { field }),
            Yaml::Null => Err(GenericError::FieldMissing { field }),
            content => content.as_type(expected, cast),
        }
    }

    fn strict_contents<'schema>(
        &'schema self,
        required: &[&'schema str],
        optional: &[&'schema str],
    ) -> Result<&Hash, GenericError<'schema>> {
        let hash = self.as_type("hash", Yaml::as_hash)?;

        let missing = required
            .iter()
            .filter(|field| !hash.contains_key(&Yaml::String((**field).to_string())))
            .map(|field| GenericError::FieldMissing { field: *field });

        let extra = hash
            .keys()
            .map(|field| field.as_type("string", Yaml::as_str).unwrap())
            .filter(|field| !required.contains(field) && !optional.contains(field))
            .map(|field| GenericError::ExtraField { field });

        let mut errors: Vec<GenericError<'schema>> = missing.chain(extra).collect();

        if errors.is_empty() {
            Ok(hash)
        } else if errors.len() == 1 {
            Err(errors.pop().unwrap())
        } else {
            Err(GenericError::Multiple { errors })
        }
    }

    fn check_exclusive_fields<'schema>(
        &'schema self,
        exclusive_keys: &[&'static str],
    ) -> Result<(), SchemaError<'schema>> {
        let hash = self.as_type("hash", Yaml::as_hash)?;

        let conflicts: Vec<&'static str> = exclusive_keys
            .iter()
            .filter(|field| hash.contains_key(&Yaml::String((**field).to_string())))
            .copied()
            .collect();

        if conflicts.len() > 1 {
            return Err(SchemaErrorKind::MalformedField {
                error: format!(
                    "conflicting constraints: {} cannot be used at the same time",
                    conflicts.join(", ")
                ),
            }
            .into());
        }

        Ok(())
    }
}

pub trait OptionalLookup<'a, T, E> {
    fn into_optional(self) -> Result<Option<T>, E>;
}

impl<'a, T> OptionalLookup<'a, T, GenericError<'a>> for Result<T, GenericError<'a>> {
    fn into_optional(self) -> Result<Option<T>, GenericError<'a>> {
        match self {
            Ok(o) => Ok(Some(o)),
            Err(e) => match e {
                GenericError::FieldMissing { field: _ } => Ok(None),
                _ => Err(e),
            },
        }
    }
}

impl<'a, T> OptionalLookup<'a, T, SchemaErrorKind<'a>> for Result<T, SchemaErrorKind<'a>> {
    fn into_optional(self) -> Result<Option<T>, SchemaErrorKind<'a>> {
        match self {
            Ok(o) => Ok(Some(o)),
            Err(e) => match e {
                SchemaErrorKind::FieldMissing { field: _ } => Ok(None),
                _ => Err(e),
            },
        }
    }
}

impl<'a, T> OptionalLookup<'a, T, SchemaError<'a>> for Result<T, SchemaError<'a>> {
    fn into_optional(self) -> Result<Option<T>, SchemaError<'a>> {
        match self {
            Ok(o) => Ok(Some(o)),
            Err(e) => match e.kind {
                SchemaErrorKind::FieldMissing { field: _ } => Ok(None),
                _ => Err(e),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Limit;

    #[test]
    fn verify_limit_logic_f64() {
        // (10.0 <= x <= 10.0) is a VALID interval
        assert!(Limit::Inclusive(10.0).has_span(&Limit::Inclusive(10.0)));

        // (10.0 < x < 10.0) is an INVALID interval
        assert!(!Limit::Exclusive(10.0).has_span(&Limit::Exclusive(10.0)));

        // (10.0 < x < 11.0) is a VALID interval
        assert!(Limit::Exclusive(10.0).has_span(&Limit::Exclusive(11.0)));

        // (20.0 <= x <= 10.0) is an INVALID interval
        assert!(!Limit::Inclusive(20.0).has_span(&Limit::Inclusive(10.0)));

        // (10.0 < x < 20.0) is a VALID interval
        assert!(Limit::Exclusive(10.0).has_span(&Limit::Exclusive(20.0)));

        // (20.0 < x < 10.0) is an INVALID interval
        assert!(!Limit::Exclusive(20.0).has_span(&Limit::Exclusive(10.0)));
    }

    #[test]
    fn verify_limit_logic_int() {
        // (10 <= x <= 10) is a VALID interval
        assert!(Limit::Inclusive(10).has_span(&Limit::Inclusive(10)));

        // (10 < x < 10) is an INVALID interval
        assert!(!Limit::Exclusive(10).has_span(&Limit::Exclusive(10)));

        // (10.0 < x < 11) is an INVALID interval
        assert!(!Limit::Exclusive(10).has_span(&Limit::Exclusive(11)));

        // (10.0 < x < 12) is a VALID interval
        assert!(Limit::Exclusive(10).has_span(&Limit::Exclusive(12)));

        // (20 <= x <= 10) is an INVALID interval
        assert!(!Limit::Inclusive(20).has_span(&Limit::Inclusive(10)));

        // (10 < x < 20) is a VALID interval
        assert!(Limit::Exclusive(10).has_span(&Limit::Exclusive(20)));

        // (20 < x < 10) is an INVALID interval
        assert!(!Limit::Exclusive(20).has_span(&Limit::Exclusive(10)));
    }
}
