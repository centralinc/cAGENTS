# RFC 0000: [Feature Name]

## Metadata

- **Status:** Draft | Proposed | Accepted | Implementing | Completed | Deferred | Rejected
- **RFC PR:** #[PR number once submitted]
- **Target Version:** 0.X.0
- **Author(s):** [Your name or GitHub username]
- **Created:** YYYY-MM-DD
- **Last Updated:** YYYY-MM-DD

## Summary

One paragraph explanation of the feature.

## Motivation

Why are we doing this? What use cases does it support? What is the expected outcome?

Include user stories or examples:

> As a [user type], I want [goal] so that [benefit].

## Guide-level Explanation

Explain the feature as if it was already implemented and you were teaching it to another cAGENTS user. That means:

- Introduce new named concepts
- Explain with examples
- Show how it interacts with existing features
- Discuss error messages and edge cases

**Example usage:**

```bash
# Show what the user would type or configure
cagents new-command --flag value
```

```toml
# Show config file changes
[new_section]
option = "value"
```

```markdown
# Show template changes
---
new_field: value
---
```

## Reference-level Explanation

This is the technical portion of the RFC. Explain the design in sufficient detail that:

- Its interaction with other features is clear
- It's reasonably clear how the feature would be implemented
- Corner cases are dissected by example

### Architecture

High-level overview of how this fits into cAGENTS:

```
┌─────────────┐      ┌──────────────┐      ┌────────────┐
│  Existing   │─────▶│  New         │─────▶│  Output    │
│  Component  │      │  Component   │      │            │
└─────────────┘      └──────────────┘      └────────────┘
```

### Data Structures

Show key types/structs:

```rust
pub struct NewFeature {
    pub field: String,
}
```

### Behavior

Describe algorithms, logic, or processing flow:

1. Step one
2. Step two
3. Result

### Error Handling

How will this fail gracefully? What error messages?

### Performance Impact

Will this slow down build? Increase memory? Require optimization?

## Drawbacks

Why should we *not* do this?

- Implementation cost
- Maintenance burden
- Learning curve for users
- Interaction with other features

## Rationale and Alternatives

- Why is this design the best in the space of possible designs?
- What other designs have been considered and what is the rationale for not choosing them?
- What is the impact of not doing this?

### Alternative 1: [Name]

**Approach:** [Brief description]

**Pros:**
- Pro 1

**Cons:**
- Con 1

**Why not chosen:** [Reasoning]

### Alternative 2: [Name]

...

## Prior Art

Discuss prior art, both the good and the bad, in relation to this proposal. Examples:

- Does Rust, Python, or another language/tool have a similar feature?
- Are there published papers or articles?
- What can we learn from similar implementations?

## Unresolved Questions

What parts of the design do you expect to resolve through the RFC process before this gets merged?

- **Question 1:** What should X do in case Y?
  - Option A: ...
  - Option B: ...
  - Recommendation: ...

- **Question 2:** How do we handle Z?

## Future Possibilities

Think about what the natural extension and evolution of your proposal would be and how it would affect the project as a whole. This is a good place to "dump ideas" that are related but outside the scope of this RFC.

- Future enhancement 1
- Future enhancement 2
- Related RFCs that might build on this

---

## Implementation Plan (Optional for Initial RFC)

*Note: Detailed implementation plans can be added after RFC acceptance. For the initial proposal, high-level phases are sufficient.*

### Phase 1: Core Implementation
- Task 1
- Task 2

### Phase 2: Integration
- Task 1
- Task 2

### Phase 3: Documentation
- Update docs
- Add examples

---

## Notes for AI Agents

*If this RFC is intended to be implemented by AI coding agents, add specific guidance here:*

- **Key files to modify:** List files with line number ranges if known
- **Test requirements:** Specific tests that must pass
- **Examples to reference:** Point to similar code in the codebase

---

## Amendments

*Track significant changes to the RFC after initial submission:*

- YYYY-MM-DD: [Description of change]
