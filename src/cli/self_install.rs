use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use tracing::warn;

use rokit::{
    storage::Home,
    system::{add_to_path, exists_in_path},
};

use crate::util::{finish_progress_bar, new_progress_bar};

/// Installs / re-installs Rokit, and updates all tool links.
#[derive(Debug, Parser)]
pub struct SelfInstallSubcommand {}

impl SelfInstallSubcommand {
    pub async fn run(self, home: &Home) -> Result<()> {
        let storage = home.tool_storage();

        let pb = new_progress_bar("Linking", 2, 1);
        let (had_rokit_installed, was_rokit_updated) = storage.recreate_all_links().await.context(
            "Failed to recreate tool links!\
            \nYour installation may be corrupted.",
        )?;

        pb.inc(1);
        pb.set_message("Pathifying");

        let mut path_errored = false;
        let path_was_changed = add_to_path(home)
            .await
            .inspect_err(|e| {
                path_errored = true;
                warn!(
                    "Failed to automatically add Rokit to your PATH!\
                    \nPlease add `~/.rokit/bin` to be able to run tools.
                    \nError: {e:?}",
                )
            })
            .unwrap_or(false);
        let path_contains_rokit = exists_in_path(home);

        // Prompt the user to restart their terminal if:
        // - PATH was changed
        // - PATH does not currently contain Rokit, and adding to PATH did not error
        let should_restart_terminal = path_was_changed || (!path_errored && !path_contains_rokit);
        let should_restart_lines = if should_restart_terminal {
            format!(
                "\n\nExecutables for Rokit and tools have been added to {}.\
                \nPlease restart your terminal for the changes to take effect.",
                style("$PATH").bold()
            )
        } else {
            String::new()
        };

        let main_message = if !had_rokit_installed {
            "Rokit has been installed successfully!"
        } else if was_rokit_updated {
            "Rokit was re-linked successfully!"
        } else {
            "Rokit links are already up-to-date."
        };

        let help_command = style("rokit --help").bold().green();
        let post_message = if should_restart_terminal {
            format!("\n\nThen, run `{help_command}` to get started using Rokit.")
        } else {
            format!("\n\nRun `{help_command}` to get started using Rokit.")
        };

        let msg = format!(
            "{main_message} {}{should_restart_lines}{post_message}",
            style(format!("(took {:.2?})", pb.elapsed())).dim(),
        );
        finish_progress_bar(pb, msg);

        Ok(())
    }
}
