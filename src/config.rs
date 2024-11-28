#[derive(knus::Decode, Debug, Clone)]
struct CfgOpt {
    #[knus(argument)]
    name: String,
    #[knus(property)]
    short: Option<String>,
    #[knus(property)]
    long: Option<String>,
    #[knus(property)]
    value_name: Option<String>,
    #[knus(property)]
    default: Option<String>,
    #[knus(property)]
    repeated: Option<bool>,
    #[knus(property)]
    description: Option<String>,
}


#[derive(knus::Decode, Debug, Clone)]
struct CfgFlag {
    #[knus(argument)]
    name: String,
    #[knus(property)]
    short: Option<String>,
    #[knus(property)]
    long: Option<String>,
    #[knus(property)]
    description: Option<String>,
}

#[derive(knus::Decode, Debug, Clone)]
struct CfgArg {
    #[knus(argument)]
    name: String,
    #[knus(property)]
    value_name: Option<String>,
    #[knus(property)]
    count: Option<String>,
    #[knus(property)]
    description: Option<String>,
}


#[derive(knus::Decode, Debug, Clone)]
struct CfgCommand {
    #[knus(argument)]
    name: String,
    #[knus(child, unwrap(argument))]
    short_flag: Option<String>,
    #[knus(child, unwrap(argument))]
    long_flag: Option<String>,
    #[knus(child, unwrap(argument))]
    require_subcommand: Option<bool>,
    #[knus(child, unwrap(argument))]
    description: Option<String>,
    #[knus(child, unwrap(argument))]
    handler: Option<String>,
    #[knus(child, unwrap(argument))]
    always_call_handler: Option<bool>,
    #[knus(children(name = "subcommand"))]
    subcommands: Vec<CfgCommand>,
    #[knus(children(name = "opt"))]
    opts: Vec<CfgOpt>,
    #[knus(children(name = "flag"))]
    flags: Vec<CfgFlag>,
    #[knus(children(name = "arg"))]
    args: Vec<CfgArg>,
}


#[derive(knus::Decode, Debug, Clone)]
struct CfgApp {
    #[knus(child, unwrap(argument))]
    name: Option<String>,
    #[knus(child, unwrap(argument))]
    version: Option<String>,
    #[knus(child, unwrap(argument))]
    infer_subcommands: Option<bool>,
    #[knus(child, unwrap(argument))]
    args_override_self: Option<bool>,
    #[knus(child, unwrap(argument))]
    description: Option<String>,
    #[knus(child, unwrap(argument))]
    handler: Option<String>,
    #[knus(child, unwrap(argument))]
    always_call_handler: Option<bool>,
    #[knus(child, unwrap(argument))]
    require_subcommand: Option<bool>,
    #[knus(children(name = "subcommand"))]
    subcommands: Vec<CfgCommand>,
    #[knus(children(name = "opt"))]
    opts: Vec<CfgOpt>,
    #[knus(children(name = "flag"))]
    flags: Vec<CfgFlag>,
    #[knus(children(name = "arg"))]
    args: Vec<CfgArg>,
}

#[derive(Debug, Clone)]
pub struct App {
    pub version: Option<String>,
    pub infer_subcommands: bool,
    pub args_override_self: bool,
    pub command: Command,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub short_flag: Option<char>,
    pub long_flag: Option<String>,
    pub require_subcommand: bool,
    pub description: Option<String>,
    pub handler: Option<String>,
    pub always_call_handler: bool,
    pub flags: Vec<Flag>,
    pub opts: Vec<Opt>,
    pub args: Vec<Arg>,
    pub subcommands: Vec<Command>,
}

#[derive(Debug, Clone)]
pub struct Flag {
    pub name: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Opt {
    pub name: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub description: Option<String>,
    pub value_name: String,
    pub default: Option<String>,
    pub repeated: bool,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub description: Option<String>,
    pub value_name: String,
    pub count: Count,
}


#[derive(Debug, Copy, Clone)]
pub enum Count {
    One,
    AtLeastOne,
    Any,
    Exactly(usize),
    MinMax(usize, usize),
}

impl Into<Opt> for CfgOpt {
    fn into(self) -> Opt {
        Opt {
            name: self.name.clone(),
            short: short_flag(self.short),
            long: self.long,
            description: self.description,
            value_name: self.value_name.unwrap_or(self.name.to_uppercase()),
            default: self.default,
            repeated: self.repeated.unwrap_or(false),
        }
    }
}


fn short_flag(name: Option<String>) -> Option<char> {
    match name {
        Some(s) => {
            if s.chars().count() == 1 {
                s.chars().nth(0)
            } else {
                panic!("invalid short name (must be exactly one character)")
            }
        }
        _ => None,
    }
}

impl Into<Flag> for CfgFlag {
    fn into(self) -> Flag {
        Flag {
            name: self.name.clone(),
            short: short_flag(self.short),
            long: self.long,
            description: self.description,
        }
    }
}

impl Into<Arg> for CfgArg {
    fn into(self) -> Arg {
        Arg {
            name: self.name.clone(),
            description: self.description,
            value_name: self.value_name.unwrap_or(self.name.to_uppercase()),
            count: match self.count {
                Some(s) => match s.as_str() {
                    "1" => Count::One,
                    "+" => Count::AtLeastOne,
                    "*" => Count::Any,
                    _ => panic!("invalid count")
                },
                None => Count::One,
            },
        }
    }
}

impl Into<Command> for CfgCommand {
    fn into(self) -> Command {
        Command {
            name: self.name,
            short_flag: short_flag(self.short_flag),
            long_flag: self.long_flag,
            require_subcommand: self.require_subcommand.unwrap_or(false),
            description: self.description,
            handler: self.handler,
            always_call_handler: self.always_call_handler.unwrap_or(false),
            flags: self.flags.iter().map(|flag| flag.clone().into()).collect(),
            opts: self.opts.iter().map(|opt| opt.clone().into()).collect(),
            args: self.args.iter().map(|arg| arg.clone().into()).collect(),
            subcommands: self.subcommands.iter().map(|cmd| cmd.clone().into()).collect(),
        }
    }
}

impl Into<App> for CfgApp {
    fn into(self) -> App {
        App {
            version: self.version,
            infer_subcommands: self.infer_subcommands.unwrap_or(false),
            args_override_self: self.args_override_self.unwrap_or(true),
            command: Command {
                name: self.name.unwrap_or("".to_string()),
                short_flag: None,
                long_flag: None,
                require_subcommand: self.require_subcommand.unwrap_or(false),
                description: self.description,
                handler: self.handler,
                always_call_handler: self.always_call_handler.unwrap_or(false),
                flags: self.flags.iter().map(|flag| flag.clone().into()).collect(),
                opts: self.opts.iter().map(|opt| opt.clone().into()).collect(),
                args: self.args.iter().map(|arg| arg.clone().into()).collect(),
                subcommands: self.subcommands.iter().map(|cmd| cmd.clone().into()).collect(),
            }
        }
    }
}


pub fn parse(config_str: &str) -> miette::Result<App> {
    Ok(knus::parse::<CfgApp>("<config>", config_str)?.into())
}