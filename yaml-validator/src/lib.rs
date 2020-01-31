use std::convert::TryFrom;
use yaml_rust::Yaml;

#[cfg(test)]
mod tests;
mod error;

struct SchemaObject {
    items: Vec<Property>
}

enum PropertyType {
    Object(SchemaObject),
}

struct Property {
    name: String,
    schematype: PropertyType,
}

impl TryFrom<Yaml> for SchemaObject {
    fn try_from(yaml: &Yaml) -> Result<Self, Self::Error> {
        
    }
}