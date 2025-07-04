# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) that document significant technical decisions made during the development of Portico.

## What is an ADR?

An ADR is a document that captures an important architectural decision made along with its context and consequences. ADRs help:
- Future developers understand why decisions were made
- Teams avoid revisiting the same discussions
- New team members quickly understand the project's evolution

## ADR Format

Each ADR follows this template:

```markdown
# ADR-XXXX: [Short Title]

Status: **[Proposed | Accepted | Deprecated | Superseded by ADR-YYYY]**
Date: **YYYY-MM-DD**

---

## Context
[What is the issue that we're seeing that is motivating this decision?]

## Decision
[What is the change that we're proposing and/or doing?]

## Alternatives Considered
[What other options were evaluated? Why were they rejected?]

## Consequences
[What becomes easier or more difficult to do because of this change?]
```

## Creating a New ADR

1. Copy the template above
2. Number it sequentially (e.g., `adr-0002-example-decision.md`)
3. Fill in all sections
4. Submit via pull request for team review
5. Update status to "Accepted" once approved

## Current ADRs

- [ADR-0001: Initial MVP Design Decisions](adr-0001-chosen-frontend.md) - Core technology choices for the MVP

## Guidelines

- Keep ADRs concise and focused on a single decision
- Include enough context for someone unfamiliar with the discussion
- Be honest about trade-offs and limitations
- Reference related ADRs when decisions build on each other
- Mark ADRs as "Superseded" rather than deleting them
