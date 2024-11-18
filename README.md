# Corkboard: A tiny RSS feed manager

A minimalist RSS feed client for your terminal.

- Manages your RSS feeds while leaving the reading for the browser.

- Easily scriptable, meant to be a part of your shell tools.

- Uses SQLite as the backing database, so you may interface with your data however you like.

**WORK IN PROGRESS**

## Installing

Requires _cargo_ and _rustc_

1. Clone the repository
```
$ cd $DESIRED_INSTALL_DIR
$ git clone https://github.com/angelAlk/corkboard.git
```

2. Build the app
```
$ cd corkboard
$ cargo build --release
```
Expect compilation to take a bit.

3. Add to your path

For bash users:
```
$ echo -e "export PATH=\$PATH:$(pwd)/target/release/corkboard" >> ~/.bashrc
```

The executable is self contained.
You may move it somewhere else after compiling.

## Usage

```
$ corkboard --help

Minimal RSS client

usage: corkboard <command>

Commands:
  add <url>             Subscribe to a feed with url <url>.
  up                    Update all feeds then display all the items/posts that were added.
  feeds                 List all subscribed feeds.
  new                   Show all items not marked as read (does not update channels).
  mark <number>         Mark an item at position <number> as read. Positions come from corkboard new.
  remove <url>          Unsuscribe from a feed and delete all of it's items from the database.
  help                  Show this help message.
```

## Custom database location

The database is installed by default under `$XDG_DATA_HOME`.
If `$XDG_DATA_HOME` is not available it uses the default value: `$HOME/.local/share/`.

For both of these the sqlite file is at: `<LOCATION>/corkboard/corkdb`.

Use a custom database path by setting the variable (in your shell environment): `$CORKDB`

