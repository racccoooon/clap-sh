#!/usr/bin/env bash

set -euo pipefail

read -r -d '' ARGPARSE_CONFIG <<'EOF' || true
name "example"
description "foo the bars or whatever"
version "6.6.6"
handler "main"

infer-subcommands true
// always-call-handler true

opt "foo" short="f" long="foo" value-name="FOOS" default="7" description="how much foo"
opt "qux" short="q" long="qux" value-name="QUX" repeated=true description="which quxes"

opt "name" short="n" long="name" default="world" description="the name to greet"

flag "debug" short="d" long="debug" description="debug the application"

// arg "input_files" value-name="FILE" description="input files" count="+"
// arg "output_file" value-name="FILE" description="output file"

subcommand "bar" {
    short-flag "B"
    description "bar subcommand"
    require-subcommand true
    flag "with_foo" short="W" long="with-foo" description="also do a foo or just bar"
    subcommand "write" {
        short-flag "w"
        description "write the bar"
        handler "bar_write_cmd"
        arg "key" value-name="KEY" description="key to write to"
        arg "value" value-name="VALUE" description="value to write"
    }
    subcommand "read" {
        short-flag "r"
        description "read the bar"
        handler "bar_read_cmd"
        arg "key" value-name="KEY" description="key to read from"
    }
}

subcommand "boo" {
    handler "boo_cmd"
}
EOF

name=

main() {
  echo "hello ${name}!"
}

argparse_eval="$(echo "$ARGPARSE_CONFIG" | target/debug/argparse-shell-rs "$0" "${@}")"
if [[ $? != 0 ]]; then exit 1 ;fi
#eval "$argparse_eval"
echo "$argparse_eval"