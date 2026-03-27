# csd

Fast search-and-replace across files using regex. Walks directories in parallel, respects `.gitignore`, and shows diffs interactively.

## Install

```bash
cargo install --path .
```

## Usage

```
csd [FILE_PATTERN] SEARCH_PATTERN REPLACEMENT
```

With three arguments, the first is a regex matched against file paths. With two, all files are searched.

```bash
# Replace "foo" with "bar" in all files
csd 'foo' 'bar'

# Only in .txt files
csd '\.txt$' 'foo' 'bar'

# Piped file list
find . -name '*.rs' | csd 'old_func' 'new_func'

# Preview changes without writing
csd --dry-run 'foo' 'bar'

# Confirm each change interactively
csd -i 'foo' 'bar'
```

## Options

| Flag | Description |
|---|---|
| `-i`, `--interactive` | Prompt before each change with a diff |
| `-q`, `--quiet` | Suppress output except errors |
| `-I`, `--no-ignore` | Don't respect `.gitignore`/`.ignore` |
| `--include-git-dir` | Include `.git` directory contents |
| `--dry-run` | Show changes without modifying files |

Backreferences work with both `\1` and `$1` syntax.

## License

[Apache-2.0](./LICENSE.md)

## Authors

### Chmouel Boudjnah

- Fediverse - <[@chmouel@chmouel.com](https://fosstodon.org/@chmouel)>
- Twitter - <[@chmouel](https://twitter.com/chmouel)>
- Blog  - <[https://blog.chmouel.com](https://blog.chmouel.com)>
