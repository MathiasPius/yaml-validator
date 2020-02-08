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

## Currently supported datatypes
The schema format supports a very limited number of types that map very closely to the YAML specification:

 * `string` utf8-compliant string
 * `integer` i64 integer
 * `real` f64 floating point value
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

<details><summary>Using references to avoid deeply nested and non-reusable structures</summary>
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


<details><summary>Combining all the different types with nested references</summary>
<p>

We can define a schema in 3 levels as below, where a customer-list is defined as an array of customers, which in turn contain elements of their own, as well as references to a third schema 'car':

```yaml
# schema.yaml
---
uri: car
schema:
  type: object
  items:
    year:
      type: integer
    model:
      type: string
    extra features:
      type: array
      items:
        type: string
    price: 
      type: real

---
uri: customer
schema:
  type: object
  items:
    name:
      type: string
    cars:
      type: hash
      items:
        $ref: car

---
uri: customer-list
schema:
  type: array
  items:
    $ref: customer
```

Source: [examples/all-types/schema.yaml](../examples/all-types/schema.yaml)

Validate the following customer list document against the defined schema:

```yaml
# customers.yaml
---
- name: Teodor Fælgen
  cars:
    work:
      model: Ford T
      extra features:
        - gps
        - heated seats
      price: 200.00
    racing:
      model: Il Tempo Gigante
      extra features:
        - blood bank
        - radar
      price: 3000.00

- name: Lightning McQueen
  cars:
    himself:
      model: Stock
      extra features:
        - massive eyes instead of windows
        - arrogance
      price: 0.00
```

Source: [examples/all-types/customers.yaml](../examples/all-types/customers.yaml)


... Using the `yaml-validator-cli` as follows:

```bash
$ yaml-validator-cli                \
    --schema schema.yaml            \
    --uri customer-list             \
    customers.yaml
all files validated successfully!
```
---

</p></details>

<details><summary>Locating errors in documents</summary>
<p>
Error messages always contain the full path within the document, as well as the document name in which the validation error occurred. This lets you pretty easily track down the exact source of the error.

With a phonebook schema as follows:

```yaml
# schema.yaml
---
uri: person
schema:
  type: object
  items:
    name:
      type: string
    age: 
      type: integer

---
uri: phonebook
schema:
  type: array
  items:
    $ref: person
```
Source: Source: [examples/locating-errors/schema.yaml](../examples/locating-errors/schema.yaml)

We can validate our very non-compliant document defined as:

```yaml
# phonebook.yaml
- name: John
  age: 52
- name: Karen
  age: 12.5
- name: 200
  age: Jimmy
```
Source: Source: [examples/locating-errors/phonebook.yaml](../examples/locating-errors/phonebook.yaml)

Using yaml-validator-cli as follows:

```
$ yaml-validator-cli      \
    --schema schema.yaml  \
    --uri phonebook       \
     phonebook.yaml
phonebook.yaml:
#[1].age: wrong type, expected integer got real
#[2].age: wrong type, expected integer got string
#[2].name: wrong type, expected string got integer
```
The error message correctly tells us that there's an issue with the document `phonebook.yaml` supplied. Karen's age is a real, not an integer, and Jimmy's age and name have been switched.

Note: The `#` denotes the root of the document, `phonebook.yaml` in this case.

---

</p></details>