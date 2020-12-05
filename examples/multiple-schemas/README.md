This example demonstrates a schema specified in multiple separate files, and then using the following command line to validate a single yaml file ('mybook.yaml') against a single root uri (`phonebook` defined in [phonebook-schema.yaml](phonebook-schema.yaml)) within the context of these files:

```shell
yaml-validator-cli                  \
   --schema person-schema.yaml     \
   --schema phonebook-schema.yaml  \
   --uri phonebook                 \
   mybook.yaml
```