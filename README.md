# oly
A CLI for managing olympiad problem solutions.

## Installation
You will need git and cargo (the Rust toolchain) to clone and build the project.
Alternatively, you can grab an executable from
[GitHub releases](https://github.com/antinomie8/oly/releases).

We provide an install script which will build the project, install the executable, the Zsh
completion, the typst header, the desktop entry file and will register `oly`
as a handler for its own URI scheme `oly://`.
```sh
curl -fsSL https://raw.githubusercontent.com/antinomie8/oly/main/install.sh | bash
```

To build the project yourself, you can
```sh
git clone https://github.com/antinomie8/oly
cd oly/rust-rewrite
cargo build --release
cp target/release/oly ~/.local/bin/oly # or anywhere in $PATH
```

```sh
# All of this is already done for you if you use the installation script

copy assets/app/oly.desktop ~/.local/share/applications/
copy assets/app/oly.svg ~/.local/share/icons/hicolor/scalable/apps/
xdg-mime default oly.desktop x-scheme-handler/oly

# if you use typst
cp -r assets/typst ~/.local/share/

# if you use zsh
sudo cp assets/extras/_oly /usr/local/share/zsh/site-functions/ # or anywhere in your $fpath

# if you use Neovim
cp assets/extras/oly.lua ~/.config/nvim/after/plugin/
```

## Usage
```
$ oly --help
usage: oly <cmd> [args [...]].

Available subcommands:
    add                          - add a problem to the database
    edit                         - edit an entry from the database
    gen                          - generate a PDF from a problem
    search                       - search problems by contest, metadata...
    show                         - print a problem statement
    list                         - list problems in the database
    alias                        - link a problem to another one
    rm                           - remove a problem and its solution file
    mv                           - rename a problem
  Run oly <cmd> --help for more information regarding a specific subcommand

Arguments:
    --help              -h       - Show this help message
    --config-file FILE  -c FILE  - Specify config file to use
    --verify-config              - Check whether the config has any errors
    --version           -v       - Print this binary's version
    --log-level LEVEL            - Set the log level
```

If you have `oly.desktop` installed (see above), then you can run `oly` from
your app launcher; it will open a terminal with `oly` running inside,
prompting you for a problem name and then editing this problem if it is already in
the database, adding it if not.

## Configuration
Configuration is done via a yaml file at `${XDG_CONFIG_HOME:-$HOME/.config}/oly/config.yaml`.

If it does not exist yet, it will be created and opened in your editor with an example config.

The default Typst header and LaTeX preamble are very minimal; you might want to use
your own instead, which you can do with the `packages`, `preamble`, and `preview`
config options. For instance, you can find a typst package compatible with the
default `preamble` and `preview` in
[the author's dotfiles](https://github.com/antinomie8/dotfiles/tree/main/etc/typst/antoine).

## Using the oly scheme handler
The `oly` typst header file creates an eponymous command which takes a problem name
as its first argument and an optional content as its second argument. It
creates a link that, when clicked, will automatically generate and open the
given problem through the `oly gen` command. This will only work if
the desktop entry file for `oly` is installed (see above).
