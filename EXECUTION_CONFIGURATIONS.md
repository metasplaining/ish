# ish Execution Configurations

ish has several execution configurations, serving different purposes. Each represents a different tradeoff between footprint, startup speed, and capability.

---

## 1. Thin Shell

When ish is started without any configuration or arguments, it runs as a **thin shell**. This is the smallest and most lightweight configuration.

- Accepts command-line input, interprets it, and processes it immediately.
- The shell, parser, virtual machine, and module loader are all loaded, but these are small, leaving ish with a minimal disk and memory footprint.
- Function declaration is supported, but lightweight — no code analysis or generation is performed. No semantic checking is done when functions are declared.
- This is the entry point for streamlined ish (see [README.md](README.md)).

## 2. Fat Shell

When `import` statements are invoked from the thin shell, ish downloads modules and loads them into memory.

- Imported modules increase ish's memory footprint.
- Because modules have been optimized and compiled during the module generation process, once loaded they execute very quickly.
- The fat shell bridges the gap between interactive use and compiled performance: the shell itself is interpreted, but imported code runs at compiled speed.

## 3. Compiler

Some of the modules that can be loaded are the **code analyzer** and the **compiler**. Once these have been loaded, ish can optimize and compile code, producing three kinds of output:

1. **Local code** — loaded into the current process for immediate use.
2. **Modules** — saved for later import in a different process.
3. **Executables** — saved as standalone programs.

This is the configuration used to build encumbered ish (see [README.md](README.md)).

## 4. Executable

When ish generates an executable, it is compiled down to just the modules the program needs. Typically, this means the thin shell modules — shell, parser, virtual machine, and module loader — are excluded. The result is a standalone executable with no ish interpreter overhead.