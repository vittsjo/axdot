extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate errln;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

extern crate copy_dir;

mod config;
use config::{Config, ConfigFormat};

mod cli;
mod utils;

pub fn apply(dry: bool, matches: &clap::ArgMatches, config: &Config) {
    let replace = matches.is_present("replace") || false;

    for dir in config.directories.iter() {
        utils::create_directory(dry, std::path::Path::new(dir)).unwrap();
    }

    for file in config.files.iter() {
        utils::create_file(dry, std::path::Path::new(file), replace).unwrap();
    }

    for (src, dest) in config.links.iter() {
        utils::create_symlink(
            dry,
            std::path::Path::new(src),
            std::path::Path::new(dest),
            replace,
        ).unwrap();
    }

    for (src, dest) in config.copy.iter() {
        utils::copy_path(
            dry,
            std::path::Path::new(src),
            std::path::Path::new(dest),
            replace,
        ).unwrap();
    }

    for cmd in config.commands.iter() {
        if dry {
            println!("Executing \"{}\"", cmd.join(" "));
        } else {
            utils::run_command(cmd)
                .map_err(|e| errln!("{:?}", e))
                .map(|cmd| {
                    cmd.map(|child| {
                        child
                            .wait_with_output()
                            .map_err(|e| errln!("{:?}", e))
                            .unwrap()
                    }).unwrap()
                })
                .unwrap();
        }
    }
}

fn main() {

    let mut cli = cli::build_cli();
    let matches = cli.clone().get_matches();

    match matches.subcommand() {
        (_, None) => {
            cli.print_help().unwrap();
        }
        ("version", Some(_)) => {
            println!("{} {}", crate_name!(), crate_version!());
        }
        ("completions", Some(cmd)) => {
            let shell = match cmd.subcommand() {
                ("bash", Some(_)) => clap::Shell::Bash,
                ("fish", Some(_)) => clap::Shell::Fish,
                ("zsh", Some(_)) => clap::Shell::Zsh,
                ("powershell", Some(_)) => clap::Shell::PowerShell,
                _ => {
                    errln!("{}", cmd.usage());
                    return;
                }
            };
            cli.gen_completions_to(crate_name!(), shell, &mut std::io::stdout());
        }
        ("init", Some(cmd)) => {
            let config_format = cli::extract_format(cmd);
            let replace = cmd.is_present("replace") || false;
            let config_file = match matches.value_of("config") {
                Some(config_file) => utils::expand_user(std::path::Path::new(config_file)),
                None => {
                    match config_format {
                        ConfigFormat::JSON => std::path::PathBuf::from(
                            format!("{}.json", crate_name!()),
                        ),                
                        ConfigFormat::YAML => std::path::PathBuf::from(
                            format!("{}.yaml", crate_name!()),
                        ),                
                    }
                }
            };

            if config_file.exists() && !replace {
                println!("{:?} already exists", config_file);
                return;
            }

            Config::new()
                .save_file(&config_file, config_format)
                .map(|_| {
                    println!("Created configuration file: {:?}", config_file)
                })
                .map_err(|e| println!("{}", e))
                .unwrap();
        }
        _ => {
            let config_file = match matches.value_of("config") {
                Some(config_file) => utils::expand_user(std::path::Path::new(config_file)),
                None => {
                    let default_yaml_file =
                        std::path::PathBuf::from(format!("{}.yaml", crate_name!()));
                    let default_json_file =
                        std::path::PathBuf::from(format!("{}.json", crate_name!()));

                    if default_yaml_file.exists() {
                        default_yaml_file
                    } else if default_json_file.exists() {
                        default_json_file
                    } else {
                        errln!("Configuration file does not exist");
                        return;
                    }
                }    
            };

            let config = match Config::load_file(&config_file) {
                Ok(config) => config,
                Err(e) => {
                    errln!("{}", e);
                    return;
                }
            };

            if config_file.has_root() {
                match config_file.parent() {
                    Some(parent) => {
                        if let Err(e) = std::env::set_current_dir(parent) {
                            errln!("{}", e);
                        }
                    }
                    None => {
                        return;
                    }
                };
            }

            match matches.subcommand() {
                ("show-config", Some(cmd)) => {
                    println!(
                        "{}",
                        match cli::extract_format(cmd) {
                            ConfigFormat::JSON => config.to_json_string(),
                            ConfigFormat::YAML => config.to_yaml_string(),
                        }
                    );
                }
                ("backup", Some(_)) => {
                    // TODO
                }
                ("apply", Some(cmd)) => {
                    apply(false, &cmd, &config);
                }
                ("dry-apply", Some(cmd)) => {
                    apply(true, &cmd, &config);
                }
                _ => {
                    cli.print_help().unwrap();
                }
            }
        }
    }
}
