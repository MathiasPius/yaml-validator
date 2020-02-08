use std::convert::TryFrom;
use std::fs::read;
use std::path::PathBuf;
use structopt::StructOpt;
use yaml_validator::{Context, Validate, Yaml, YamlLoader};

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
        long = "schema",
        help = "Schemas to include in context to validate against. Schemas are added in order, but do not validate references to other schemas upon loading."
    )]
    schemas: Vec<PathBuf>,

    #[structopt(short, long, help = "URI of the schema to validate the files against.")]
    uri: String,

    #[structopt(
        parse(from_os_str),
        help = "Files to validate against the selected schemas."
    )]
    files: Vec<PathBuf>,
}

fn read_file(filename: &PathBuf) -> Result<String, Error> {
    let contents = read(filename).map_err(|e| {
        Error::FileError(format!(
            "could not read file {}: {}\n",
            filename.to_string_lossy(),
            e
        ))
    })?;

    let utf8 = String::from_utf8_lossy(&contents).parse().map_err(|e| {
        Error::FileError(format!(
            "file {} did not contain valid utf8: {}\n",
            filename.to_string_lossy(),
            e
        ))
    })?;

    Ok(utf8)
}

fn load_yaml(filenames: &Vec<PathBuf>) -> Result<Vec<Yaml>, Vec<Error>> {
    let (yaml, errs): (Vec<_>, Vec<_>) = filenames
        .iter()
        .map(|file| {
            read_file(&file)
                .and_then(|source| YamlLoader::load_from_str(&source).map_err(Error::from))
        })
        .partition(Result::is_ok);

    if !errs.is_empty() {
        Err(errs.into_iter().map(Result::unwrap_err).collect())
    } else {
        Ok(yaml.into_iter().map(Result::unwrap).flatten().collect())
    }
}

// Ideally this would just be the real main function, but since errors are
// automatically printed using the Debug trait rather than Display, the error
// messages are not very easy to read.
pub(crate) fn actual_main(opt: Opt) -> Result<(), Error> {
    if opt.schemas.is_empty() {
        return Err(Error::ValidationError(
            "no schemas supplied, see the --schema option for information\n".into(),
        ));
    }

    if opt.files.is_empty() {
        return Err(Error::ValidationError(
            "no files to validate were supplied, use --help for more information\n".into(),
        ));
    }

    let yaml_schemas = load_yaml(&opt.schemas).map_err(Error::Multiple)?;
    let context = Context::try_from(&yaml_schemas)?;

    let schema = {
        if let Some(schema) = context.get_schema(&opt.uri) {
            schema
        } else {
            return Err(Error::ValidationError(format!(
                "schema referenced by uri `{}` not found in context\n",
                opt.uri
            )));
        }
    };

    let documents = opt
        .files
        .iter()
        .zip(load_yaml(&opt.files).map_err(Error::Multiple)?);

    for (name, doc) in documents {
        schema.validate(&context, &doc).map_err(|err| {
            Error::ValidationError(format!(
                "{name}:\n{err}",
                name = name.to_string_lossy(),
                err = err
            ))
        })?;
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    match actual_main(opt) {
        Ok(()) => println!("all files validated successfully!"),
        Err(e) => {
            eprint!("{}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
