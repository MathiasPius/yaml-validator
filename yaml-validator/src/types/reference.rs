use crate::error::{SchemaError, SchemaErrorKind};
use crate::{Context, Validate};
use yaml_rust::Yaml;

#[derive(Debug, Default)]
pub(crate) struct SchemaReference<'schema> {
    pub(crate) uri: &'schema str,
}

impl<'yaml, 'schema: 'yaml> Validate<'yaml, 'schema> for SchemaReference<'schema> {
    fn validate(
        &self,
        ctx: &'schema Context<'schema>,
        yaml: &'yaml Yaml,
    ) -> Result<(), SchemaError<'yaml>> {
        if let Some(schema) = ctx.get_schema(self.uri) {
            schema.validate(ctx, yaml)
        } else {
            Err(SchemaErrorKind::UnknownSchema { uri: self.uri }.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_simple;
    use crate::SchemaReference;

    #[test]
    fn validate_string() {
        assert_eq!(
            SchemaReference { uri: "test" }
                .validate(&Context::default(), &load_simple("hello"))
                .unwrap_err(),
            SchemaErrorKind::UnknownSchema { uri: "test" }.into()
        );
    }
}
