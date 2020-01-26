# yaml-validator
YAML validation using schemas written in yaml

This project is really two parts:

 *  [yaml-validator](yaml-validator/), a Rust library for validating YAML files against schemas that are themselves defined in YAML.
 * [yaml-validator-cli](yaml-validator-cli/), a command-line interface using the `yaml-validator` library to validate YAML files

 Documentation for both are somewhat lacking at the moment, but [yaml-validator-cli](yaml-validator-cli/) is by far the most useable of the two, and contains a lot of examples for how to get started using it to write schemas.