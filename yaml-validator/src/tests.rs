use crate::{error::SchemaErrorKind, SchemaObject};
use std::convert::TryFrom;
use yaml_rust::{Yaml, YamlLoader};

fn load_simple(source: &'static str) -> Yaml {
    YamlLoader::load_from_str(source).unwrap().remove(0)
}

#[test]
fn schemaobject_from_yaml() {
    SchemaObject::try_from(&load_simple(
        "items:\n  - name: something\n  - type: string",
    ))
    .unwrap();

    assert_eq!(
        SchemaObject::try_from(&load_simple("world")).unwrap_err(),
        SchemaErrorKind::WrongType {
            expected: "hash",
            actual: "string"
        }
        .into()
    );

    assert_eq!(
        SchemaObject::try_from(&load_simple("10")).unwrap_err(),
        SchemaErrorKind::WrongType {
            expected: "hash",
            actual: "integer"
        }
        .into()
    );

    assert_eq!(
        SchemaObject::try_from(&load_simple("- hello\n  - world")).unwrap_err(),
        SchemaErrorKind::WrongType {
            expected: "hash",
            actual: "array"
        }
        .into()
    );
}
