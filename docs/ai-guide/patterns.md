---
title: "AI Guide: Common Patterns"
category: ai-guide
audience: [ai-agent]
status: placeholder
last-verified: 2026-03-19
depends-on: [docs/spec/types.md, docs/spec/assurance-ledger.md, docs/spec/modules.md, docs/spec/errors.md]
---

# Common Patterns

Patterns that recur in idiomatic ish code. Use these as templates when generating code.

## Builder Pattern (Low-Assurance)

```
let config = {}
config.host = 'localhost'
config.port = 8080
config.debug = true
start_server(config)
```

## Builder Pattern (High-Assurance)

```ish
type ServerConfig = {
    host: String
    port: i32
    debug: bool
}

let config: ServerConfig = {
    host: 'localhost',
    port: 8080,
    debug: true
}
start_server(config)
```

## Optional Values

```ish
// Low-assurance
let name = get_name()    // might be null
if name != null { println(name) }

// High-assurance
let name: String? = get_name()
if name != null {
    // name narrowed to String here
    println(name)
} else {
    println("anonymous")
}
```

## Error Handling

Errors in ish are ordinary objects with entry-based identity. Throw an object with a `message` property — the ledger automatically adds an `Error` entry. Add a `code` property to get a `CodedError` entry.

```ish
// Throw a simple error (auto-gets Error entry)
throw { message: "something went wrong" }

// Throw a coded error (auto-gets CodedError entry)
throw { message: "file not found", code: "E003" }

// Catch and inspect
try {
    risky_operation()
} catch (e) {
    if is_error(e) {
        println(error_message(e))
        let code = error_code(e)    // null if not CodedError
        if code != null { println("Code: " + code) }
    }
}
```

## Union Types for Error Results

```ish
fn parse(input: String) -> i32 | ParseError {
    // ...
}

let result = parse("42")
if is_type(result, i32) {
    use(result)
} else {
    report(result)
}
```

## Structural Subtyping

```ish
// Any object with a .name field works
fn greet(thing: { name: String }) {
    println("Hello, " + thing.name)
}

@[Open]
let person = { name: "Alice", age: 30 }
greet(person)    // OK — Open object, extra fields allowed
```

## Open and Closed Objects

```ish
// Object literals are closed by default
let point = { x: 0, y: 0 }    // Closed — only x and y

// Explicitly open for dynamic data
@[Open]
let config = load_config()     // Open — extra properties allowed

// Type declarations are indeterminate
type Settings = { theme: String, font_size: i32 }
// Open or closed depends on context/annotation
```

## Type Narrowing

```ish
let value: i32 | String | null = get_value()

if value == null {
    println("nothing")
} else if is_type(value, String) {
    // value narrowed to String
    println("string: " + value)
} else {
    // value narrowed to i32
    println("number: " + to_string(value))
}
```

## String Patterns

Use single quotes for literal strings and double quotes for interpolation:

```
// Literal strings (no interpolation)
let sql = 'SELECT * FROM users'
let regex = '(\d+)\s+'
let json = '{"key": "value"}'

// Interpolation
let msg = "Hello, {name}!"
let path = "Home: $HOME"

// Multiline
let query = """
    SELECT *
    FROM {table}
    WHERE id = {id}
    """

// Char literals
let ch = c'A'
let newline = c'\n'
```

---

## Referenced by

- [docs/ai-guide/INDEX.md](INDEX.md)
- [docs/ai-guide/orientation.md](orientation.md)
