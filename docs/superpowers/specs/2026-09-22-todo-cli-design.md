# todo-cli Design

**Date:** 2026-09-22
**Status:** Approved (pending implementation plan)
**Audience:** Beginner Rust developer building a full-featured CLI todo app.

## Goal

A cross-platform CLI todo manager with priorities, due dates, tags, search, edit-in-place, and soft-delete with auto-purge buffer. Stored in SQLite. Plain-text output for v1, colored output deferred to v2.

## Scope (v1)

### Features

- Add todo with optional priority, due date, tags.
- List todos with filters: `--all`, `--tag`, `--priority`, `--search`, `--trash`.
- Mark done / undone by ID.
- Edit any field of an existing todo.
- Soft-delete with auto-purge buffer of 15 most recent deletions.
- Restore from trash.
- `--help` and `--version` via clap.

### Out of scope (v1)

- Colored output (deferred to v2).
- Archive (replaced by soft-delete buffer).
- Recurring todos, reminders, sync, multi-user.
- Configuration file, plugin system.

## Tech Stack

| Concern | Choice |
|---------|--------|
| Language | Rust (edition 2024) |
| CLI parsing | `clap` with `derive` feature |
| Storage | `rusqlite` with `bundled` feature (SQLite, no system dep) |
| Data dir | `dirs` crate (cross-platform user data dir) |
| Dates | `chrono` with `serde` feature |
| Errors | `thiserror` (definitions) + `anyhow` (top-level) |
| Testing | Built-in `#[test]` + `assert_cmd` + `predicates` for CLI integration tests |

### Code structure (after refactor)

```
src/
├── main.rs          # entry: init DB path, parse CLI, dispatch
├── cli.rs           # clap structs (Cli, Commands enum)
├── db.rs            # connection open, schema migration, path resolution
├── error.rs         # AppError + Result alias
├── models.rs        # Todo struct, Priority enum
└── commands/
    ├── mod.rs
    ├── add.rs
    ├── list.rs
    ├── done.rs
    ├── undone.rs
    ├── edit.rs
    ├── delete.rs
    └── restore.rs

tests/
├── common/
│   └── mod.rs       # shared test helpers (setup_db)
└── cli.rs           # end-to-end CLI tests via assert_cmd
```

**Development flow:** start in `main.rs` until `add` + `list` work, then split into modules (hybrid approach). User confirmed this structure.

## Data Model

### Rust

```rust
pub enum Priority {
    Low,
    Medium,
    High,
}

pub struct Todo {
    pub id: i64,
    pub title: String,
    pub done: bool,
    pub priority: Priority,
    pub due: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub created_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
}
```

### SQLite schema

```sql
CREATE TABLE todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    done INTEGER NOT NULL DEFAULT 0,
    priority TEXT NOT NULL DEFAULT 'medium',
    due DATE,
    tags TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    completed_at TEXT,
    deleted_at TEXT
);

CREATE INDEX idx_done ON todos (done);
CREATE INDEX idx_priority ON todos (priority);
CREATE INDEX idx_deleted_at ON todos (deleted_at);
```

### Field encoding choices
- `done` stored as `INTEGER` (0/1) since SQLite has no bool.
- `tags` stored as comma-separated string. Sufficient at expected scale; can be normalized to a separate table if needed later.
- All timestamps stored as ISO 8601 strings.
- `priority` stored as TEXT (`'low' | 'medium' | 'high'`).

### ID semantics
- IDs are 1-based SQLite row IDs, stable across soft-delete.
- Restore uses the same ID — soft-deleted rows keep their IDs.

## Command Surface

```
todo add "title"        [-p low|medium|high] [-d YYYY-MM-DD] [-t tag1,tag2]
todo list [--all] [--tag X] [--priority X] [--search X] [--trash]
todo done <id>
todo undone <id>
todo edit <id>          [--title X] [--priority X] [--due X] [--tag X]
todo delete <id>
todo restore <id>
todo --help
todo --version
```

### Delete semantics (soft + auto-purge)

1. `UPDATE todos SET deleted_at = ? WHERE id = ? AND deleted_at IS NULL`.
2. If 0 rows affected → return `AppError::NotDeleted(id)`.
3. Count rows with non-null `deleted_at`.
4. If count > 15, hard-delete the oldest excess:
   ```sql
   DELETE FROM todos
   WHERE id IN (
       SELECT id FROM todos
       WHERE deleted_at IS NOT NULL
       ORDER BY deleted_at ASC
       LIMIT ?
   )
   ```
   where `?` = `count - 15`.

## Storage Location

SQLite file at user data dir, app subfolder `todo-cli/todos.db`:
- Windows: `%APPDATA%/todo-cli/todos.db`
- macOS: `~/Library/Application Support/todo-cli/todos.db`
- Linux: `~/.local/share/todo-cli/todos.db`

Resolved via `dirs` crate. Directory auto-created on first run via `std::fs::create_dir_all`.

## Command Flow

```
std::env::args()
   ↓
Cli::parse()              ← clap derives
   ↓
match Cli.command {
    Add(args)    → commands::add::run(&conn, args),
    List(args)   → commands::list::run(&conn, args),
    Done(id)     → commands::done::run(&conn, id),
    Undone(id)   → commands::undone::run(&conn, id),
    Edit(args)   → commands::edit::run(&conn, args),
    Delete(id)   → commands::delete::run(&conn, id),
    Restore(id)  → commands::restore::run(&conn, id),
}
   ↓
print handler's String to stdout; errors to stderr; exit 1 on AppError
```

Each handler:
- Pure function: `(&Connection, args) -> Result<String, AppError>`.
- Returns displayable string. `main` prints. Handlers have no stdout side effects — testable.

### Example: add flow

1. Validate `priority` is a known enum variant.
2. Parse `due` as `NaiveDate` if present.
3. Split `tags` on `,`, trim whitespace.
4. `INSERT INTO todos (...) VALUES (...)`.
5. Return `"Added #3: buy milk"`.

### Example: delete flow

1. Run soft-delete UPDATE (see Delete semantics).
2. If 0 rows → `AppError::NotDeleted(id)`.
3. Count + purge excess (see Delete semantics).
4. Return `"Deleted #3 (buffered, 15 max)"`.

## Error Handling

### AppError (thiserror)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("todo #{0} not found")]
    NotFound(i64),

    #[error("todo #{0} is already done")]
    AlreadyDone(i64),

    #[error("todo #{0} is not deleted")]
    NotDeleted(i64),

    #[error("invalid priority: {0} (expected low|medium|high)")]
    InvalidPriority(String),

    #[error("invalid date: {0} (expected YYYY-MM-DD)")]
    InvalidDate(String),

    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
```

### Top-level (anyhow)

```rust
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let conn = db::open()?;
    db::migrate(&conn)?;
    let output = dispatch(&cli, &conn)?;
    println!("{}", output);
    Ok(())
}
```

`thiserror` defines typed errors handlers can match on. `anyhow` catches anything that escapes, prints it, exits 1.

## Testing

**Target:** 80%+ coverage, TDD per project rules.

### Unit tests (in-module)

- Every handler tested with `:memory:` SQLite.
- DB is in-RAM, fresh per test, fast.
- Each command: happy path + error paths (`NotFound`, `AlreadyDone`, etc.).

### Integration tests (`tests/cli.rs`)

- Uses `assert_cmd` + `predicates` crates.
- Spawns compiled binary, asserts stdout/stderr/exit code.
- Covers clap parsing + argument wiring + end-to-end delete-then-purge.

### Test fixtures

- `tests/common/mod.rs` provides `setup_db() -> Connection` helper.
- Each test starts with empty DB.

### Coverage

- `cargo install cargo-llvm-cov` (or `cargo-tarpaulin`).
- Run before declaring v1 complete.

### Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn fresh_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrate(&conn).unwrap();
        conn
    }

    #[test]
    fn add_returns_new_id() {
        let conn = fresh_db();
        let id = add(&conn, "buy milk", Priority::High, None, &[]).unwrap();
        assert_eq!(id, 1);
    }

    #[test]
    fn delete_unknown_id_returns_not_deleted() {
        let conn = fresh_db();
        let err = delete(&conn, 999).unwrap_err();
        assert!(matches!(err, AppError::NotDeleted(999)));
    }
}
```

## Open Items / Risks

- **Cargo.toml `name` field** currently says `downloads-sorter` — needs to be renamed to `todo-cli` in the package manifest. Tracked as first implementation step.
- **`bundled` rusqlite feature** increases binary size (~1MB) but eliminates system dependency. Acceptable trade.
- **No CI yet** — manual `cargo test` + `cargo clippy` per local rules.

## Next Step

After this spec is reviewed and approved, the `writing-plans` skill will produce a phased implementation plan (red-green-refactor per feature).