# Log Analyzer

Coding task performed May 2022 in context of an interview process.

Copyright (C) 2022 Sebastian Müller

## Task

### Log Files

The task was to implement a CLI tool that is able to parse log files.

Log files are defined as text files that contain an arbitrary number of lines
consisting of JSON objects. It is guaranteed that each JSON object has a `type`
key providing a String identifier. There might be additional arbitrary
key-value pairs in each JSON object.

Example log file:

```json
{"type": "Foo", "id": 3, "cluster": -3}
{"type": "Bar", "error": 1}
{"type": "Foo", "name": "titan", "calibration": 3.141}
```

### Output

The CLI in scope of this task should be able to output a table, providing
information on the different `type`s in the log file and the accumulated size
in bytes of each such `type`. In the example above, the output should look
similar to this:

```text
Type | Size
-----------
Foo  | 93
Bar  | 27
```
