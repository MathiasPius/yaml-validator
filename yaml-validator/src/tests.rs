use crate::{error::SchemaErrorKind};
use std::convert::TryFrom;
use yaml_rust::{Yaml, YamlLoader};

fn load_simple(source: &'static str) -> Yaml {
    YamlLoader::load_from_str(source).unwrap().remove(0)
}


mod schemaobject {
    use crate::SchemaObject;
    use super::*;
    #[test]
    fn schemaobject_from_yaml() {
        SchemaObject::try_from(&load_simple(r#"
            items:
              - name: something
                type: string
        "#))
        .unwrap();
    }

    #[test]
    fn schemaobject_from_string() {
        assert_eq!(
            SchemaObject::try_from(&load_simple("world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn schemaobject_from_integer() {
        assert_eq!(
            SchemaObject::try_from(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn schemaobject_from_array() {
        assert_eq!(
            SchemaObject::try_from(&load_simple(r#"
                - hello
                - world
            "#)).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .into()
        );
    }
}

mod schemastring {
    use crate::SchemaString;
    use super::*;
    #[test]
    fn schemastring_from_yaml() {
        SchemaString::try_from(&load_simple("type: string"))
        .unwrap();
    }

    #[test]
    fn schemastring_from_string() {
        assert_eq!(
            SchemaString::try_from(&load_simple("world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn schemastring_from_integer() {
        assert_eq!(
            SchemaString::try_from(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn schemastring_from_array() {
        assert_eq!(
            SchemaString::try_from(&load_simple(r#"
                - hello
                - world
            "#)).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .into()
        );
    }
}

mod schemainteger {
    use crate::SchemaInteger;
    use super::*;
    #[test]
    fn schemainteger_from_yaml() {
        SchemaInteger::try_from(&load_simple("type: string")).unwrap();
    }

    #[test]
    fn schemainteger_from_string() {
        assert_eq!(
            SchemaInteger::try_from(&load_simple("world")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "string"
            }
            .into()
        );
    }

    #[test]
    fn schemainteger_from_integer() {
        assert_eq!(
            SchemaInteger::try_from(&load_simple("10")).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "integer"
            }
            .into()
        );
    }

    #[test]
    fn schemainteger_from_array() {
        assert_eq!(
            SchemaInteger::try_from(&load_simple(r#"
                - hello
                - world
            "#)).unwrap_err(),
            SchemaErrorKind::WrongType {
                expected: "hash",
                actual: "array"
            }
            .into()
        );
    }
}
