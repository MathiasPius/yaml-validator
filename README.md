# yaml-validator [![Test Status]][library tests] [![Latest Version]][crates.io] [![Docs]][docs.rs]

[Test Status]: https://github.com/MathiasPius/yaml-validator/workflows/library-tests/badge.svg
[library tests]: yaml-validator/src/tests.rs
[Latest Version]: https://img.shields.io/crates/v/yaml-validator
[crates.io]: https://crates.io/crates/yaml-validator
[Docs]: https://docs.rs/yaml-validator/badge.svg
[docs.rs]: https://docs.rs/yaml-validator

YAML validation using schemas written in yaml

This project is really two parts:

 *  [yaml-validator](yaml-validator/), a Rust library for validating YAML files against schemas that are themselves defined in YAML.
 * [yaml-validator-cli](yaml-validator-cli/), a command-line interface using the `yaml-validator` library to validate YAML files

 Documentation for both are somewhat lacking at the moment, but [yaml-validator-cli](yaml-validator-cli/) is by far the most useable of the two, and contains a lot of examples for how to get started using it to write schemas.

 