# tm_chat_server
This program is a TCP-server meant to be interface-compatible with the lobby chat functionality in online PC RTS/MOBA [The Maestros](https://store.steampowered.com/app/553560/The_Maestros/).  It's currently 100% compatible with the client-facing interface.

Remaining work, loosely prioritized:
1. Any tests at all
2. Graceful handling (cleanup) of disconnected streams
3. replace some panics (expects) with stderr (more robust to bad client data)
4. Code Cleanliness pass
5. HTTP server to report service readiness (to support internal tooling)
6. Graylog2 compatible metrics (to support internal tooling)
