# ADA AntiBot

This bot bans spammers in Telegram chat.

## Run 
To run you should export environment variables:

 - `TELOXIDE_TOKEN` - telegram bot token
 - `TELEGRAM_ADMINS_GROUP_ID` - telegram group id for managing banned users
 - `TELEGRAM_TARGET_GROUP_ID` - identifier of the group protected from spammers
 
Example:
```shell script
$ TELOXIDE_TOKEN=<your token> TELEGRAM_ADMINS_GROUP_ID=-00000000 TELEGRAM_TARGET_GROUP_ID=-00000000 cargo run
```
