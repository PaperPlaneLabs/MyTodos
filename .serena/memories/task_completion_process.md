# Task Completion Process

## When Adding a Feature
1. **Update Dependencies**: Add packages to `Cargo.toml` or `package.json` as needed
2. **Backend Implementation**: Add Rust command in `src-tauri/src/commands/`
3. **Register Commands**: Add to `invoke_handler!` macro in `src-tauri/src/lib.rs`
4. **Service Layer**: Add TypeScript wrapper in `src/lib/services/db.ts`
5. **Frontend Store** (if needed): Create in `src/lib/stores/`
6. **UI Components** (if needed): Create in `src/lib/components/`
7. **Testing**: Run `npm run tauri dev` and test functionality

## Code Review Checklist
- Type safety: All interfaces typed in TypeScript
- Error handling: Use `Result` in Rust, proper error serialization
- Consistency: Follow existing patterns in codebase
- No unused imports or variables
- Svelte components use runes properly

## Formatting
- Rust: Standard rustfmt
- TypeScript/Svelte: Standard prettier
- No special linting configured yet
