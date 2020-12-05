This example is designed to *fail* the schema validation, it is not an example of a valid schema/yaml file setup!

Use the following command to show an example of the error codes produced by this tool, when validation fails:
```shell
yaml-validator-cli        \
    --schema schema.yaml  \
    --uri phonebook       \
    phonebook.yaml
```