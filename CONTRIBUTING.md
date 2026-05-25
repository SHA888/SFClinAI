# Contributing to SFClinAI

Thank you for your interest in contributing to this project!

## Development Setup

### Prerequisites

- **Rust**: Version 1.86.0 or later (see `clinlat/rust-toolchain.toml`)
- **Git**: For version control

### Tooling

We use `cargo-skill` for streamlined Rust development workflows:

```bash
cargo install cargo-skill
```

This tool provides enhanced development ergonomics for the clinlat crate and related tooling.

See the [cargo-skill documentation](https://github.com/yourlibrary/cargo-skill) for detailed usage.

## Project Structure

```
SFClinAI/
├── NOTE.md             # Position note (source of truth)
├── SPEC.md             # Formal definitions and proof obligations
├── ARCHITECTURE.md     # Visual diagrams (Mermaid)
├── clinlat/            # Rust substrate kernel v0.1.0
│   ├── Cargo.toml
│   ├── src/
│   └── README.md       # Kernel-specific documentation
└── LICENSE, LICENSE-MIT, LICENSE-APACHE
```

## Contribution Order

When proposing changes, please prioritize in this order:

1. **Demonstrate the synthesis is already published** and was missed in §6 prior-art mapping.
2. **Show one of the eighteen principles** (NOTE.md §4A–§4D) is wrong or unnecessary.
3. **Demonstrate the substrate-first framing fails** on a clinical decision not covered by the six worked examples (NOTE.md §7E.1–§7E.6).

Short critiques: Open an issue.
Long-form critiques: Submit a PR to the `critique/` branch.

## Three-Document Stack

- **NOTE.md** (v0.12.0-draft): The position note and source of truth.
- **SPEC.md** (v0.3.0-draft): Formalization layer with formal definitions and proof obligations.
- **ARCHITECTURE.md** (v0.1.0-draft): Visual diagrams supporting comprehension.

When revising, update all three documents to maintain consistency. See `CLAUDE.md` for detailed editing guidelines.

## License

- **Prose and diagrams**: CC BY 4.0 (see `LICENSE`)
- **Code**: MIT OR Apache-2.0 (see `LICENSE-MIT` and `LICENSE-APACHE`)

## Contact

For questions, please file an issue on the GitHub repository.
