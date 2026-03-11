---
title: "History: Documentation System"
category: history
audience: [human-dev, ai-agent]
status: stable
last-verified: 2026-03-10
depends-on: [docs/project/decisions/001-documentation-structure.md]
---

# 2026-03-10: Documentation System

## Summary

Established the ish project's documentation infrastructure. The original `DOCUMENTATION.md` file described requirements for a comprehensive, agentic-AI-friendly documentation system. A proposal was created (`DOCUMENTATION_PROPOSAL.md`), reviewed, and implemented.

## Background

The ish project's documentation consisted of root-level Markdown files without consistent structure:
- `TYPES.md`, `MODULES.md`, `REASONING.md`, `AGREEMENT.md`, `EXECUTION_CONFIGURATIONS.md` — language spec fragments
- `*_TODO.md` files — open questions and unresolved design issues
- `README.md` — project overview with embedded spec content
- `proto/ARCHITECTURE.md` — prototype architecture in a single file

This was insufficient for a project where AI agents are primary documentation consumers.

## Requirements Identified

The original request identified 10 documentation types needed:
1. GitHub summary for newcomers
2. Human developer user guide
3. AI developer user guide with playbooks
4. Language specification
5. Architecture specification
6. Contributor documentation
7. Development history
8. Organized open questions and TODOs
9. Roadmap and project plan
10. Maturity/implementation status tracking

Key principles:
- Context management is critical for agentic AI productivity
- Minimize context window size while providing complete information
- Minimize redundancy to prevent inconsistency
- Clear standards for incremental updates
- Audit mechanisms for consistency checking

## Decisions Made

See [ADR-001](../decisions/001-documentation-structure.md) for the full decision record.

Key choices:
- Structured `docs/` directory with category subdirectories
- YAML frontmatter on all files
- 500-line soft limit per file
- Layered loading strategy (L0–L3) for AI agents
- Hybrid open questions (in spec files AND consolidated file, cross-linked)
- Documentation debt tracked separately
- Backward reference sections required
- Audit scripts for links, frontmatter, staleness, glossary

## Participants

- Human developer: provided requirements, reviewed proposal, made decisions
- AI developer: researched best practices, created proposal, implemented infrastructure

---

## Referenced by

- [docs/project/history/INDEX.md](INDEX.md)
