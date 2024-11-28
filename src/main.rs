use std::collections::HashMap;
use std::io::{read_to_string, Read};
use clap::ArgMatches;
use crate::config::Count;
use crate::error::Error;
use crate::shell::Shell;

mod config;
mod shell;
mod error;

fn build_command(config: &config::Command) -> clap::Command {
    let mut cmd = clap::Command::new(&config.name)
        .subcommand_required(config.require_subcommand);

    if let Some(description) = &config.description {
        cmd = cmd.about(description);
    }

    if let Some(short) = &config.short_flag {
        cmd = cmd.short_flag(*short);
    }

    if let Some(long) = &config.long_flag {
        cmd = cmd.long_flag(long);
    }

    for flag in &config.flags {
        cmd = cmd.arg(build_flag(&flag));
    }

    for opt in &config.opts {
        cmd = cmd.arg(build_opt(&opt));
    }

    for arg in &config.args {
        cmd = cmd.arg(build_arg(&arg));
    }

    for subcommand in &config.subcommands {
        cmd = cmd.subcommand(build_command(&subcommand));
    }

    cmd
}

fn build_flag(config: &config::Flag) -> clap::Arg {
    let mut arg = clap::Arg::new(&config.name).action(clap::ArgAction::Count);
    if let Some(short) = config.short {
        arg = arg.short(short);
    }
    if let Some(long) = &config.long {
        arg = arg.long(long);
    }
    if let Some(description) = &config.description {
        arg = arg.help(description);
    }

    arg
}

fn build_opt(config: &config::Opt) -> clap::Arg {
    let mut arg = clap::Arg::new(&config.name).value_name(&config.value_name);
    if let Some(short) = config.short {
        arg = arg.short(short);
    }
    if let Some(long) = &config.long {
        arg = arg.long(long);
    }
    if let Some(description) = &config.description {
        arg = arg.help(description);
    }
    if config.repeated {
        arg = arg.action(clap::ArgAction::Append);
    }

    arg
}
fn build_arg(config: &config::Arg) -> clap::Arg {
    let mut arg = clap::Arg::new(&config.name).value_name(&config.value_name).required(true);

    if let Some(description) = &config.description {
        arg = arg.help(description);
    }

    match config.count {
        Count::One => (),
        Count::AtLeastOne => {
            arg = arg.num_args(1..)
        }
        Count::Any => {
            arg = arg.num_args(0..)
        }
        Count::Exactly(_) => {}
        Count::MinMax(_, _) => {}
    }

    arg
}


#[derive(Debug, Clone)]
enum VarValue {
    Unset,
    Val(String),
    List(Vec<String>),
}


fn handle_matches(matches: &ArgMatches, config: &config::Command, vars: &mut HashMap<String, VarValue>, handlers: &mut Vec<String>) {
    for opt in &config.opts {
        let val = if opt.repeated {
            VarValue::List(match matches.get_many::<String>(opt.name.as_str()) {
                None => if let Some(default) = &opt.default {
                    vec![default.to_string()]
                } else {
                    vec![]
                },
                Some(v) => v.map(|x| x.to_string()).collect(),
            })
        } else {
            match matches.get_one::<String>(opt.name.as_str()) {
                None => if let Some(default) = &opt.default {
                    VarValue::Val(default.to_string())
                } else {
                    VarValue::Unset
                },
                Some(v) => VarValue::Val(v.clone()),
            }
        };
        vars.insert(opt.name.clone(), val);
    }

    for flag in &config.flags {
        let count = matches.get_count(flag.name.as_str());
        vars.insert(flag.name.clone(), if count == 0 {
            VarValue::Unset
        } else {
            VarValue::Val(count.to_string())
        });
    }

    for arg in &config.args {
        match arg.count {
            Count::One => {
                vars.insert(arg.name.clone(), match matches.get_one::<String>(arg.name.as_str()) {
                    None => VarValue::Unset,
                    Some(v) => VarValue::Val(v.clone()),
                });
            }
            _ => {
                vars.insert(arg.name.clone(), match matches.get_many::<String>(arg.name.as_str()) {
                    None => VarValue::List(vec![]),
                    Some(v) => VarValue::List(v.map(|x| x.clone()).collect()),
                });
            }
        }
    }


    match matches.subcommand() {
        None => {
            if let Some(handler) = &config.handler {
                handlers.push(handler.to_string());
            }
        }
        Some((name, matches)) => {
            if config.always_call_handler {
                if let Some(handler) = &config.handler {
                    handlers.push(handler.to_string());
                }
            }

            for cmd in &config.subcommands {
                if cmd.name == name {
                    handle_matches(matches, cmd, vars, handlers);
                    break;
                }
            }
        }
    }
}


fn do_things(shell: &Box<dyn Shell>) -> Result<(), Error> {
    let mut config_str = read_to_string(std::io::stdin())?;
    std::io::stdin().read_to_string(&mut config_str)?;
    let config = config::parse(config_str.as_str())?;

    let mut cmd = build_command(&config.command)
        .infer_subcommands(config.infer_subcommands)
        .args_override_self(config.args_override_self);

    if let Some(version) = config.version {
        cmd = cmd.version(version);
    }


    let matches = cmd.clone().try_get_matches_from(
        std::env::args().skip(1).collect::<Vec<String>>())?;

    let mut vars: HashMap<String, VarValue> = HashMap::new();
    let mut handlers: Vec<String> = Vec::new();
    handle_matches(&matches, &config.command, &mut vars, &mut handlers);
    shell.check_handlers(&handlers);
    shell.write_vars(&vars);
    shell.write_handlers(&handlers);

    Ok(())
}

fn main() {
    let shell: Box<dyn Shell> = Box::new(shell::BashZsh {});

    match do_things(&shell) {
        Ok(_) => {}
        Err(err) => {
            shell.write_error(err);
        }
    }
}
