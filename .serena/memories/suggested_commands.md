# Development Commands

## Running the Application
```bash
npm run tauri dev          # Run development server with hot reload
npm run tauri build        # Build for production
npm run build              # Build frontend only
npm run preview            # Preview production build
```

## Code Quality
```bash
npm run check              # Run type checking with svelte-check
npm run check:watch        # Watch mode for type checking
```

## Database
- Location: `%APPDATA%\my-todos\todos.db` (Windows)
- Use SQLite browser to inspect tables and schema
- Check migrations in `src-tauri/src/db/schema.rs`

## Git Commands
```bash
git status                 # Check current changes
git add .                  # Stage changes
git commit -m "message"    # Commit changes
git push                   # Push to remote
```

## Project Structure Navigation
- Frontend components: `src/lib/components/`
- Frontend stores: `src/lib/stores/`
- Tauri commands: `src-tauri/src/commands/`
- Database layer: `src-tauri/src/db/`
- Service wrappers: `src/lib/services/db.ts`
