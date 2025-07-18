# DY library

The DY library is a Rust crate to quickly define a new delightful human writable syntax, optimized for redaction. This is a halfway between Markdown and YAML.

This is built to be used by the [PLX](https://github.com/samuelroland/plx) project. Future usage is also planned in [Delibay](https://samuelroland.github.io/mvp).

## Example

<!-- // TODO fix with new examples here ? -->

### Definition of a programming exercise
With a pipe implementation in a custom shell started in Qemu via a `./st` script. For PLX.

<table><tr><td>
<img style="border: 2px gray solid; padding: 10px;" src="https://raw.githubusercontent.com/samuelroland/tb-docs/6320bc2f87f28a4e0aaacd65f5dcfcdfa4288e5a/report/sources/plx-dy-all.svg" height="550" />
</td></tr></table>

### Definition of a multiple choice question
For Delibay.

<img src="https://raw.githubusercontent.com/samuelroland/tb-docs/6320bc2f87f28a4e0aaacd65f5dcfcdfa4288e5a/report/imgs/mcq.svg" height="150" />

## Error display

Considering this pretty wrong definition of a course in a `course.dy` file
```dy
code YEP
course PRG1
goal Learn C++
course PRG2
```

The parser is able to to generate this kind of errors list
```
Found 1 item in course.dy with 3 errors.

Error at course.dy:0:0
code YEP
^^^^ The 'code' key can be only used under a `course`

Error at course.dy:1:0
course PRG1
| Missing required key 'code'

Error at course.dy:3:0
course PRG2
^^^^^^ The 'course' key can only be used once in the document root
```

