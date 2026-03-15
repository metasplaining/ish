---
title: "AI Guide: Common Patterns"
category: ai-guide
audience: [ai-agent]
status: placeholder
last-verified: 2026-03-10
depends-on: [docs/spec/types.md, docs/spec/modules.md]
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

```
type ServerConfig = {
    host: String
    port: Int
    debug: Bool
}

let config: ServerConfig = {
    host: 'localhost',
    port: 8080,
    debug: true
}
start_server(config)
```

## Optional Values

```
// Low-assurance
let name = get_name() // might be nil
if name { print(name) }

// High-assurance
let name: String? = get_name()
match name {
    Some(n) -> print(n)
    None -> print('anonymous')
}
```

## Union Types for Error Handling

```
fn parse(input: String) -> Int | ParseError {
    // ...
}

match parse('42') {
    Int(n) -> use(n)
    ParseError(e) -> report(e)
}
```

## Structural Subtyping

```
// Any object with a .name field works
fn greet(thing: { name: String }) {
    print("Hello, {thing.name}")
}

greet({ name: 'Alice', age: 30 })  // OK — extra fields ignored
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
