# What?

A tool to show how many over/undertime work hours you have. It connects to redmine, downloads the time entries in the given time frame (default: this month), downloads a list of holidays and prints a table. For me, it's exactly the same as the list from administration at work using the internal redmine server. Your milage might warry, be cautious. ;D

# How to build

You need a recent stable rust version, i.e. installed via `rustup`. And you need to export the server as environment variable (only during build time), i.e.:

    export REDMINE_SERVER_NAME=redmine.somedomain.x
    cargo build --release

# Usage

    redmine-helper --help

Example:

    redmine-helper --username MaxMustermann --to 2018-09-30 --from=2018-01-01 --teilzeit=0.8

## Days of absence
A file called `absence.json` needs to be be placed to `~/.config/redmine-helper/` to
configure your vacation or illness days. C.f. the folder `doc` for an example.

## Config file
Defaults for parameters like `--username` or `--teilzeit`
can be configured in `config.json`, same folder as `absence.json`.

# Building

    cargo build --release
