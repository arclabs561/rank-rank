# mgrep for Cursor

## Quick Start

```bash
# Authenticate (one time)
npx -y @mixedbread/mgrep login

# Index your repo (run in project directory)
npx -y @mixedbread/mgrep watch

# Search
npx -y @mixedbread/mgrep "your query"
```

## Future MCP Config

When mgrep MCP server is available, add to `~/.cursor/mcp.json`:

```json
"mgrep": {
  "command": "npx",
  "args": ["-y", "@mixedbread/mgrep-mcp"],
  "env": {
    "MXBAI_API_KEY": "your_key_here"
  }
}
```

## Alias (Optional)

Add to `~/.zshrc`: `alias mgrep='npx -y @mixedbread/mgrep'`


