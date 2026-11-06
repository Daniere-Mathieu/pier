use prettytable::{row, Table};
use snafu::{ensure, OptionExt, ResultExt};
use std::fs;
use std::sync::LazyLock;
use std::{path::PathBuf, process::ExitStatus};
pub mod cli;
mod config;
pub mod error;
use config::Config;
mod defaults;
mod macros;
use defaults::*;
pub mod script;
use error::*;
use script::Script;

pub type PierResult<T, E = PierError> = ::std::result::Result<T, E>;

/// Main library interface
#[derive(Debug, Default)]
pub struct Pier {
    config: Config,
    path: PathBuf,
    verbose: bool,
}

use prettytable::format::FormatBuilder;
use prettytable::format::LinePosition;
use prettytable::format::LineSeparator;
use prettytable::format::TableFormat;

static COOL_SEP: LazyLock<LineSeparator> =
    LazyLock::new(|| LineSeparator::new('\u{2256}', '\u{2256}', '\u{2256}', '\u{2256}'));
pub static COOL_FORMAT: LazyLock<TableFormat> = LazyLock::new(|| {
    FormatBuilder::new()
        .column_separator('\u{22EE}')
        .borders('\u{22EE}')
        .separator(LinePosition::Title, *COOL_SEP)
        .separator(LinePosition::Bottom, *COOL_SEP)
        .separator(LinePosition::Top, *COOL_SEP)
        .padding(1, 1)
        .build()
});

impl Pier {
    /// Wrapper to write the configuration to path.
    pub fn write(&self) -> PierResult<()> {
        self.config.write(&self.path)?;

        Ok(())
    }

    pub fn config_init(&mut self, new_path: Option<PathBuf>) -> PierResult<()> {
        self.path = new_path
            .unwrap_or(fallback_path().unwrap_or(xdg_config_home!("pier/config.toml").unwrap()));

        ensure!(
            !self.path.exists(),
            ConfigInitFileAlreadyExists {
                path: &self.path.as_path()
            }
        );

        if let Some(parent_dir) = &self.path.parent() {
            if !parent_dir.exists() {
                fs::create_dir(parent_dir).context(CreateDirectory)?;
            }
        };

        let _ = self.add_script(
            Script {
                alias: String::from("hello-pier"),
                command: String::from("echo Hello, from Pier!, you should try edit this script with pier edit hello-pier !"),
                description: Some(String::from("This is an example command.")),
                reference: None,
                tags: None,
            },
            false,
        );

        self.write()?;

        Ok(())
    }

    pub fn new() -> Self {
        Pier::default()
    }

    /// Create new pier directly from path.
    pub fn from_file(path: PathBuf, verbose: bool) -> PierResult<Self> {
        let pier = Self {
            config: Config::from(&path)?,
            verbose,
            path,
        };
        Ok(pier)
    }
    /// Create new pier from what might be a path, otherwise use the first existing default path.
    pub fn from(input_path: Option<PathBuf>, verbose: bool) -> PierResult<Self> {
        let path = match input_path {
            Some(path) => path,
            None => fallback_path()?,
        };

        let pier = Pier::from_file(path, verbose)?;

        Ok(pier)
    }

    /// Fetches a script that matches the alias
    pub fn fetch_script(&self, alias: &str) -> PierResult<&Script> {
        ensure!(!self.config.scripts.is_empty(), NoScriptsExists);

        let script = self.config.scripts.get(alias).context(AliasNotFound {
            alias: &alias.to_string(),
        })?;

        Ok(script)
    }

    /// Edits a script that matches the alias
    pub fn edit_script(&mut self, alias: &str) -> PierResult<&Script> {
        ensure!(!self.config.scripts.is_empty(), NoScriptsExists);

        let script = self.config.scripts.get_mut(alias).context(AliasNotFound {
            alias: &alias.to_string(),
        })?;

        script.command = open_editor(Some(&script.command))?;

        println!("Edited {}", alias);

        Ok(script)
    }

    /// Removes a script that matches the alias
    pub fn remove_script(&mut self, aliases: &Vec<String>) -> PierResult<()> {
        ensure!(!self.config.scripts.is_empty(), NoScriptsExists);
        for alias in aliases {
            self.config.scripts.remove(alias).context(AliasNotFound {
                alias: &alias.to_string(),
            })?;

            println!("Removed {}", alias);
        }

        Ok(())
    }

    /// Adds a script that matches the alias
    pub fn add_script(&mut self, script: Script, force: bool) -> PierResult<()> {
        if !force {
            ensure!(
                !&self.config.scripts.contains_key(&script.alias),
                AliasAlreadyExists {
                    alias: script.alias
                }
            );
        }

        println!("Added {}", script.alias);

        self.config.scripts.insert(script.alias.to_string(), script);

        Ok(())
    }

    /// Prints only the aliases in current config file that matches tags and namespace.
    pub fn list_aliases(
        &self,
        tags: Option<Vec<String>>,
        namespace: Option<String>,
    ) -> PierResult<()> {
        ensure!(!self.config.scripts.is_empty(), NoScriptsExists);

        let ns_prefix = namespace.map(|n| format!("{}:", n));

        for (alias, script) in self.config.scripts.iter() {
            if let Some(ref prefix) = ns_prefix {
                if !alias.starts_with(prefix) {
                    continue;
                }
            }

            match (&tags, &script.tags) {
                (Some(list_tags), Some(script_tags)) => {
                    for tag in list_tags {
                        if script_tags.contains(tag) {
                            println!("{}", alias);
                            continue;
                        }
                    }
                }
                (None, _) => {
                    println!("{}", alias);
                    continue;
                }
                _ => (),
            };
        }

        Ok(())
    }

    /// Copy an alias a script that matches the alias
    pub fn copy_script(&mut self, from_alias: &str, new_alias: &str) -> PierResult<()> {
        ensure!(
            !&self.config.scripts.contains_key(new_alias),
            AliasAlreadyExists { alias: new_alias }
        );

        let script = self
            .config
            .scripts
            .get(from_alias)
            .context(AliasNotFound {
                alias: &from_alias.to_string(),
            })?
            .clone();

        println!("Copy from alias {} to new alias {}", from_alias, new_alias);

        self.config.scripts.insert(new_alias.to_string(), script);

        Ok(())
    }

    /// Move a script that matches the alias to another alias
    pub fn move_script(
        &mut self,
        from_alias: &str,
        new_alias: &str,
        force: bool,
    ) -> PierResult<()> {
        if !force {
            ensure!(
                !&self.config.scripts.contains_key(new_alias),
                AliasAlreadyExists { alias: new_alias }
            );
        }

        let script = self
            .config
            .scripts
            .remove(from_alias)
            .context(AliasNotFound {
                alias: &from_alias.to_string(),
            })?
            .clone();

        println!("Move from alias {} to new alias {}", from_alias, new_alias);

        self.config.scripts.insert(new_alias.to_string(), script);

        Ok(())
    }

    /// Prints a terminal table of the scripts in current config file that matches tags.
    pub fn list_scripts(
        &self,
        tags: Option<Vec<String>>,
        cmd_full: bool,
        cmd_width: Option<usize>,
        namespace: Option<String>,
    ) -> PierResult<()> {
        let width = match (cmd_width, self.config.default.command_width) {
            (Some(width), _) => width,
            (None, Some(width)) => width,
            (None, None) => FALLBACK_COMMAND_DISPLAY_WIDTH,
        };
        ensure!(!self.config.scripts.is_empty(), NoScriptsExists);

        let ns_prefix = namespace.map(|n| format!("{}:", n));

        let mut table = Table::new();

        table.set_format(*COOL_FORMAT);
        table.set_titles(row![
            Fc -> "Alias",
            Fc -> "Tags",
            Fc -> "Command",
            Fc -> "Description",
        ]);

        for (alias, script) in self.config.scripts.iter() {
            if let Some(ref prefix) = ns_prefix {
                if !alias.starts_with(prefix) {
                    continue;
                }
            }

            let shbang = script.command.starts_with("#!");
            let descp = match &script.description.as_ref() {
                Some(d) => d,
                None => "",
            };

            match (&tags, &script.tags) {
                (Some(list_tags), Some(script_tags)) => {
                    for tag in list_tags {
                        if script_tags.contains(tag) {
                            if shbang {
                                table.add_row(row![
                                    FY -> &alias,
                                    Fg -> script_tags.join(","),
                                    Fm -> "#! script",
                                    Fw -> descp,
                                ]);
                            } else {
                                table.add_row(row![
                                    FY -> &alias,
                                    Fg -> script_tags.join(","),
                                    Fb -> script.display_command(cmd_full, width),
                                    Fw -> descp,
                                ]);
                            }

                            continue;
                        }
                    }
                }
                (None, Some(script_tags)) => {
                    if shbang {
                        table.add_row(row![
                            FY -> &alias,
                            Fg -> script_tags.join(","),
                            Fm -> "#! script",
                            Fw -> descp,
                        ]);
                    } else {
                        table.add_row(row![
                            FY -> &alias,
                            Fg -> script_tags.join(","),
                            Fb -> script.display_command(cmd_full, width),
                            Fw -> descp,
                        ]);
                    }

                    continue;
                }
                (None, None) => {
                    if shbang {
                        table.add_row(row![
                            FY -> &alias,
                            Fg -> "",
                            Fm -> "#! script",
                            Fw -> descp,
                        ]);
                    } else {
                        table.add_row(row![
                            FY -> &alias,
                            Fg -> "",
                            Fb -> script.display_command(cmd_full, width),
                            Fw -> descp,
                        ]);
                    }

                    continue;
                }
                _ => (),
            };
        }

        let _ = table.print_tty(true);

        Ok(())
    }

    /// Runs a script and print stdout and stderr of the command.
    pub fn run_script(
        &self,
        alias: &str,
        args: Vec<String>,
        dry_run: bool,
    ) -> PierResult<ExitStatus> {
        let script = self.fetch_script(alias)?;
        let interpreter = match self.config.default.interpreter {
            Some(ref interpreter) => interpreter.clone(),
            None => fallback_shell(),
        };

        if dry_run {
            println!("Alias:       {}", alias);
            if script.has_shebang() {
                println!("Mode:        shebang");
            } else {
                println!("Mode:        interpreter");
                println!("Interpreter: {}", interpreter.join(" "));
            }
            println!("Command:     {}", script.command);
            if args.is_empty() {
                println!("Args:        (none)");
            } else {
                println!("Args:        {}", args.join(", "));
            }
            return Ok(std::process::ExitStatus::default());
        }

        if self.verbose {
            println!("Starting script \"{}\"", alias);
            println!("-------------------------");
        };

        let cmd = match script.has_shebang() {
            true => script.run_with_shebang(args)?,
            false => script.run_with_cli_interpreter(&interpreter, args)?,
        };

        let stdout = String::from_utf8_lossy(&cmd.stdout);
        let stderr = String::from_utf8_lossy(&cmd.stderr);

        if !stdout.is_empty() {
            println!("{}", stdout);
        };
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        };

        if self.verbose {
            println!("-------------------------");
            println!("Script complete");
        };

        Ok(cmd.status)
    }

    /// Export scripts as a TOML config string, optionally filtered by alias or tag.
    pub fn export_scripts(&self, alias: Option<String>, tag: Option<String>) -> PierResult<String> {
        ensure!(!self.config.scripts.is_empty(), NoScriptsExists);

        let mut export_config = Config::default();

        match alias {
            Some(ref a) => {
                let script = self.config.scripts.get(a).context(AliasNotFound {
                    alias: a.to_string(),
                })?;
                export_config.scripts.insert(a.clone(), script.clone());
            }
            None => {
                for (key, script) in self.config.scripts.iter() {
                    let include = match (&tag, &script.tags) {
                        (Some(filter_tag), Some(script_tags)) => script_tags.contains(filter_tag),
                        (Some(_), None) => false,
                        (None, _) => true,
                    };
                    if include {
                        export_config.scripts.insert(key.clone(), script.clone());
                    }
                }
            }
        }

        let config_string = toml::to_string_pretty(&export_config).context(TomlSerialize)?;
        Ok(config_string)
    }

    /// Import scripts from a TOML config file.
    pub fn import_scripts(&mut self, path: &std::path::Path, force: bool) -> PierResult<()> {
        let import_config = Config::from(&path.to_path_buf())?;

        for (key, script) in import_config.scripts.iter() {
            if !force {
                ensure!(
                    !self.config.scripts.contains_key(key),
                    AliasAlreadyExists { alias: key }
                );
            }
            self.config.scripts.insert(key.clone(), script.clone());
        }

        Ok(())
    }
}

pub fn open_editor(content: Option<&str>) -> PierResult<String> {
    let reader = match content {
        Some(txt) => scrawl::with(&txt),
        None => scrawl::new(),
    }
    .map_err(|e| PierError::EditorError {
        message: e.to_string(),
    })?;

    reader.to_string().map_err(|e| PierError::EditorError {
        message: e.to_string(),
    })
}
