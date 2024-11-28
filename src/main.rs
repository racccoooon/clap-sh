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


fn do_things<'a>(shell: &Box<dyn Shell>, config: Option<&String>, progname: Option<String>, args: impl Iterator<Item=&'a String>) -> Result<(), Error> {
    let config_str = if let Some(str) = config {
        str.to_string()
    } else {
        read_to_string(std::io::stdin())?
    };

    let config = config::parse(config_str.as_str())?;

    let progname = vec![progname.or(config.name).expect("missing program name, supply in config or via command line")];

    let args = progname.iter().chain(args.map(|x|x));

    let mut cmd = build_command(&config.command)
        .infer_subcommands(config.infer_subcommands)
        .args_override_self(config.args_override_self);

    if let Some(version) = config.version {
        cmd = cmd.version(version);
    }

    let matches = cmd.clone().try_get_matches_from(args)?;

    let mut vars: HashMap<String, VarValue> = HashMap::new();
    let mut handlers: Vec<String> = Vec::new();
    handle_matches(&matches, &config.command, &mut vars, &mut handlers);
    if !handlers.is_empty() {
        shell.check_handlers(&handlers);
    }
    shell.set_vars(&vars);
    shell.call_handlers(&handlers);

    Ok(())
}

fn main() {
    let matches = clap::command!()
        .arg(
            clap::arg!(
                -c --config <CONFIG> "config string, stdin is used if not provided"
            ).global(true)
        )
        .arg(
            clap::arg!(
                -n --progname <NAME> "the name of the program to use in error messages and help text"
            ).global(true)
        )
        .arg(
            clap::arg!(
                -N --"progname-in-args" "the first argument is used as the program name"
            ).global(true).conflicts_with("progname")
        )
        .subcommand(
            clap::Command::new("bash")
                .alias("zsh")
                .about("generate bash/zsh-compatible code")
                .arg(clap::arg!([args] ... "arguments to parse")
                    .trailing_var_arg(true))
        )
        .subcommand(
            clap::Command::new("posix")
                .about("generate posix-compatible code")
                .arg(clap::arg!([args] ... "arguments to parse")
                    .trailing_var_arg(true))
        )
        .subcommand_required(true)
        .get_matches();


    let (cmd, matches) = matches.subcommand().unwrap();

    let config = matches.get_one::<String>("config");

    let mut args = matches.get_many::<String>("args")
        .unwrap_or(clap::parser::ValuesRef::default());

    let progname_in_args = matches.get_flag("progname-in-args");

    let progname = if progname_in_args {
        Some(args.next().expect("missing program name in arguments").to_string())
    } else if let Some(progname) = matches.get_one::<String>("progname") {
        Some(progname.to_string())
    } else {
        None
    };


    let shell: Box<dyn Shell> = if cmd == "bash" {
        Box::new(shell::BashZsh {})
    } else if cmd == "posix" {
        Box::new(shell::Posix {})
    } else {
        panic!("invalid command");
    };


    match do_things(&shell, config, progname, args) {
        Ok(_) => {}
        Err(err) => {
            shell.print_error(err);
        }
    }
}
