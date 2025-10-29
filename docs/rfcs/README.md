# cAGENTS RFCs

This directory contains RFCs (Request for Comments) for major features and architectural changes to cAGENTS. The RFC process is inspired by [Rust](https://github.com/rust-lang/rfcs), [React](https://github.com/reactjs/rfcs), and other successful open-source projects.

## What is an RFC?

An RFC is a design document that:
- Proposes a new feature or change
- Describes the motivation and design in detail
- Allows for community discussion before implementation
- Serves as documentation once accepted

## When to write an RFC?

Write an RFC for:
- **Major features** (e.g., Claude Skills support, new output formats)
- **Breaking changes** (e.g., changing default behavior)
- **Architectural changes** (e.g., adding a plugin system)

**Don't need an RFC for:**
- Bug fixes
- Minor improvements
- Documentation updates
- Refactoring (unless changing public APIs)

## The RFC Process

### 1. **Draft the RFC**

Copy the template and fill it in:

```bash
cp docs/rfcs/0000-template.md docs/rfcs/0000-my-feature.md
```

**Focus on:**
- **Problem**: What are we solving?
- **Motivation**: Why is this important?
- **Design**: How will it work (high-level)?
- **Alternatives**: What else did we consider?
- **Unresolved questions**: What needs discussion?

**Note:** The RFCs in this repo are detailed for AI coding agents. For new RFCs, start lighter—you can add implementation details after acceptance.

### 2. **Submit as Pull Request**

```bash
git checkout -b rfc-my-feature
git add docs/rfcs/0000-my-feature.md
git commit -m "RFC: My Feature"
git push origin rfc-my-feature
# Open PR on GitHub
```

**Title:** `RFC: My Feature Name`

**PR Description:** Summarize the RFC and link to the file.

### 3. **Discussion**

- Community discusses in PR comments
- Author revises RFC based on feedback
- Aim for consensus (not unanimity)
- Decision makers: Core maintainers

### 4. **Acceptance**

When consensus is reached:
- Assign final RFC number (e.g., 0005)
- Rename file: `0000-my-feature.md` → `0005-my-feature.md`
- Update metadata to `Status: Accepted`
- **Merge the RFC PR** (merging means design is approved, not implementation)

### 5. **Implementation**

After RFC is merged:
- Implementation happens in **separate PRs**
- Reference the RFC: `Implements RFC 0005` or `Part of RFC 0005`
- If implementation deviates, update RFC or document in PR

### 6. **Completion**

When fully implemented:
- Update RFC metadata: `Status: Completed`
- Link to implementation PRs in RFC
- Add to CHANGELOG

## Active RFCs

| # | Title | Status | Version | PR |
|---|-------|--------|---------|-----|
| 0001 | Claude Skills Support | Draft | 0.2.0 | TBD |
| 0002 | Progressive Disclosure | Draft | 0.2.0 | TBD |
| 0003 | Multi-Format Consistency | Draft | 0.2.0 | TBD |
| 0004 | CLI Enhancements | Draft | 0.2.0 | TBD |

## RFC Statuses

- **Draft**: Initial proposal, not yet submitted as PR or under discussion
- **Proposed**: Submitted as PR, open for comments
- **Accepted**: Merged (design approved), ready for implementation
- **Implementing**: Work in progress
- **Completed**: Fully implemented and shipped
- **Deferred**: Good idea, but not now (moved to backlog)
- **Rejected**: Not moving forward

## Tips for Writing RFCs

### Keep it focused
- One RFC per feature
- If the scope grows, split into multiple RFCs

### Show your work
- Explain alternatives considered
- Link to research, prior art, discussions
- Include code examples (even pseudocode)

### Be open to feedback
- RFCs are collaborative
- Goal is the best design, not "your" design

### Update as you learn
- RFCs are living documents until accepted
- After acceptance, major changes may need a new RFC

## For AI Coding Agents

The RFCs in this repository include **detailed implementation guides** to help AI agents pick up work:

- **Phased tasks** with acceptance criteria
- **Test specifications** (what to test before considering it done)
- **Code examples** in Rust
- **File locations** for changes

This level of detail is unusual for RFCs but intentional for AI-first development.

**When implementing from an RFC:**
1. Read the entire RFC first
2. Follow the implementation plan phases
3. Write tests before code (TDD)
4. Mark tasks complete as you go
5. Update the RFC if you deviate from the plan

## Questions?

- For questions about the RFC process: Open an issue with `question` label
- For feedback on a specific RFC: Comment on its PR
- For proposing changes to an accepted RFC: Open a new issue first

## References

- [Rust RFC Process](https://github.com/rust-lang/rfcs)
- [React RFC Process](https://github.com/reactjs/rfcs)
- [Why RFCs?](https://buriti.ca/6-lessons-i-learned-while-implementing-technical-rfcs-as-a-decision-making-tool-34687dbf46cb)
