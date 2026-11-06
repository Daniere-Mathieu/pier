use clap::{CommandFactory, Parser};
use std::process;

use pier::{
    cli::{Cli, CliSubcommand},
    open_editor,
    script::Script,
    Pier, PierResult,
};

fn main() {
    let opt = Cli::parse();

    match handle_subcommands(opt) {
        Ok(status) => {
            if let Some(status) = status {
                let code = status.code().unwrap_or(0);
                process::exit(code)
            } else {
                process::exit(0)
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };
}

/// Handles the commandline subcommands
fn handle_subcommands(cli: Cli) -> PierResult<Option<process::ExitStatus>> {
    if let Some(subcmd) = cli.cmd {
        match subcmd {
            CliSubcommand::Add {
                command,
                alias,
                description,
                tags,
                force,
            } => {
                let mut pier = Pier::from(cli.opts.path, cli.opts.verbose)?;
                pier.add_script(
                    Script {
                        alias,
                        description,
                        command: match command {
                            Some(cmd) => cmd,
                            None => open_editor(None)?,
                        },
                        tags,
                        reference: None,
                    },
                    force,
                )?;
                pier.write()?;
            }

            CliSubcommand::Edit { alias } => {
                let mut pier = Pier::from(cli.opts.path, cli.opts.verbose)?;

                match alias {
                    None => {
                        unimplemented!("interactive edit not implemented");
                    }
                    Some(ref a) => {
                        pier.edit_script(a)?;
                    }
                }

                pier.write()?;
            }
            CliSubcommand::Remove { alias } => {
                let mut pier = Pier::from(cli.opts.path, cli.opts.verbose)?;
                pier.remove_script(&alias)?;
                pier.write()?;
            }
            CliSubcommand::ConfigInit => {
                let mut pier = Pier::new();
                pier.config_init(cli.opts.path)?;
            }
            CliSubcommand::Completions { shell } => {
                clap_complete::generate(shell, &mut Cli::command(), "pier", &mut std::io::stdout());
            }
            CliSubcommand::Show { alias } => {
                let pier = Pier::from(cli.opts.path, cli.opts.verbose)?;
                let script = pier.fetch_script(&alias)?;
                println!("{}", script.command);
            }
            CliSubcommand::List {
                list_aliases,
                tags,
                cmd_full,
                cmd_width,
                namespace,
            } => {
                let pier = Pier::from(cli.opts.path, cli.opts.verbose)?;
                if list_aliases {
                    pier.list_aliases(tags, namespace)?
                } else {
                    pier.list_scripts(tags, cmd_full, cmd_width, namespace)?
                }
            }
            CliSubcommand::Run { alias, args } => {
                let pier = Pier::from(cli.opts.path, cli.opts.verbose)?;
                let exit_code = pier.run_script(&alias, args, cli.opts.dry_run)?;
                return Ok(Some(exit_code));
            }
            CliSubcommand::Copy {
                from_alias,
                to_alias,
            } => {
                let mut pier = Pier::from(cli.opts.path, cli.opts.verbose)?;
                pier.copy_script(&from_alias, &to_alias)?;
                pier.write()?;
            }
            CliSubcommand::Move {
                from_alias,
                to_alias,
                force,
            } => {
                let mut pier = Pier::from(cli.opts.path, cli.opts.verbose)?;
                pier.move_script(&from_alias, &to_alias, force)?;
                pier.write()?;
            }
            CliSubcommand::Export { alias, tag } => {
                let pier = Pier::from(cli.opts.path, cli.opts.verbose)?;
                let exported = pier.export_scripts(alias, tag)?;
                print!("{}", exported);
            }
            CliSubcommand::Import { path, force } => {
                let mut pier = Pier::from(cli.opts.path, cli.opts.verbose)?;
                pier.import_scripts(&path, force)?;
                pier.write()?;
            }
        };
    } else {
        let alias = &cli.alias.expect("Alias is required unless subcommand.");
        let pier = Pier::from(cli.opts.path, cli.opts.verbose)?;
        let exit_code = pier.run_script(alias, cli.args, cli.opts.dry_run)?;
        return Ok(Some(exit_code));
    }

    Ok(None)
}
