# Libgen Bot

[Libgen bot](https://t.me/libgen1bot) is a Telgram bot to interface with libgen

## Features

You can message anything to the bot and it will search by using the default
search options in libgen, if you need something more granular you can use these
commands:

| Command            | Action           |
| :----------------- | :--------------- |
| `/title <title>`   | Search by title  |
| `/author <author>` | Search by author |
| `/isbn <ISBN>`     | Search by isbn   |

## Running using docker

The bot requires three environment variables to run:

- **TELOXIDE_TOKEN**: Telegram bot token
- **DB_PATH**: Path to the analytics sqlite database file. Use `db.sqlite` for
  default behavior
- **LOG_PATH**: Path to the log4rs log format file. Use `log.yml` for default
  behavior

### docker-compose

```yaml
version: '3'

services:
  libgen-bot:
    image: <TODO>
    container_name: libgen-bot
    environment:
      - TELOXIDE_TOKEN=<token>
      - DB_PATH=db.sqlite
    volumes:
      # It's possible to mount the db file to a volume to persist the data and allow inspection from host
      - /path/to/my/db.sqlite:/app/db.sqlite
```

## Running using systemd

### Configuration

Create a service file, e.g. `/etc/systemd/system/bot.service`

```
[Unit]
Description=Telegram Bot Service
After=network.target

[Service]
Type=simple
User=<user>
Group=<group>
Restart=always
RestartSec=10
ExecStart=/usr/local/bin/bot
Environment="TELOXIDE_TOKEN=<token>"
Environment="DB_PATH=db.sqlite"
Environment="LOG_PATH=log.yml"

[Install]
WantedBy=multi-user.target
```

Copy the bot binary in `/usr/local/bin/`

### Run

Enable the service so that it boots on reboot and start it

```
$ sudo systemctl enable bot.service
$ sudo systemctl start bot.service
```
