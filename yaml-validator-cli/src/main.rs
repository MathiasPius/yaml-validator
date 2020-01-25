use yaml_validator::YamlSchema;
use std::path::PathBuf;
use std::fs::read;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "yaml-validator-cli", about = "YAML validator")]
struct Opt {
    #[structopt(parse(from_os_str), short, long)]
    schema: Vec<PathBuf>,

    #[structopt(parse(from_os_str))]
    files: Vec<PathBuf>
}

fn read_file(filename: &PathBuf) -> Result<String, std::io::Error> {
    Ok(String::from_utf8_lossy(&read(filename).unwrap()).parse().unwrap())
}

fn main() {
    let opt = Opt::from_args();
    for schemafile in opt.schema.iter() {
        let schema = YamlSchema::from_str(&read_file(&schemafile).expect("failed to load schemafile"));
        
        for yamlfile in opt.files.iter() {
            let yaml = read_file(&yamlfile).expect("failed to load yaml file");
            match schema.validate_str(&yaml) {
                Ok(()) => println!("{:?} valid!", &yamlfile),
                Err(e) => println!("{:?} validation failed: {}", &yamlfile, e)
            }
        }
    }
}
