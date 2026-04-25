# uefibootmgrrs

A Rust CLI/TUI application to manage UEFI boot entries and the EFI System Partition (ESP).

## Features

- **List, inspect, create, edit, delete** UEFI boot entries (Boot####)
- **Set boot order** and **BootNext** (one-time override)
- **Enable/disable** boot entries (LOAD_OPTION_ACTIVE flag)
- **Backup/restore** all entries to JSON (lossless via base64 raw bytes)
- **Interactive TUI** mode with full CRUD and keyboard navigation
- **ESP management** — list GPT partitions, set/clear ESP type GUID
- **Cross-platform** — Windows (Win32 firmware APIs) and Linux (efivarfs)

## Requirements

- **Rust 1.75+** (edition 2021)
- **Administrator/root** privileges (required for EFI variable access)
- Windows: Visual Studio Build Tools (MSVC linker)
- Linux: `efivarfs` mounted at `/sys/firmware/efi/efivars/`

## Build

```bash
cargo build --release
```

## Usage

All commands require elevated privileges.

### List boot entries

```bash
uefibootmgrrs list
uefibootmgrrs list --json
```

### Inspect a boot entry

```bash
uefibootmgrrs info 0001
uefibootmgrrs info 0001 --json
```

### Create a boot entry

```bash
uefibootmgrrs create --description "My OS" --loader "\EFI\myos\bootx64.efi"
uefibootmgrrs create --description "My OS" --loader "\EFI\myos\bootx64.efi" --id 0005
```

### Edit a boot entry

```bash
uefibootmgrrs edit 0001 --description "New Name"
uefibootmgrrs edit 0001 --loader "\EFI\newpath\boot.efi"
```

### Delete a boot entry

```bash
uefibootmgrrs delete 0001
uefibootmgrrs delete 0001 --force   # skip confirmation
```

### Enable / disable

```bash
uefibootmgrrs enable 0001
uefibootmgrrs disable 0001
```

### Set boot order

```bash
uefibootmgrrs order 0001,0000,0002
```

### Set BootNext (one-time boot)

```bash
uefibootmgrrs next 0001
```

### Backup & restore

```bash
uefibootmgrrs backup boot-backup.json
uefibootmgrrs restore boot-backup.json
```

The backup format stores raw EFI variable bytes (base64-encoded) for lossless round-trips. Decoded fields (description, file path) are included for readability but ignored on restore.

### Interactive TUI

```bash
uefibootmgrrs tui
```

**Keybindings:**

| Key | Action |
|-----|--------|
| `↑`/`↓` | Navigate entries |
| `Enter` | View entry details |
| `n` | Create new entry (blank) |
| `w` | Boot entry wizard (Windows/GRUB presets) |
| `e` | Edit selected entry |
| `d` | Delete selected entry |
| `Space` | Toggle active/inactive |
| `o` | Toggle reorder mode (then `↑`/`↓` to move) |
| `b` | Backup to file |
| `r` | Restore from file |
| `?` | Help overlay |
| `q` | Quit |

### ESP partition management

```bash
# List all GPT partitions
uefibootmgrrs esp list
uefibootmgrrs esp list --json

# Set a partition as ESP
uefibootmgrrs esp set "0" 1          # Windows: disk 0, partition 1
uefibootmgrrs esp set "/dev/sda" 1   # Linux

# Clear ESP flag (revert to Basic Data)
uefibootmgrrs esp clear "0" 1
```

**ESP type GUIDs:**
- ESP: `C12A7328-F81F-11D2-BA4B-00A0C93EC93B`
- Basic Data: `EBD0A0A2-B9E5-4433-87C0-68B6B72699C7`

## Architecture

```
src/
├── main.rs            Entry point, clap dispatch
├── lib.rs             Module root
├── error.rs           Error types (thiserror)
├── cli/               CLI command handlers
│   ├── mod.rs         Clap derive + Commands enum
│   ├── list.rs        list subcommand
│   ├── info.rs        info <id>
│   ├── create.rs      create
│   ├── delete.rs      delete <id>
│   ├── edit.rs        edit <id>
│   ├── order.rs       order <ids>
│   ├── next.rs        next <id>
│   ├── toggle.rs      enable/disable <id>
│   ├── backup.rs      backup <file>
│   ├── restore.rs     restore <file>
│   └── esp.rs         ESP partition commands
├── core/              Business logic
│   ├── mod.rs         Re-exports
│   ├── manager.rs     BootManager (wraps dyn VarManager)
│   ├── entry.rs       BootEntryInfo (parsed, display-friendly)
│   ├── backup.rs      BackupData serde + export/import
│   ├── privilege.rs   Runtime privilege checks
│   └── esp.rs         ESP partition management
└── tui/               Interactive terminal UI
    ├── mod.rs          Terminal setup, run_tui()
    ├── app.rs          App state + CRUD actions
    ├── event.rs        Event loop + key handling
    ├── widgets.rs      Status bar, confirm dialog
    └── views/          View renderers
        ├── entry_list.rs
        ├── entry_detail.rs
        ├── entry_form.rs
        ├── backup.rs
        └── help.rs
```

## Testing

```bash
cargo test --workspace
```

Tests use `efivar::file_store()` (TOML-backed) to avoid requiring real EFI variables. 56 tests cover:
- Core library (14): CRUD, boot order, backup/restore
- CLI integration (16): All subcommands against file store
- TUI state (22): Navigation, forms, delete confirm, reorder, backup, wizard templates
- ESP parsing (4): Windows PowerShell JSON output parsing

## License

MIT
