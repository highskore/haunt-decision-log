# haunt-decision-log

Haunt's decision log as a first-party plugin (`dev.haunt.decision-log`). It adds the ⌘K `log` mode: type `log ` in the palette and search every verified permission request Haunt recorded, newest first.

Each row shows:

- the outcome: "You approved", "You denied", "<plugin> approved", "Allowed in terminal:", "Superseded:"
- the tool and its preview
- the tab (the last part of the agent's folder), and the agent if it isn't Claude Code
- the time

Enter jumps to the tab the request came from, if it is still open.

The search matches the tool, preview, folder, outcome (`allowed_in_haunt`, `denied_in_haunt`, …) or tab id, case-insensitively. Haunt's core keeps the records (`~/.haunt/decisions.jsonl`, the last 10,000 in memory). This plugin only reads them.

## Capabilities

| capability | why |
|---|---|
| `decisions_read` | `decisions.query`, read only. The decisions include commands and file paths, which can contain secrets. |

It asks for nothing else: no events, no files, no pty, no notifications.

## Build, test, install

```sh
cargo test                                     # native unit tests
cargo build --release --target wasm32-wasip2   # or ./build.sh
haunt plugin install .
```

Or from a haunt.app built with it (`macos/make-app.sh --plugins decision-log`): `haunt setup --plugins decision-log`, or check it in the setup sheet or Settings. Haunt never installs it on its own.

It needs Haunt with `haunt:plugin@0.4` (`api = "0.4"`). `wit/haunt-plugin.wit` is a copy of Haunt's, pinned at that version.

## Licence

MIT, see `LICENSE`.
