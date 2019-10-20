# tm_chat_server
This program is a TCP-server meant to be interface-compatible with the lobby chat functionality in online PC RTS/MOBA [The Maestros](https://store.steampowered.com/app/553560/The_Maestros/).  It's currently 100% compatible with the client-facing interface.

Remaining work:
- Truly non-blocking IO (remove awful wait on reads)
- replace some panics (expects) with stderr (more robust to bad client data)
- general code clean up
- HTTP server to report service readiness (to support internal tooling)
- Graylog2 compatible metrics (to support internal tooling)
