# yaml-validator
YAML validation using schemas written in yaml

```
yaml-validator-cli 0.0.2
    Command-line interface to the yaml-validator library.
    Use it to validate YAML files against a context of any number of cross-referencing schema files.
    The schema format is proprietary, and does not offer compatibility with any other known YAML tools

USAGE:
    yaml-validator-cli [OPTIONS] [--] [files]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --schemas <schemas>...    Schemas to include in context to validate against. Schemas are added in order, but do
                                  not validate references to other schemas upon loading.
    -u, --uri <uri>               URI of the schema to validate the files against. If not supplied, the last schema
                                  added will be used for validation.

ARGS:
    <files>...    Files to validate against the selected schemas.
```

# Examples
## Validating a single YAML file against a single schema
You can use the command line tool to test a single yaml file against a single schema, by first defining a schema file and a yaml file to test it against:

Schema: examples/person.yaml
```yaml
---
schema:
  - name: firstname
    type: string
  - name: age
    type: number
```

YAML-file: examples/johnsmith.yaml
```yaml
---
firstname: John
age: 58
```
Run the command with the above schema and user file:
```bash
$ yaml-validator-cli --schema examples/person.yaml -- examples/johnsmith.yaml
valid: "examples/johnsmith.yaml"
All files validated successfully!
```
