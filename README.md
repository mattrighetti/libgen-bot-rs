# Libgen Bot

Libgen bot is a Telgram bot to interface with libgen

## Features
You can message anything to the bot and it will search by using the default search options in libgen, if you need something more granular you can use these commands:

|Command|Action|
|:-|:-|
|`/title <title>`|Search by title|
|`/author <author>`|Search by author|
|`/isbn <ISBN>`|Search by isbn|

## How to run
I use systemd to run bots, but you can also package it to run in a docker container

### Configuration
The bot can be configured using the following environment variables:
 * TELOXIDE_TOKEN: The telegram bot token. This is a required variable.
 * DB_PATH: Optional varable with the path where the bot will store simple analytics. It defaults to `db.sqlite`
 * LOG_PATH: Optional variable with the path to the `log4rs` config file. If not provided the default bundled configuration from `log.yml` will be used.

### Systemd configuration
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
