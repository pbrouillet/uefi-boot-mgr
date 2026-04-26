use clap::Parser;
use tracing_subscriber::EnvFilter;

use uefibootmgrrs::cli::{self, Cli, Commands};
use uefibootmgrrs::core;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Init tracing
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("warn")
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Check privileges before doing anything that touches EFI variables
    core::privilege::check_privileges().map_err(|e| {
        eprintln!("Error: {e}");
        eprintln!("Hint: {}", core::privilege::privilege_hint());
        e
    })?;

    // Create the boot manager backed by the system EFI variable store
    let var_manager = efivar::system();
    let mut mgr = core::BootManager::new(var_manager);

    match cli.command {
        Commands::List => cli::list::run(&mgr, cli.json)?,
        Commands::Info { ref id } => cli::info::run(&mgr, id, cli.json)?,
        Commands::Create {
            ref description,
            ref loader,
            ref partition,
            ref id,
        } => {
            cli::create::run(
                &mut mgr,
                description,
                loader,
                partition.as_deref(),
                id.as_deref(),
            )?;
        }
        Commands::Delete { ref id, force } => {
            cli::delete::run(&mut mgr, id, force)?;
        }
        Commands::Edit {
            ref id,
            ref description,
            ref loader,
        } => {
            cli::edit::run(&mut mgr, id, description.as_deref(), loader.as_deref())?;
        }
        Commands::Order { ref ids } => {
            cli::order::run(&mut mgr, ids)?;
        }
        Commands::Next { ref id } => {
            cli::next::run(&mut mgr, id)?;
        }
        Commands::Enable { ref id } => {
            cli::toggle::run_enable(&mut mgr, id)?;
        }
        Commands::Disable { ref id } => {
            cli::toggle::run_disable(&mut mgr, id)?;
        }
        Commands::Backup { ref file } => {
            cli::backup::run(&mgr, file)?;
        }
        Commands::Restore { ref file, force } => {
            cli::restore::run(&mut mgr, file, force)?;
        }
        Commands::Tui => {
            uefibootmgrrs::tui::run_tui(mgr)?;
        }
        Commands::Esp { ref action } => match action {
            cli::EspAction::List => cli::esp::run_list(cli.json)?,
            cli::EspAction::Set {
                disk,
                partition,
            } => cli::esp::run_set(disk, *partition)?,
            cli::EspAction::Clear {
                disk,
                partition,
            } => cli::esp::run_clear(disk, *partition)?,
            cli::EspAction::Bootloader => cli::esp::run_bootloader(cli.json)?,
        },
    }

    Ok(())
}
