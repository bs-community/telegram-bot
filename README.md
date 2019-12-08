# Telegram Bot

This is a Telegram bot for publishing news to the channel.

## How to use?

Install Rust with rustup first. Then, run in command line:

```sh
export TELEGRAM_BOT_TOKEN=...
export TELEGRAM_CHAT_ID=...  # It's the channel ID.
cargo build --release
```

Copy the binary file to anywhere you like or you need.

### Analyzing Blessing Skin

Make sure the current working directory is at the repository of Blessing Skin.
Run:

```sh
./bot diff
```

That's it.

### Plugins Update

Prepare the plugins JSON file first.
Suppose we have a `plugins.json` file like this:

```json
[
    { "name": "a", "version": "1.0.0" },
    // can be more...
]
```

Then run:

```sh
./bot plugin plugins.json
```

## License

MIT License

2019-present (c) The Blessing Skin Team
