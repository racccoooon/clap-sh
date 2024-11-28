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
        format!("'{}'", s
            .replace("'", r#"'\''"#)
            .replace("\n", r#"'$'\n''"#)
            .replace("\t", r#"'$'\t''"#)
        )
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
        err_message.write_str(color_print::cstr!(r#"<bold><red>error:</red></bold> handler <yellow>'$1'</yellow> not found"#)).unwrap();

        print!(r#"
__argparse_handler_err () {{
  if [[ -t 1 ]]; then
    >&2 echo "{err_styled}"
  else
    >&2 echo "{err_unstyled}"
  fi
  exit 1
}}
"#, err_styled = err_message.ansi(), err_unstyled = err_message);
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
        err_message.write_str(color_print::cstr!(r#"<bold><red>error:</red></bold> handler <yellow>'$1'</yellow> not found"#)).unwrap();

        print!(r#"
__argparse_handler_err () {{
  if [ -t 1 ]; then
    >&2 printf '%s' "{err_styled}"
  else
    >&2 printf '%s' "{err_unstyled}"
  fi
  exit 1
}}
"#, err_styled = err_message.ansi(), err_unstyled = err_message);
        for handler in handlers {
            println!("type {handler} >/dev/null || __argparse_handler_err {handler}");
        }
    }
}