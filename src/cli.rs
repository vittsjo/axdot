extern crate clap;

use clap::{App, Arg, SubCommand};
use config::ConfigFormat;

pub fn build_cli() -> App<'static, 'static> {
    let arg_format = Arg::with_name("format")
        .long("format")
        .takes_value(true)
        .help("format of configuration file");

    let arg_replace = Arg::with_name("replace").long("replace").short("r").help(
        "replace files/folders if they already exist",
    );

    App::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .takes_value(true)
                .help("the JSON/YAML file you want to use"),
        )
        .subcommand(SubCommand::with_name("version").about(
            "Show version of axdot",
        ))
        .subcommand(
            SubCommand::with_name("completions")
                .about("Generate shell completions")
                .subcommand(SubCommand::with_name("bash").about(
                    "Generate Bash completions",
                ))
                .subcommand(SubCommand::with_name("fish").about(
                    "Generate Fish completions",
                ))
                .subcommand(SubCommand::with_name("zsh").about(
                    "Generate Zsh completions",
                ))
                .subcommand(SubCommand::with_name("powershell").about(
                    "Generate PowerShell completions",
                )),
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("Generate configuration file")
                .arg(arg_replace.clone())
                .arg(arg_format.clone()),
        )
        .subcommand(SubCommand::with_name("backup"))
        .subcommand(SubCommand::with_name("apply").arg(arg_replace.clone()))
        .subcommand(SubCommand::with_name("dry-apply").arg(arg_replace.clone()))
        .subcommand(
            SubCommand::with_name("show-config")
                .about("Show content of configuration file")
                .arg(arg_format.clone()),
        )

}

pub fn extract_format(cmd: &clap::ArgMatches) -> ConfigFormat {
    match cmd.value_of("format") {
        Some(f) => {
            match f {
                "json" => ConfigFormat::JSON,
                "yaml" | "yml" | _ => ConfigFormat::YAML,
            }
        }
        None => ConfigFormat::YAML,
    }
}
