use std::fs::read;
use std::path::PathBuf;
use structopt::StructOpt;
use yaml_validator::{YamlContext, YamlSchema};

mod error;
use error::Error;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "yaml-validator-cli",
    about = "    Command-line interface to the yaml-validator library.
    Use it to validate YAML files against a context of any number of cross-referencing schema files.
    The schema format is proprietary, and does not offer compatibility with any other known YAML tools"
)]
struct Opt {
    #[structopt(
        parse(from_os_str),
        short,
        long,
        help = "Schemas to include in context to validate against. Schemas are added in order, but do not validate references to other schemas upon loading."
    )]
    schemas: Vec<PathBuf>,

    #[structopt(
        short,
        long,
        help = "URI of the schema to validate the files against. If not supplied, the last schema added will be used for validation."
    )]
    uri: Option<String>,

    #[structopt(
        parse(from_os_str),
        help = "Files to validate against the selected schemas."
    )]
    files: Vec<PathBuf>,
}

fn read_file(filename: &PathBuf) -> Result<String, std::io::Error> {
    Ok(String::from_utf8_lossy(&read(filename).unwrap())
        .parse()
        .unwrap())
}

fn secret_main(opt: &Opt) -> Result<(), Error> {
    let mut context = YamlContext::new();

    for schemafile in opt.schemas.iter() {
        let content = read_file(&schemafile)?;
        context.add_schema(YamlSchema::from_str(&content));
    }

    let schema = {
        if let Some(uri) = &opt.uri {
            if let Some(schema) = context.lookup(&uri) {
                schema
            } else {
                panic!("Schema referenced by uri `{}` not found in context", uri);
            }
        } else {
            if let Some(schema) = context.schemas().last() {
                schema
            } else {
                panic!("No schemas supplied, see the --schema option for information");
            }
        }
    };

    for yamlfile in opt.files.iter() {
        let yaml = read_file(&yamlfile)?;
        schema
            .validate_str(&yaml, Some(&context))
            .map_err(|e| Error::ValidationError(format!("{:?}: {}", yamlfile, e)))?;
        println!("valid: {:?}", &yamlfile);
    }

    Ok(())
}

fn main() {
    let opt = Opt::from_args();

    match secret_main(&opt) {
        Ok(()) => println!("All files validated successfully!"),
        Err(e) => {
            println!("failed: {}", e);
            std::process::exit(1);
        }
    }
}
