# Nine Saves 
![Image of GUI](./assets/readme_image.png) 

Nine Saves is a simple, unofficial save manager application for Nine Souls.

Currently, the game has only four save slots available, and doesn't let you save
to them freely. This tool allows you to back up your save slots with
useful names, as well as load save backups into your existing slots.

Note: Nine Saves has no affiliation with Red Candle Games - they are
not liable for any issues experienced while using this software.

## Safety
All destructive operations back up files before making any
irreversible changes. The risk of losing data with Nine Saves is low.
Still, you should exercise some caution; this tool is early in
development.

## Installation
### Releases
Binary executables for Windows, Mac, and Linux are available in the [releases tab](https://github.com/semilin/nine_saves/latest).
### From Source
Building from source requires the Rust toolchain.
```sh
git clone https://github.com/semilin/nine_saves
cd nine_saves
cargo install --path .
```

## Where are my extra saves and backups stored?
### Windows
`C:\Users\YOURUSERNAME\AppData\Roaming\nine_saves\`
### MacOS
`/Users/YOURUSERNAME/Library/Application Support/nine_saves/`
### Linux
`~/.local/share/nine_saves/`
