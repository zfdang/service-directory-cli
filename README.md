# service-directory-cli

Rust CLI for Kite **Service Directory**.

The repo is `service-directory-cli`; the installed user-facing binary is
**`kitedir`**.

The CLI is a thin client over the HTTP API exposed by
[`service-directory-backend`](https://github.com/zfdang/service-directory-backend).
It depends on the published `service-directory-client` Rust crate and never
talks to Postgres or imports private backend crates by path.

## Command shape (planned)

```text
kitedir providers search
kitedir providers get <provider-id>
kitedir descriptors validate <path-or-url>
kitedir providers submit <descriptor-path-or-url>
kitedir comments add <provider-id>
kitedir ratings add <provider-id>
kitedir evaluations add <provider-id>
kitedir auth device-flow start
kitedir auth device-flow poll <device-code>
kitedir moderation list
kitedir moderation hide <target-type> <target-id>
kitedir admin managers invite <email-or-account>
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
