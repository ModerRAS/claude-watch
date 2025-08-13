# claude-watch - CRUSH.md

## Build/Test Commands
```bash
# Build
cargo build --release
cargo build

# Run tests
cargo test
cargo test activity_detection
cargo test integration

# Run single test
cargo test test_activity_detection_standard_format

# Lint/Format
cargo fmt --check
cargo clippy
```

## Code Style Guidelines

### Formatting
- Use `cargo fmt` for formatting
- 100 character line limit
- Prefer `snake_case` for functions and variables
- Use `PascalCase` for types and structs

### Imports
- Group imports: std, external, internal modules
- Use `use crate::mod::item` for internal imports
- Avoid wildcard imports except for tests

### Error Handling
- Use `Result<T, E>` for recoverable errors
- Use `?` operator for error propagation
- Provide context with `context()` or custom error messages

### Types
- Prefer explicit types over inference
- Use `String` for owned strings, `&str` for borrowed
- Prefer `usize` for indices and counts

### Async/Await
- Use `.await` instead of `block_on()` in async contexts
- Avoid creating new runtimes within existing ones
- Handle async errors properly with `?`

### Testing
- Name tests descriptively: `test_functionality_scenario`
- Use `assert_eq!` with error messages for debugging
- Group related tests in separate modules