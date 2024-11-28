#!/usr/bin/env bash

set -eu

ARGPARSE_CONFIG='
name "example"
description "foo the bars or whatever"
version "6.6.6"
handler "main"

infer-subcommands true
// always-call-handler true

opt "foo" short="f" long="foo" value-name="FOOS" description="how much foo"
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
'

name=
qux=
key=
value=

main() {
  echo "hello ${name}!"

  if [[ -n "${debug+x}" ]]; then
    echo "debug is on!"
  fi

  if [[ -z "${foo+x}" ]]; then
    echo "foo is not set!"
  else
    echo "foo is $foo"
  fi

  for q in "${qux[@]}"; do
    echo "qux: $q"
  done
}

bar_write_cmd() {
  echo "writing key \"$key\" with value \"$value\""
}

bar_read_cmd() {
  echo "reading key \"$key\""
}

argparse_eval="$(printf '%s' "$ARGPARSE_CONFIG" | target/debug/clap-sh bash -n "$0" -- "${@}")"
if [ $? != 0 ] ; then exit 1 ; fi
eval "$argparse_eval"
