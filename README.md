# hazel-rs

A utility to watch and organize folders inspired by Hazel for MacOS.

This is very much an alpha version. No guarantees.

## Getting started

- Create a `jobs.yaml` file
- Define a job
  ```yaml
  - name: Sort Documents
    source: "~/Documents"
    pattern: "/{month:created}/{day:created}"
  ```
- Run job: `hazel-rs --config jobs.yaml`

## Renaming Patterns

Jobs contain patterns that tell the application how the items within the source directory should be organized. The pattern is
the parsed, the variables replaced and then the resulting string is appended to the source or destination directory, if one is provided.

For example, this pattern sorts a directory into subdirectories by the month and year created:

`{month:created}/{year:created}` => `2023/03/<item>`
`{kind}/{size[500M, 1G, 10G]}` => `video/{500M,1G,10G}/<item>`

### Variables

Variables are used to inject information from the current item into the path. If an item does not provide information
for that variable, it is ignored.

Variables can be defined in the pattern in the following structure:

`{token:specifier[thresholds]:modifer}`

- `token` is the field data you want to insert
- `specifier` is the specific type of date to access
- `thresholds` enable grouping by the specifier
- `modifier` changes the output of the token

### Thresholds

For tokens that have thresholds, the items in the source directory will be grouped by those amounts.

For example..

Organize by the number of days since file or directory creation:

`/{month[30,60,90]:created} days` => `/{30,60,90} days/<item>`

Organize by file size:

`{kind}/{size[500M, 1G, 10G]}` => `video/{500M,1G,10G}/<item>`

That will create a folder for each threshold and put the items into the appropriate directory.

Note: These ultimately translate to `>30 || >60 || >90` so anything that isn't over the lowest threshold is ignored and
antyhing over the largest threshold is moved to that folder.

#### List of Tokens and Specifiers

- `month[thresholds]` (number or name)
  - `created`
  - `accessed`
  - `modified`
- `day[thresholds]` (number or name)
  - `created`
  - `accessed`
  - `modified`
- `year[thresholds]`
  - `created`
  - `accessed`
  - `modified`
- `size` (files only)
- `kind` (application,image,video,etc., files only)
- `mime` (organized into `type/subtype` folders, files only)
- `file` extension (files only)
