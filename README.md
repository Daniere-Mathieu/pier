# **pier** ~ A CLI to organize and run short Unix shell scripts
[![Build Status](https://github.com/Daniere-Mathieu/pier/actions/workflows/ci.yml/badge.svg)](https://github.com/Daniere-Mathieu/pier/actions/workflows/ci.yml)
[![Security](https://github.com/Daniere-Mathieu/pier/actions/workflows/security.yml/badge.svg)](https://github.com/Daniere-Mathieu/pier/actions/workflows/security.yml)
![crates.io](https://img.shields.io/crates/v/pier.svg)

A central repository to manage all your one-liners, scripts, tools, and CLIs. Add, remove, list, and run scripts - storing metadata to easily find them later. No more digging through your `bin` folder...

> **Note:** This is a fork of [pier-cli/pier](https://github.com/pier-cli/pier). It adds new features (shell completions, import/export, dry-run, namespaces, argument pass-through) and modernizes the codebase to Rust 2021 / stable with up-to-date dependencies. These additions are **not battle-tested yet** — use at your own risk and feel free to report issues.

![Boat pier](https://raw.githubusercontent.com/BenSchZA/pier/master/.media/boat-dock.png)

## Description

If you've spent any amount of time in the terminal you no doubt have built up a lovely collection of one-liners, scripts, useful tools, and CLIs. Whenever you want to use them you dig through your `bin` folder trying to remember what you called the script... Linux users love hard-to-remember naming conventions.

Scripts should be first-class citizens. In a GUI world we can find our programs using a menu of sorts. In the terminal scripts get lost.

The idea behind `pier` is to create a central repository for all your scripts, and provide a way to attach metadata about these scripts. Using `pier` you can add, remove, list, and run scripts. These can be managed by `pier` in a human-readable TOML config, or you can use it to catalog existing scripts that you may have lying around - you'd then simply add the metadata for the specific script, and attach it to the name in the `PATH`.

## Installation

**One-line install** (Linux / macOS):
```sh
curl -fsSL https://raw.githubusercontent.com/Daniere-Mathieu/pier/master/install.sh | sh
```

This downloads the latest release binary and installs it to `~/.local/bin`. Set `PIER_INSTALL_DIR` to change the location:
```sh
curl -fsSL https://raw.githubusercontent.com/Daniere-Mathieu/pier/master/install.sh | PIER_INSTALL_DIR=/usr/local/bin sh
```

**From source** with Cargo:
```sh
cargo install --git https://github.com/Daniere-Mathieu/pier.git
```

**From Crates.io**: `cargo install pier`


## Operation

Use `pier --help` to display help on commandline or see `src/cli.rs` for a more detailed spec.
```
A simple script management CLI

Usage: pier [OPTIONS] <ALIAS> [ARGS]...
       pier [OPTIONS] [ALIAS] [ARGS]... <COMMAND>

Commands:
  add          Add a new script to config
  remove       alias: rm - Remove a script matching alias
  config-init  alias: init - Add a config file
  edit         Edit a script matching alias
  show         Show a script matching alias
  run          Run a script matching alias
  list         alias: ls - List scripts
  copy         alias: cp - Copy existing alias to the new one
  completions  Generate shell completions for the given shell
  move         alias: mv, rename - Move/rename existing alias to the new one
  export       Export scripts from the config
  import       Import scripts from a file
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose              The level of verbosity
      --dry-run              Preview the command without executing it
  -c, --config-file <PATH>   Sets a custom config file [env: PIER_CONFIG_PATH=]
  -h, --help                 Print help
```

* `pier list`
* `pier add "ip link set wlp58s0 down && sleep 5 && ip link set wlp58s0 up" --alias refresh-wifi`
* `pier refresh-wifi`

## Example `pier` TOML config

```toml
[scripts.refresh-wifi]
command = "ip link set wlp58s0 down && sleep 5 && ip link set wlp58s0 up"

[scripts.twa-analyze]
command = "docker run --rm -t trailofbits/twa -vw"
tags = [ "infosec" ]

[scripts.enabled-services]
command = "systemctl list-unit-files --state=enabled"

[scripts.flush-docker]
command = "docker container stop $(docker container ls -a -q) && docker system prune -a -f --volumes"
description = "A script to clear out old Docker containers and images"
tags = [ "docker", "flush" ]
```

## Example `pier list` output

```
▶ pier list           
 Alias             | Command 
-------------------+----------------------------------------------------------------------------------------------------------------
 fromscratch       | appimage-run ~/AppImage/FromScratch.1.4.3.AppImage 
 nspawn-bionic     | sudo systemd-nspawn --bind=/tmp/.X11-unix -D /var/lib/machines/bionic --bind /home/bscholtz:/home/bscholtz 
 bspwm-refresh     | .config/bspwm/bspwmrc 
 flush-docker      | docker container stop $(docker container ls -a -q) && docker system prune -a -f --volumes 
 zfs-compression   | sudo zfs get all | grep compressratio 
 mongo-docker      | docker run --name mongodb -d mongo:latest 
 forward-mongo     | kubectl port-forward mongo-molecule-set-0 27018:27017 
 refresh-wifi      | ip link set wlp58s0 down && sleep 5 && ip link set wlp58s0 up 
 lepton            | appimage-run ~/AppImage/Lepton-1.8.0-x86_64.AppImage 
 ledger            | appimage-run AppImage/ledger-live-desktop-1.6.0-linux-x86_64.AppImage 
 reload-urxvt      | xrdb ~/.Xresources 
 kill-docker       | rm -rf /var/lib/docker 
 reload-xresources | xrdb ~/.Xresources 
 graphiql          | appimage-run ~/AppImage/graphiql-app-0.7.2-x86_64.AppImage 
 enabled-services  | systemctl list-unit-files --state=enabled 
 ports             | netstat -tulpn 
 chmod-copy        | chmod --reference= 
 zfs-drop-caches   | sync; echo 2 | sudo tee /proc/sys/vm/drop_caches 
 update            | sudo nix-channel --update && sudo nixos-rebuild switch 
 flush-untagged    | docker images -q --filter dangling=true | xargs -r docker rmi 
 twa-analyze       | docker run --rm -t trailofbits/twa -vw 
 parity-ubuntu     | docker image pull yodascholtz/parity-ubuntu:latest && docker run -p 8545:8545 yodascholtz/parity-ubuntu:latest
```
## Execute pier scripts in any interpreted languages
Scripts starting with a shebang `#!` will be run with the specified interpeter just like it would in a normal script. Pier does this by creating a temp file from your script, executing it and then finally cleaning the file up. This allows you to write your pier script in python, node.js etc. even compiled languages can be run if using something like scriptisto.

#### Shebang example config

```
[scripts.run_rust_script]
command = '''
#!/usr/bin/env scriptisto

// scriptisto-begin
// script_src: src/main.rs
// build_cmd: cargo build --release && strip ./target/release/script
// target_bin: ./target/release/script
// files:
//  - path: Cargo.toml
//    content: |
//     package = { name = "script", version = "0.1.0", edition = "2018"}
//     [dependencies]
// scriptisto-end


fn main() {
    println!("This is a rust script!");
}

'''

[scripts.run_python]
command = '''
#!/usr/bin/env python3
import sys

print("Running python with version {}".format(sys.version))

'''
```

### Setting the default shell / interpreter
By default if no shebang is specified pier will try to use your default shell to execute the script inline. This can be overwritten with the variable interpreter. It needs to be a list with the first item being the binary and the rest is any flags necessary.

#### Default interpreter example config
```
# Sets the default interpreter, the first item in the list should be the binary and the rest are the arguments for the interpreter cli option.
[default]
interpreter = ["node", "-e"]

# Runs as the fallback interpreter nodejs as it's lacking a shebang
[scripts.hello_world_nodejs]
command = '''
console.log("Hello world!")

'''

# This will be run as a posix sh script as it has a shebang
[scripts.a_shell_script]
command = '''
#!/bin/sh

nohup st > /dev/null 2>&1&

'''
```

## Origin

Originally created by [BenSchZA](https://github.com/BenSchZA) and intended as a way to manage Docker one-liners, the name `pier` continues along the same maritime theme. This fork picks up where the original left off, with updated tooling and new features.

## Shell Completions

Generate completions for your shell and source them:

```sh
# Bash
pier completions bash > ~/.local/share/bash-completion/completions/pier

# Zsh
pier completions zsh > ~/.zfunc/_pier

# Fish
pier completions fish > ~/.config/fish/completions/pier.fish
```
