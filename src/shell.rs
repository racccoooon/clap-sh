use std::collections::HashMap;
use std::fmt::Write;
use crate::error::Error;
use crate::VarValue;

pub trait Shell {
    fn quote_string(&self, s: &str) -> String;
    fn set_string_var(&self, name: &str, value: &str);
    fn set_list_var(&self, name: &str, value: &Vec<String>);
    fn call_handler(&self, name: &str);

    fn print_error(&self, err: Error);

    fn check_handlers(&self, handlers: &Vec<String>);

    fn call_handlers(&self, handlers: &Vec<String>) {
        for handler in handlers {
            self.call_handler(handler);
        }
    }

    fn set_vars(&self, vars: &HashMap<String, VarValue>) {
        for (name, value) in vars {
            match value {
                VarValue::Unset => {
                    self.set_string_var(format!("{name}_not_set").as_str(), "y");
                }
                VarValue::Val(v) => {
                    self.set_string_var(name.as_str(), v.as_str());
                }
                VarValue::List(vs) => {
                    self.set_list_var(name.as_str(), vs);
                }
            }
        }
    }
}

pub struct BashZsh {}

impl Shell for BashZsh {
    fn quote_string(&self, s: &str) -> String {
        let escaped = s
            .replace("\\", r#"\\"#)
            .replace("'", r#"\'"#)
            .replace("\r", r#"\r"#)
            .replace("\n", r#"\n"#)
            .replace("\t", r#"\t"#)
            .replace("\x1b", r#"\e"#)
            ;

        if s.len() != escaped.len() {
            format!("$'{}'", escaped)
        } else {
            format!("'{}'", escaped)
        }
    }
    fn set_string_var(&self, name: &str, value: &str) {
        println!("{name}={v}", v = self.quote_string(value));
    }

    fn set_list_var(&self, name: &str, values: &Vec<String>) {
        println!("{name}=({values})", values = values.iter().map(|x|
            self.quote_string(x)
        ).collect::<Vec<_>>().join(" "));
    }

    fn call_handler(&self, name: &str) {
        println!("{name}");
    }

    fn print_error(&self, err: Error) {
        let message_unstyled = format!("{}", err.message());
        let message_styled = format!("{}", err.message().ansi());

        self.set_string_var("__argparse_error_unstyled", message_unstyled.as_str());
        self.set_string_var("__argparse_error_styled", message_styled.as_str());

        print!(r#"
if [[ -t 1 ]]; then
  {redirect} echo "${{__argparse_error_styled}}"
else
  {redirect} echo "${{__argparse_error_unstyled}}"
fi
exit 1
"#, redirect = if err.use_stderr() { ">&2" } else { "" });
    }

    fn check_handlers(&self, handlers: &Vec<String>) {
        let mut err_message = clap::builder::StyledStr::new();
        err_message.write_str(color_print::cstr!("<bold><red>error:</red></bold> handler <yellow>'%s'</yellow> not found\n")).unwrap();

        let err_unstyled = err_message.to_string();
        let err_styled = err_message.ansi().to_string();

        print!(r#"
__argparse_handler_err () {{
  if [[ -t 1 ]]; then
    >&2 printf {err_styled} "$1"
  else
    >&2 printf {err_unstyled} "$1"
  fi
  exit 1
}}
"#, err_styled = self.quote_string(err_styled.as_str()), err_unstyled = self.quote_string(err_unstyled.as_str()));
        for handler in handlers {
            println!("type -t {handler} >/dev/null || __argparse_handler_err {handler}");
        }
    }
}

pub struct Posix {}

impl Shell for Posix {
    fn quote_string(&self, s: &str) -> String {
        let escaped = s.replace("'", r#"'\''"#);
        let quoted = format!("'{}'", escaped);
        let quoted = quoted.strip_prefix("''").unwrap_or(quoted.as_str());
        let quoted = quoted.strip_suffix("''").unwrap_or(quoted);

        quoted.to_string()
    }
    fn set_string_var(&self, name: &str, value: &str) {
        println!("{name}={v}", v = self.quote_string(value));
    }

    fn set_list_var(&self, name: &str, values: &Vec<String>) {
        println!("{name}={values}", values = self.quote_string(values.iter().map(|x|
            self.quote_string(x)
        ).collect::<Vec<_>>().join(" ").as_str()));
    }

    fn call_handler(&self, name: &str) {
        println!("{name}");
    }

    fn print_error(&self, err: Error) {
        let message_unstyled = format!("{}", err.message());
        let message_styled = format!("{}", err.message().ansi());

        self.set_string_var("__argparse_error_unstyled", message_unstyled.as_str());
        self.set_string_var("__argparse_error_styled", message_styled.as_str());

        print!(r#"
if [ -t 1 ]; then
  {redirect} printf '%s' "${{__argparse_error_styled}}"
else
  {redirect} printf '%s' "${{__argparse_error_unstyled}}"
fi
exit 1
"#, redirect = if err.use_stderr() { ">&2" } else { "" });
    }

    fn check_handlers(&self, handlers: &Vec<String>) {
        let mut err_message = clap::builder::StyledStr::new();
        err_message.write_str(color_print::cstr!("<bold><red>error:</red></bold> handler <yellow>'%s'</yellow> not found\n")).unwrap();

        let err_unstyled = err_message.to_string();
        let err_styled = err_message.ansi().to_string();

        print!(r#"
__argparse_handler_err () {{
  if [ -t 1 ]; then
    >&2 printf {err_styled} "$1"
  else
    >&2 printf {err_unstyled} "$1"
  fi
  exit 1
}}
"#, err_styled = self.quote_string(err_styled.as_str()), err_unstyled = self.quote_string(err_unstyled.as_str()));
        for handler in handlers {
            println!("type {handler} >/dev/null || __argparse_handler_err {handler}");
        }
    }
}