---
description: Analyze a Rust source file, suggest test cases, then write them with create-tests
argument-hint: <file-path>
allowed-tools: Read, Edit, Bash, Skill
---

# Suggest Tests

Target file: $ARGUMENTS

File contents:
!`cat "$ARGUMENTS"`

## Step 1: Analyze

Identify every testable item in the file above:
- Public functions and methods (happy path and error branches)
- Edge cases: empty inputs, boundary values, `None`, empty collections
- Invariants that must hold

## Step 2: Present suggested test cases

Output a numbered list **before writing any code**:

```
1. `item_name` — scenario description
   Why: one sentence on what this validates
```

## Step 3: Write the tests

Invoke the `create-tests` skill to write all suggested test cases for $ARGUMENTS.
