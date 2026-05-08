# service-directory-cli

Rust CLI for Kite **Service Directory**.

The repo is `service-directory-cli`; the installed user-facing binary is
**`kitedir`**.

The CLI is a thin client over the HTTP API exposed by
[`service-directory-backend`](https://github.com/zfdang/service-directory-backend).
It depends on the published `service-directory-client` Rust crate and never
talks to Postgres or imports private backend crates by path.

## Current command surface

```text
kitedir version
kitedir providers search
kitedir providers get <provider-id>
kitedir providers submit <provider-payload.json>
kitedir providers endpoints <provider-id>
kitedir providers payments <provider-id>
kitedir descriptors validate <descriptor.json>
kitedir descriptors get <provider-id>
kitedir comments add <provider-id>
kitedir comments list <provider-id>
kitedir ratings add <provider-id>
kitedir ratings list <provider-id>
kitedir evaluations add <provider-id>
kitedir auth login --email <email>
kitedir auth verify --token <token>
kitedir auth whoami
kitedir auth logout
kitedir auth stepup
kitedir auth device-flow start
kitedir auth device-flow poll <device-code>
kitedir auth device-flow approve <user-code>
kitedir moderation recent-hides
kitedir moderation action --target-type <type> --target-id <id> --action <action>
kitedir admin submissions
kitedir admin review <submission-id>
kitedir admin audit
kitedir admin ranking-weights
kitedir admin managers invite --email <email>
kitedir admin managers list
kitedir admin managers revoke <invitation-id>
kitedir admin force-verify <provider-id> --reason <text>
kitedir me sessions
kitedir me revoke-session <session-id>
kitedir me tokens
kitedir me revoke-token <token-id>
kitedir completions {bash|zsh|fish}
```

## Configuration

- tokens live in `~/.config/kite/directory/credentials.toml` (mode `0600`)
- profile selection: `--profile <name>` or `KITEDIR_PROFILE`
- precedence: `CLI flag > environment variable > config file`

## Sibling repos

- [service-directory-backend](https://github.com/zfdang/service-directory-backend) — API + workers
- [service-directory-web](https://github.com/zfdang/service-directory-web) — React + Tailwind WebUI
- [service-directory-mcp](https://github.com/zfdang/service-directory-mcp) — Rust MCP server (`kitedir-mcp`)
- [service-directory-deploy](https://github.com/zfdang/service-directory-deploy) — Kubernetes / k3s manifests
