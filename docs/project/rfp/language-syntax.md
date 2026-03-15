---
title: "RFP: Language Syntax"
category: rfp
audience: [all]
status: stable
last-verified: 2026-03-14
depends-on: [docs/spec/syntax.md, docs/spec/types.md, docs/spec/execution.md, docs/spec/modules.md, GLOSSARY.md]
---

# RFP: Language Syntax

*Converted from `syntax` on 2026-03-14.*

---

## Overview

ish needs a concrete syntax for the parts of the language that are common to most programming languages:

- Control flow
- Comments
- Function declaration and invocation
- Closures
- Primitives
- Variables
- Required or optional semicolons
- Expressions
- Literals
- Objects
- Operations
- Statements
- Visibility
- And other standard language constructs

## Requirements

### Phased Implementation

The syntax should be specified in such a way that it can be cleanly separated into chunks for separate implementation. The chunks should be organized by implementation priority, not by feature similarity.

### Source Languages

The syntax should draw primarily on four sources:

1. **Rust syntax**
2. **TypeScript syntax**
3. **Existing syntax in the ish documentation** (see `docs/spec/` and `docs/user-guide/`)
4. **Existing syntax in the old prototype** (see `ish-workspace/ish-parser/src/ish.pest`)

Where these sources disagree with each other, provide alternatives with pros and cons, internet consensus, and a recommendation. Where the sources do not disagree, no alternatives are needed.

### Shell Mode vs. Command-Line Mode

ish is intended to have both a **shell mode** and a **command-line mode**:

- **Shell mode:** The user executes one statement at a time interactively.
- **Command-line mode:** The user invokes ish with a one-line program (e.g., "build the project in the current directory") and ish runs that program and exits.

This creates tension because the shell's need to invoke executables without special syntax interferes with programming language constructs. Alternative solutions to this problem are needed, along with research on how other languages handle it.

One approach under consideration: the language has a shell-like syntax, with a few reserved words (`use`, `mod`, `fn`, etc.) that put the parser into a programming-language mode until a terminator is seen. This was partially explored in the old prototype but not completed.

### Project Definition in Shell Mode

A mechanism is needed to handle project definition in shell mode. It must be possible to import packages and call their functions interactively, so there needs to be a way to do that from the shell.

### Parser Strategy

The ish parser is intended to be implemented similarly to the old prototype, using [pest](https://pest.rs/), with a grammar that deliberately "recognizes" poorly formed ish and accepts it. This allows the language processor to provide good error messages rather than relying on whatever the pest framework produces. This approach should be evaluated and alternatives proposed, with pros and cons and a recommendation.

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
