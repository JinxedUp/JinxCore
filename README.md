# JinxCore

JinxCore is a utility plugin for Pumpkin servers, focused on clean UX, practical commands, and easy configuration. It includes chat formatting/filtering, Discord bridge, starter kit support, kits, and a growing set of admin and player commands.

JinxCore is maintained and will evolve alongside the pumpkin project and its plugin api, there were many great features planned for this plugin that were cancelled because theyre either impossible or extremely unstable to add with the current pumpkin builds

Pull Requests are welcomed at any point.

![JinxCore Commands](commands.png)

## Features
- 50+ commands
- Custom join/leave messages with color support
- Chat format + chat filter + anti-spam
- Discord bridge (bot) for chat, join/leave, death messages like DiscordSRV
- Scoreboard support
- Starter kit on first join
- Kits with cooldowns
- Utility/admin commands (gamemode, heal, feed, fly, god, speed, etc.)

## Commands (high-level)
Player:
- ```/online```, ```/near```, ```/coords```, `/me`, `/whoami`, `/playtime`, `/ping`, `/suicide`, `/calc`, `/flip`
- `/rules`, `/discord`, `/website`, `/store`, `/socials`, `/clearchat`
- `/kit <name>`

Admin:
- `/gmc` `/gms` `/gmsp` `/gma` (+ aliases: /c /s /sp /a)
- `/creative` `/survival` `/spectator` `/adventure`
- `/heal` `/feed` `/fly` `/god` `/speed`
- `/day` `/night` `/rain` `/clear` `/thunder`
- `/createkit <name> <delay>`
- `/starterkit`, `/delstarterkit`
- `/i` (alias of /give)
- `/pl` (lists plugins)
- `/jinx help`, `/jinx credits`, `/jinx health`, `/jinx reload`

Tip: Use `/jinx help` for the full paged command list.

## Discord bridge

JinxCore includes an optional built-in Discord bridge inspired by DiscordSRV.

The bridge focuses on reliable server-to-Discord communication, including:
- Chat relay
- Player join/leave messages
- Death messages

It is intentionally scoped to remain stable on current Pumpkin builds and does not aim to fully replicate DiscordSRV feature parity.

## Install
1) Build the plugin:
```
cargo build --release
```
2) Copy the built library to your Pumpkin plugins folder:
- Windows: `target/release/jinxcore.dll`
- Linux: `target/release/libjinxcore.so`
- macOS: `target/release/libjinxcore.dylib`

3) Start Pumpkin once to generate config files in `./plugins/JinxCore/`.

## Config
Config files are created under `./plugins/JinxCore/` on first boot.

Notable files:
- `config.yml`: chat formatting, join/leave, Discord bot, spam/filter, scoreboard, etc.
- `rules.txt`: content for `/rules`
- `socials.txt`: content for `/discord`, `/website`, `/store`, `/socials`
- `scoreboard.txt`: lines for the scoreboard
- `kits.yml`: kits created via `/createkit`
- `starterkit.yml`: starter kit created via `/starterkit`

## License
MIT

## Credits
- Jinx
