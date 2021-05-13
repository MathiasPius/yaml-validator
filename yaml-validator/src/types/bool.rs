use crate::error::{add_path_name, optional, SchemaError, SchemaErrorKind};
use crate::utils::{try_into_usize, YamlUtils};
use crate::{Context, Validate};
use std::convert::TryFrom;
use yaml_rust::Yaml;

#[derive(Debug, Default)]
pub(crate) struct SchemaBool {

}

impl<'schema> TryFrom<&'schema Yaml> for SchemaBool {
    type Error = SchemaError<'schema>;
    fn try_from(yaml: &'schema Yaml) -> Result<Self, Self::Error> {

        Ok(SchemaBool {
        })
    }
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaBool {
    fn validate(
        &self,
        _: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        let _value = yaml.as_type("bool", Yaml::as_bool)?;


        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SchemaErrorKind;
    use crate::utils::load_simple;
    use crate::SchemaString;


}
