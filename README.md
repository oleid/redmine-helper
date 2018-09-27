# What?

A tool to show how many over/undertime work hours you have. It connects to redmine, downloads the time entries in the given time frame (default: this month), downloads a list of holidays and prints a table. For me, it's exactly the same as the list from administration. Your milage might warry, be cautious. ;D

# Usage

    redmine-helper --help

Example:

    redmine-helper --username OlafLeidinger --to 2018-09-30 --from=2018-01-01 --teilzeit=0.8


# Building

    cargo build --release

## Precompiled version

Please find it in `~/shares/mitarbeiter/oleidinger/redmine-helper`
