# ish Reasoning System

## Overview

Several parts of the ish system need to reason about code: the compiler, the static code analyzer, and the LSP server. Rather than each component implementing its own analysis logic, ish provides a single shared reasoning tool that services all of them.

## What the Reasoning Tool Analyzes

The reasoning tool answers questions about code such as:

1. Is a statement reachable?
2. Is a variable mutated?
3. Does a block of code throw errors?
4. Is a variable guaranteed to be initialized?

## Exposing the Reasoning Tool to the Language

It is proposed to expose the reasoning tool's interface to the language itself. This would allow developers to annotate their code with arbitrary assertions and queries, which the language processor would evaluate. It would also support plugin developers who want to extend the analyzer.

## Propositions

The building blocks of the reasoning system are logic propositions. Two kinds are supported:

### Atomic Propositions

Atomic propositions are the primitives of the reasoning system. Each is defined by a plugin — an ish function that takes an AST node (and perhaps some state) as input and returns a boolean result. The language provides an interface for defining new atomic proposition plugins.

An initial set of built-in atomic propositions includes:

1. Statement reachable
2. Variable mutated
3. Block might throw
4. Variable guaranteed to be initialized

### Compound Propositions

Compound propositions are formed by applying logical operations (`and`, `or`, `not`) to atomic or other compound propositions.

## Relationship to the Type System

One possibility is to implement the entire type system on top of this reasoning tool. The interfaces of the type system would be unified with those of the reasoning tool — effectively, the type system would be a special case of a more general facility for reasoning about code.