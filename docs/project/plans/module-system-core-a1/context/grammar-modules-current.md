*Extracted verbatim from [proto/ish-parser/src/ish.pest](../../../../proto/ish-parser/src/ish.pest) §Visibility, §Modules, §statement rule, §annotated_stmt, §keyword.*

## Visibility rule (current — to be replaced)

```pest
// --- Visibility ---
pub_modifier = { "pub" ~ ("(" ~ identifier ~ ")")? }

let_stmt = { pub_modifier? ~ "let" ~ mut_kw? ~ identifier ~ (":" ~ type_annotation)? ~ "=" ~ expression }
fn_decl = {
    pub_modifier? ~ async_kw? ~ "fn" ~ identifier ~ generic_params? ~ "(" ~ param_list? ~ ")" ~ ("->" ~ type_annotation)? ~ block
}
type_alias = { pub_modifier? ~ "type" ~ identifier ~ "=" ~ type_annotation }
```

## Module rules (current — to be replaced)

```pest
// --- Modules ---
use_stmt = { "use" ~ module_path }
module_path = { identifier ~ ("::" ~ identifier)* }
mod_stmt = { pub_modifier? ~ "mod" ~ identifier ~ block? }
```

## statement rule (current — mod_stmt entry to be removed, declare_block and bootstrap_stmt to be added)

```pest
statement = _{
    annotated_stmt |
    standard_def |
    entry_type_def |
    fn_decl |
    let_stmt |
    if_stmt |
    while_stmt |
    for_stmt |
    return_stmt |
    yield_stmt |
    throw_stmt |
    try_catch |
    with_block |
    defer_stmt |
    match_stmt |
    type_alias |
    use_stmt |
    mod_stmt |
    assign_stmt |
    expression_stmt |
    shell_command |
    unterminated_block
}
```

## annotated_stmt rule (current — mod_stmt reference to be removed)

```pest
annotated_stmt = { (annotation ~ NEWLINE*)+ ~ (fn_decl | let_stmt | type_alias | mod_stmt | block | while_stmt | for_stmt) }
```

## keyword rule (current — must add priv, pkg, declare, bootstrap; remove mod)

```pest
keyword = {
    ("let" | "mut" | "fn" | "if" | "else" | "while" | "for" | "in" |
     "return" | "true" | "false" | "null" | "and" | "or" | "not" |
     "match" | "use" | "mod" | "pub" | "type" | "standard" | "entry" |
     "try" | "catch" | "finally" | "throw" | "with" | "defer" |
     "async" | "await" | "spawn" | "yield" |
     "break" | "continue") ~ !(ASCII_ALPHANUMERIC | "_")
}
```
