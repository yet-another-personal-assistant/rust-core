# PA infrastructure core

Core infrastructure service for PA message routing.

Messaging protocol is described in [PA infrastructure protocol].

## Current implementation

The service listens fixed port: 6142.

If incoming message matches the following:

- `from->channel` equal to `"tg"`
- `from->chat_id` equal to `from->user_id` (private chat)

The service replies to the same client with "dont-understand" and
"what-is .. ?" messages.

All other messages are discarded.

[PA infrastructure protocol]: https://gitlab.com/personal-assistant-bot/infrastructure/protocol
