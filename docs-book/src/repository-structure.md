# Repository Structure

```text
.
├── Cargo.lock                           # Cargo dependency lockfile (generated; checked in for reproducible builds)
├── Cargo.toml                           # Crate metadata and dependencies
├── LICENSE                              # Project license
├── README.md                            # Top-level overview (link to docs)
└── src
    ├── cli                              # CLI subcommands and orchestration
    │   ├── build                        # `autograder-setup build` — render workflow YAML from autograder.json
    │   │   ├── build_functions.rs       # Preamble, YAML helpers, commit-count script writer, small utilities
    │   │   ├── mod.rs                   # Subcommand entry + YAMLAutograder builder (ties everything together)
    │   │   ├── steps.rs                 # Hand-assembled YAML step emitters (CommandStep / ReporterStep)
    │   │   └── tests.rs                 # Unit tests for YAML rendering and build behavior
    │   ├── init                         # `autograder-setup init` — scan tests and write `.autograder/autograder.json`
    │   │   ├── functions.rs             # High-level constructors for AutoTests (clippy/commit count/test count)
    │   │   ├── mod.rs                   # Subcommand entry and pipeline glue
    │   │   ├── scan.rs                  # Rust source scanner (finds #[test]/#[..::test], docs, manifests)
    │   │   └── tests.rs                 # Parser/scan tests and manifest-path logic tests
    │   ├── mod.rs                       # Top-level CLI wiring (arg parsing, subcommand dispatch)
    │   ├── reset                        # `autograder-setup reset` — remove generated files
    │   │   ├── mod.rs                   # Subcommand entry
    │   │   └── tests.rs                 # Safety checks for deleting generated artifacts
    │   ├── table                        # `autograder-setup table` — generate student-facing Markdown table
    │   │   └── mod.rs                   # Subcommand entry and table rendering
    │   └── tests.rs                     # Cross-subcommand/integration-style tests for the CLI layer
    ├── main.rs                          # Binary entrypoint; delegates to `cli`
    ├── types                            # Core data model for the autograder
    │   ├── command_makers.rs            # Per-variant command builders (cargo test/clippy/test-count/commit-count)
    │   └── mod.rs                       # `AutoTest { meta, kind }`, `TestMeta`, `TestKind` + Markdown row impl
    └── utils
        ├── mod.rs                       # Shared helpers: path walking, slug/id, yaml_quote, replace_double_hashtag, etc.
        └── tests.rs                     # Unit tests for utilities
```
