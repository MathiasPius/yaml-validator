# yaml-validator-cli
Command-line interface for validating YAML files using schemas written in yaml.

Quick Links:
* [Supported Datatypes](#supported-datatypes)
* [Examples](#examples)

<details><summary>Command-line help information</summary>
<p>

```
yaml-validator-cli 0.1.0
    Command-line interface to the yaml-validator library.
    Use it to validate YAML files against a context of any number of cross-referencing schema files.
    The schema format is proprietary, and does not offer compatibility with any other known YAML tools

USAGE:
    yaml-validator-cli [OPTIONS] --uri <uri> [--] [files]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --schema <schemas>...    Schemas to include in context to validate against. Schemas are added in order, but do
                                 not validate references to other schemas upon loading.
    -u, --uri <uri>              URI of the schema to validate the files against.

ARGS:
    <files>...    Files to validate against the selected schemas.
```
</p></details>

## Currently Supported datatypes
The schema format supports a very limited number of types that map very closely to the YAML specification:

 * `string` utf8-compliant string
 * `integer` i64 integer
 * `hash` (also know as `dictionary` or `hashmap`) that maps `string ➞ <type>` as defined in `items`
    * `items: <type>` (optional) type of the values in the hash
 * `array` array of items of type `<type>`
    * `items: <type>` (optional) type of the values in the array.
 * `object` struct with known fields (unlike a hash).
    * `items` array of fields and their types as below
       * `<name>: <type>`
 * `$ref: <uri>` reference to schema in same context identified by `<uri>`

# Examples
All of the examples below can also be found in the [examples/](../examples/) directory.

<details><summary>Using nested schemas through references</summary>
<p>

We can define a `person` object and later refer to it by its uri in a different schema `phonebook`:

```yaml
# phonebook.yaml
---
uri: person
schema:
  type: object
  items:
    name:
      type: string
    phone:
      type: integer

---
uri: phonebook
schema:
  type: object
  items:
    phonebook:
      type: array
      items:
        $ref: person
```

Source: [examples/nesting/schema.yaml](../examples/nesting/schema.yaml)

We can then use the above schema to validate a yaml document as defined here:

```yaml
# mybook.yaml
---
phonebook:
  - name: timmy
    phone: 123456
  - name: tammy
    phone: 987654
```
Source: [examples/nesting/mybook.yaml](../examples/nesting/mybook.yaml)

... Using the `yaml-validator-cli` as follows:

```bash
$ yaml-validator-cli --schema phonebook.yaml --uri phonebook -- mybook.yaml
all files validated successfully!
```
---

</p></details>


<details><summary>Referencing schemas across file boundaries</summary>
<p>

All schemas given using the `--schema` commandline option are all loaded into the same context, so referencing a schema defined in a separate file is exactly the same as if they had been defined in the same file.

```yaml
# person-schema.yaml
---
uri: person
schema:
  type: object
  items:
    name:
      type: string
    phone:
      type: integer
```

Source: [examples/multiple-schemas/person-schema.yaml](../examples/multiple-schemas/person-schema.yaml)

```yaml
# phonebook-schema.yaml
---
uri: phonebook
schema:
  type: object
  items:
    phonebook:
      type: array
      items:
        $ref: person
```
Source: [examples/multiple-schemas/phonebook-schema.yaml](../examples/multiple-schemas/phonebook-schema.yaml)

Validate the following yaml document against our schemas above:

```yaml
# mybook.yaml
---
phonebook:
  - name: timmy
    phone: 123456
  - name: tammy
    phone: 987654
```
Source: [examples/multiple-schemas/mybook.yaml](../examples/multiple-schemas/mybook.yaml)

... Using the `yaml-validator-cli` as follows:

```bash
$ yaml-validator-cli                \
    --schema phonebook-schema.yaml  \
    --schema person-schema.yaml     \
    --uri phonebook                 \
    mybook.yaml
all files validated successfully!
```
---

</p></details>