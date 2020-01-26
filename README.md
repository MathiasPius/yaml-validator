# yaml-validator
YAML validation using schemas written in yaml

## Supported datatypes
The schema format supports a very limited number of types that map very closely to the YAML specification:

 * `string`
    * `min_length: number` (optional)
    * `max_length: number` (optional)
 * `number`
 * `dictionary`
    * `key` (optional)
       * `type: <type>` should always be string really, will be removed.
    * `value` (optional)
       * `type: <type>` type of the value in the dictionary.
 * `list`
    * `type: <type>` required, but will be made optional.
 * `object`
    * `fields` struct with known fields (unlike a dictionary). List of:
       * `name: string`
       * `type: <type>`
 * `reference`
    * `uri: string` uri of the schema this property references

Here's an example schema file containing to interdependent schemas using a bit of all of the above types:

Source: [acquaintance.yaml](yaml-validator-cli/examples/acquaintance.yaml) and [phonebook.yaml](yaml-validator-cli/examples/phonebook.yaml)
```yaml
---
uri: examples/0.0.3/acquaintance
schema:
  - name: firstname
    type: string
    max_length: 20
  - name: age
    type: number
  - name: favorite_foods
    type: list
    inner:
      type: string
  - name: movie_scores
    type: dictionary
    key:
      type: string
    value:
      type: number

---
uri: examples/0.0.3/phonebook
schema:
  - name: friends
    type: list
    inner:
      type: reference
      uri: examples/0.0.3/acquaintance
  - name: colleagues
    type: list
    inner: 
      type: object
      fields:
        - name: name
          type: string
        - name: department
          type: string
```
And a sample yaml file we can validate with the above schema:
```yaml
---
friends:
  - firstname: John
    age: 58
    favorite_foods:
      - Spaghetti
      - Lasagna
    movie_scores:
      The Room: 2.3
      Good Will Hunting: 8.3
colleagues:
  - name: Peter
    department: HR
  - name: Harry
    department: Finance
```
Test the above yaml using [peopleiknow.yaml](yaml-validator-cli/examples/peopleiknow.yaml):
```bash
$ yaml-validator-cli \
    --schemas acquaintance.yaml \
    --schemas phonebook.yaml 
    peopleiknow.yaml
valid: "peopleiknow.yaml"
All files validated successfully!
```

## Command-line help information
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
All the example yaml files and schemas below can be found in the [yaml-validator-cli/examples](yaml-validator-cli/examples/) directory.
## Validating a single YAML file against a single schema
You can use the command line tool to test a single yaml file against a single schema, by first defining a schema file and a yaml file to test it against:

Schema: [person.yaml](yaml-validator-cli/examples/person.yaml)
```yaml
---
schema:
  - name: firstname
    type: string
  - name: age
    type: number
```

YAML-file: [johnsmith.yaml](yaml-validator-cli/examples/johnsmith.yaml)
```yaml
---
firstname: John
age: 58
```
Run the command with the above schema and user file:
```bash
$ yaml-validator-cli --schema person.yaml -- johnsmith.yaml
valid: "johnsmith.yaml"
All files validated successfully!
```

## Validating multiple files against a single schema
For this example, we'll re-use the files from before, but add some more people

YAML-file: [janedoe.yaml](yaml-validator-cli/examples/janedoe.yaml)
```yaml
---
firstname: Jane
age: 33
```

YAML-file: [malfoy.yaml](yaml-validator-cli/examples/malfoy.yaml)
```yaml
---
firstname: Malfoy
age: Thirty-five
```
Running the same command, but with the other people appended:
```bash
$ yaml-validator-cli --schemas person.yaml \
    johnsmith.yaml \
    janedoe.yaml \
    malfoy.yaml
valid: "johnsmith.yaml"
valid: "janedoe.yaml"
failed: "malfoy.yaml": $.age: wrong type, expected `number` got `String("Thirty-five")`
```
We see that *malfoy.yaml* does not conform to the provided schema, and our program has exited with an error.

## Validating against a context containing interdependent schemas
In this example we'll make use of the `reference` data type, which means we'll need to provide schemas we'll be referring to with a `uri` we can locate them by.

The [person.yaml](yaml-validator-cli/examples/person.yaml) file from the first examples already has a uri defined, it was just ommitted from the examples for simplicity's sake:
```yaml
---
uri: examples/0.0.3/person
schema:
  - name: firstname
    type: string
  - name: age
    type: number
```
We can therefore go on to define our other schema, which will contain a list of *persons*:

Schema: [listofpeople.yaml](yaml-validator-cli/examples/listofpeople.yaml)
```yaml
---
schema:
  - name: people
    type: list
    inner:
      type: reference
      uri: examples/0.0.3/person
```

Now we have to define a test file we can validate using the above schema:

YAML-file: [contacts.yaml](yaml-validator-cli/examples/contacts.yaml)
```yaml
---
people:
  - firstname: John
    age: 58
  - firstname: Jane
    age: 33
  - firstname: Malfoy
    age: Thirty-five
```
We can validate our contacts list, by specifying both the schemas necessary to validate it in order:

```bash
$ yaml-validator-cli \
    --schemas person.yaml \
    --schemas listofpeople.yaml \
    contacts.yaml
failed: "contacts.yaml": $.people[2].age: wrong type, expected `number` got `String("Thirty-five")`
```
once again *Malfoy* violates our schema with his stringified age, as we can tell from the error message telling us that the 3rd (because of zero-indexed arrays) entry in our *$.people* value is malformed.

**Note:** This "just works", because we supplied our "listofpeople.yaml" file last, which means it will be used as the schema to validate against by default. If we had reversed the order of the schemas, or if we are not sure about the order the will be loaded in, we can give our `listofpeople.yaml` struct a uri too, and specify it on the command line to make it explicit which schema we want our `contacts.yaml` file to validate against:

Schema: [listofpeople.yaml](yaml-validator-cli/examples/listofpeople.yaml)
```yaml
---
uri: examples/0.0.3/listofpeople
schema:
  - name: people
    type: list
    inner:
      type: reference
      uri: examples/0.0.3/person
```
Now in any order:
```bash
$ yaml-validator-cli \
    --schemas listofpeople.yaml \
    --schemas person.yaml \
    --uri examples/0.0.3/listofpeople
    contacts.yaml
failed: "contacts.yaml": $.people[2].age: wrong type, expected `number` got `String("Thirty-five")`
```
We of course still get the *Malfoy* error, since we haven't fixed our contacts.yaml list, but if you remove the --uri argument from our command, you'll be met with a completely different error:

```bash
failed: "yaml-validator-cli/examples/contacts.yaml": $: missing field, `firstname` not found
```
The message claims there's a missing `firstname` field in our root document, because it thinks `contacts.yaml` is supposed to conform to the `person.yaml` schema.