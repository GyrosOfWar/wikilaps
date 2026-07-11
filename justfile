set shell := ["nu", "-c"]

project := justfile_directory()
frontend := project + "/frontend"
backend := project + "/backend"

default:
  @just --list

@backend *cmd:
    cd {{backend}}; just {{cmd}}

@frontend *cmd:
    cd {{frontend}}; just {{cmd}}

format:
    just backend format
    just frontend format

check:
    just backend check
    just frontend check

test:
    just backend test
    just frontend test

generate:
    just frontend generate

start:
    mprocs --config etc/services.yaml
