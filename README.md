# ish

**ish** is a general-purpose programming language designed to support a wide range of language tradeoffs within a single unified language.

## Motivation

> "All happy families are alike; each unhappy family is unhappy in its own way." — Leo Tolstoy

Most modern programming languages are quite similar to one another. They share the same features — types, variables, objects, conditional logic, functions — and often share syntax for them too. This convergence is natural: when one language introduces a useful feature or convenient syntax, others incorporate it. But where languages diverge, it is usually for one of two reasons:

1. A language introduced something unfortunate early on and is now stuck with it for backward compatibility.
2. A language introduced a genuinely useful feature that comes with a tradeoff — a cost that only makes sense in certain circumstances.

| Language   | Strength                                          | Tradeoff                                                  |
|------------|---------------------------------------------------|-----------------------------------------------------------|
| Rust       | Compiles to small, fast executables                | Requires attention to many low-level details               |
| Java       | Abstracts memory management; platform-independent  | Requires a heavyweight runtime                             |
| TypeScript | Very flexible typing                               | Never represents objects with efficient memory layouts     |

Most software ecosystems have multiple niches, each favoring a different set of tradeoffs. Developers are forced either to pick a single language that is suboptimal for many niches, or to adopt a polyglot approach — using the best language for each niche at the cost of maintaining competency in many languages.

**ish** aims to be a single programming language that supports many sets of language tradeoffs well.

## Approach

Think of ish as a family of languages on a continuum between **streamlined ish** on one end and **encumbered ish** on the other.

### Streamlined ish

Streamlined ish is approachable by anyone who knows at least one other programming language. It has all the standard features, and there is not much else a developer needs to know.

- Parsed at runtime
- Executed by an interpreter
- Error checking deferred until execution
- Memory is garbage collected
- Agreement checks are optional (see [AGREEMENT.md](AGREEMENT.md) for an explanation of agreement and marked language features)

### Encumbered ish

Encumbered ish sits at the intersection of Rust and TypeScript. It has every kind of early evaluation possible.

- Parsed at build time and compiled to an optimized binary
- Errors checked at build time as much as possible
- Developers must concern themselves with memory management
- Many language features are marked, requiring agreement (see [AGREEMENT.md](AGREEMENT.md))

### Execution configurations

The streamlined ↔ encumbered continuum manifests as four execution configurations, from lightweight to fully compiled. See [EXECUTION_CONFIGURATIONS.md](EXECUTION_CONFIGURATIONS.md) for details.

### Flexible configuration

Features may be independently configured as streamlined or encumbered. Configuration can be applied at the project level, but also much more narrowly — sometimes down to individual variables and functions. This allows different codebases, and different parts of the same codebase, to adopt the level of encumbrance that best suits them:

- **Prototype code** can be minimally encumbered.
- **Production code** can be heavily encumbered.
- **Non-performance-critical production code** can have full agreement checking without performance optimization encumbrance.

## Features

### Polymorphism

Existing languages implement polymorphism in several ways. Ordered roughly from most performant / most constrained to least performant / least constrained:

| Strategy              | Description                                                                                                                                      |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------|
| None                  | Data is stored in a fixed format determined at build time.                                                                                       |
| Enumerated            | Data can be one of several variant formats, each determined at build time. Code matches on the variant and executes case logic.                   |
| Monomorphized         | Code is written against interfaces. The build tool generates a specialized (monomorphized) variant of each function for each conforming format.   |
| Virtual method table  | Code is written against interfaces. The build tool attaches metadata to data records, enabling functions to interpret the data at runtime.        |
| Associative array     | Each record is stored as a hash table, allowing an arbitrary set of properties per object at runtime.                                            |

ish supports all of these strategies. In general, the implementation detail is hidden from the developer. The ish language processor chooses the highest-performing strategy for which all constraints are met. For example, interpreted ish always uses associative arrays because there is no build step at which to choose a more constrained option. The build can be encumbered to fail with a descriptive error message if the constraints for a particular strategy are not met.

### Memory management

ish supports four memory management models, again ordered from most performant / most constrained to least:

| Model              | Description                                                                                                                                                     |
|--------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Stack              | A fixed-size slot is allocated in the function's stack frame. The variable is deallocated when the function returns.                                             |
| Heap (owned)       | Space is allocated on the heap with exactly one owning pointer. The variable is deallocated when the pointer goes out of scope.                                  |
| Reference counted  | Space is allocated on the heap. A reference count tracks pointers; the variable is deallocated when the count reaches zero.                                      |
| Garbage collected  | Space is allocated on the heap. A mark-and-sweep garbage collector deallocates unreachable variables.                                                            |

The ish language processor chooses the highest-performing model for which all constraints are met. For example, interpreted ish always uses garbage collection. The stack is used when a variable's size is known at build time and the variable only exists for the lifetime of a single function call. The build can be encumbered to fail with a descriptive error message if the constraints for a particular model are not met.

### Modules

ish code is organized into modules, with each `.ish` file defining a module. Symbols are private by default and exposed via visibility directives (`pub(self)`, `pub(super)`, `pub(project)`, `pub(global)`, `pub(in path)`). Projects build into packages that can be distributed as annotated ASTs or compiled object code. See [MODULES.md](MODULES.md) for the full module system specification.

### Reasoning about code

Reasoning about code is core to many aspects of ish. Code can be annotated with metadata that influences how the language processor reasons about it. See [REASONING.md](REASONING.md) for the reasoning system specification.

## Optimization Philosophy

The ish processor reserves the right to be inefficient.

The **interpreter** does everything in the most general (least optimized) way, because it must:

1. All metadata is preserved. Nothing is erased.
2. All memory management is done through garbage collection.
3. All objects are implemented as associative arrays.
4. All consistency checks are performed at runtime.

The **compiler** guarantees that it can do anything the interpreter can do, by doing it the same way. The compiler may find a more efficient strategy, but if it cannot, it always falls back to the interpreter's approach.

**Encumbrance does not directly make code more efficient.** If the compiler is able to find an optimization for encumbered code, it can find that same optimization for streamlined code. Instead, encumbering code to require a particular optimization causes the build to **fail with a descriptive error message** when the compiler cannot apply that optimization. This gives the developer the choice of either switching to streamlined mode or reorganizing their code so the compiler can optimize it.

Put differently: encumbering code does not make it faster — it makes concerns that were previously abstracted away suddenly visible. It is the developer's job to address those concerns.

## Internals

Just enough of ish is written in Rust to make streamlined ish work. The rest is written in ish itself. This accomplishes two objectives:

1. **Productivity.** Rust forces attention on low-level details that are not important for most of the ish codebase. Developing in streamlined ish improves productivity.
2. **Dogfooding.** Developing in ish provides real-time feedback on the language's usability. If the language is hard to use, there is no better way to discover that than by using it.

### Core modules

| Module           | Responsibility                                                                                                                                              |
|------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| AST              | Defines the abstract syntax tree primitives (data types and operations such as if/else, variable declaration/assignment, function declaration/call). Exposes primitive types, construction factories, and JSON serialization/deserialization. |
| Virtual machine  | Executes AST programs in interpreted mode.                                                                                                                   |
| Parser           | Parses ish source code into an AST.                                                                                                                          |
| Code analyzer    | Analyzes ASTs and annotates them with metadata.                                                                                                              |
| Rust generator   | Translates ASTs into Rust source code.                                                                                                                       |
| Linker           | Orchestrates the linking of code into the virtual machine.                                                                                                   |
| Shell            | Binds the other modules together into a single executable.                                                                                                   |

