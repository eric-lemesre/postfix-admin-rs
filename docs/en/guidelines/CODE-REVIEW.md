> **Language:** English | [Francais](../fr/guidelines/CODE-REVIEW.md)

# Code Review Guidelines — postfix-admin-rs

## Objective

Ensure the quality, security and maintainability of the code through systematic review before integration.

---

## 1. Roles

### PR Author
- Ensures that the PR compiles, tests pass and Clippy is satisfied
- Provides a clear context in the description
- Responds to comments within a reasonable timeframe
- Does not merge their own PR (unless critical hotfix with agreement)

### Reviewer
- Verifies business logic and architectural consistency
- Identifies security, performance and maintainability issues
- Proposes constructive improvements (no personal criticism)
- Explicitly approves or requests modifications

---

## 2. Review Checklist

### Architecture and Design

- [ ] The code respects the separation of responsibilities between crates
- [ ] Dependencies between crates go in the right direction (no cycles)
- [ ] New public types are justified
- [ ] The Repository pattern is respected for data access
- [ ] Traits are used for abstraction (no direct coupling to implementations)

### Business Logic

- [ ] Business rules documented in specs are respected
- [ ] Edge cases are handled (null values, empty strings, numeric limits)
- [ ] Validations are done at the right level (DTO → service → repository)
- [ ] SQL transactions encompass atomic operations

### Security

- [ ] No SQL injection (parameterized queries only)
- [ ] No XSS (data escaped in templates)
- [ ] No secrets in code (passwords, API keys)
- [ ] CSRF protection on POST forms
- [ ] Passwords are never logged or serialized
- [ ] Secret comparisons are constant-time
- [ ] User inputs are validated

### Performance

- [ ] SQL queries are paginated (LIMIT/OFFSET)
- [ ] No N+1 queries
- [ ] Necessary indexes are created
- [ ] Unnecessary allocations are avoided (clones, String vs &str)
- [ ] Password hashing uses spawn_blocking

### Tests

- [ ] New public functions have tests
- [ ] Error cases are tested
- [ ] Tests are independent and reproducible
- [ ] Test names are descriptive

### Style and Readability

- [ ] Code follows project naming conventions
- [ ] Imports are organized by group
- [ ] Functions are less than 50 lines (preferred)
- [ ] Code is self-documenting (clear names, readable structure)
- [ ] Comments explain the **why**, not the **what**
- [ ] `cargo fmt` and `cargo clippy` pass

### Documentation

- [ ] Public functions have a `///` doc comment
- [ ] Modules have a `//!` header
- [ ] API changes are documented
- [ ] SQL migrations are documented
- [ ] CHANGELOG is updated if necessary

---

## 3. Comment Levels

| Prefix | Meaning | Required Action |
|--------|---------|----------------|
| `blocker:` | Critical issue (security, bug, data loss) | Must be fixed before merge |
| `issue:` | Significant problem (logic, performance) | Should be fixed |
| `nit:` | Minor detail (style, naming) | Optional, at author's discretion |
| `question:` | Request for clarification | Response expected |
| `suggestion:` | Improvement proposal | To discuss |
| `praise:` | Positive point to highlight | Encouragement |

### Examples

```
blocker: This SQL query concatenates user input without parameterization.
Use sqlx::query! with a $1 parameter.

issue: This function is 120 lines long. Consider extracting sub-functions
for readability.

nit: Prefer `DomainName` (newtype) over `String` here for type safety.

question: Why use `fetch_one` instead of `fetch_optional` here?
The record might not exist.

suggestion: We could use `impl From<DomainRow> for Domain` instead
of a `to_domain()` method to follow Rust conventions.

praise: The transparent password rehash pattern is well designed,
edge case handling is complete.
```

---

## 4. Process

```
1. Author opens PR with complete description
2. CI runs automatic checks
3. A reviewer is assigned (or volunteers)
4. Reviewer examines code and leaves comments
5. Author responds and makes changes
6. Reviewer re-examines changes
7. Approval → Merge
```

### Timelines

- First review: target 24 business hours
- Comment response: target 48 business hours
- If reviewer is unavailable, another can take over

---

## 5. What We Do NOT Do in Review

- Criticize the person (we criticize code, not author)
- Enforce undocumented style preferences in guidelines
- Block PR for cosmetic details
- Rewrite PR in comments (suggest, don't impose)
- Approve without reading code ("LGTM" without examination)

---
