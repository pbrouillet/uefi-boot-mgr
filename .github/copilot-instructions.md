# Copilot Instructions — uefibootmgrrs

## Build & Test

The MSVC linker (`link.exe`) is **not** in the default PATH. Source the VS environment before every `cargo` invocation:

```powershell
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat" x64 && cargo build 2>&1'
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat" x64 && cargo test --workspace 2>&1'
```

Single test:

```powershell
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat" x64 && cargo test -p uefibootmgrrs <test_name> 2>&1'
```

- Rust 1.75+, **edition 2024**, target `x86_64-pc-windows-msvc`
- VS 2022 BuildTools at `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools`

## Architecture

- **`core/manager.rs`** — `BootManager` wraps `Box<dyn efivar::VarManager>` for all CRUD. Only `main.rs` calls `efivar::system()` — everything else takes the trait object for testability.
- **`core/backup.rs`** — Backup format uses base64-encoded raw EFI variable bytes as canonical. Restore reads from raw bytes, never from decoded display fields. Validates all entries before writing any; writes `Boot####` first, `BootOrder` last.
- **`core/esp.rs`** — ESP partition management via shell-out: PowerShell `Get-Partition`/`Set-Partition` on Windows, `lsblk`/`sgdisk` on Linux. No raw Win32/ioctl.
- **`tui/`** — Interactive mode behind `tui` subcommand. `App` struct owns all state; `event.rs` runs the event loop; `views/` has per-view renderers.
- **`cli/`** — Clap derive with `Commands` enum. Each subcommand in its own file.

## Testing

All tests use `efivar::file_store(path)` which returns `Box<dyn VarManager>` backed by a TOML file — no real EFI variables needed. Use unique temp file paths per test to avoid collisions:

```rust
let store = efivar::file_store(std::env::temp_dir().join(format!("test-{}-{}.toml", name, std::process::id())));
let mgr = BootManager::new(store);
```

## efivar Crate (v2.0.0) API Reference

These are the **actual** names — do not guess alternatives:

| What you want | Correct API | Wrong guesses to avoid |
|---|---|---|
| Format boot var name | `u16.boot_var_name()` (trait `BootVarName`) | ~~`boot_var_format()`~~ |
| Add a boot entry | `manager.add_boot_entry(id, entry)` | ~~`create_boot_entry()`~~ |
| File path list fields | `FilePathList { file_path: FilePath, hard_drive: EFIHardDrive }` | ~~not a Vec~~ |
| Parse boot entry | `BootEntry::parse(Vec<u8>)` | |
| Serialize boot entry | `BootEntry::to_bytes()` | |
| System manager | `efivar::system()` → `Box<dyn VarManager>` | |
| Test file store | `efivar::file_store("path.toml")` → `Box<dyn VarManager>` | |

`VarManager` trait = `VarEnumerator + VarReader + VarWriter + BootVarReader + BootVarWriter`.

## Rust Edition 2024 Gotchas

1. **Unsafe extern blocks** — Must write `unsafe extern "system" { ... }`, not just `extern "system" { ... }`. The `#[link(name = "...")]` attribute goes before the extern block.

2. **No `ref` in implicitly-borrowing patterns** — When matching on a reference (`&Enum`), the pattern borrows implicitly. Writing `ref field` inside is an error. Just use the field name directly:

   ```rust
   // Given: match on &EspAction
   // WRONG:  EspAction::Set { ref disk, partition } =>
   // RIGHT:  EspAction::Set { disk, partition } =>
   ```

## ratatui 0.30 API Notes

- `Table::highlight_style()` is **deprecated** → use `row_highlight_style()`
- `Cell` lives in `ratatui::widgets::Cell` — **not** re-exported via `prelude::*`
- `TableState::default().with_selected(Some(idx))` for initial selection
- Use `frame.render_stateful_widget()` for tables with selection state
