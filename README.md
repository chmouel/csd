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

# Backreferences
#
# Swap adjacent words (capture group $1 and $2)
csd '(\w+) (\w+)' '$2 $1'
# "hello world" → "world hello"

# Same as above using \1 syntax
csd '(\w+) (\w+)' '\2 \1'
# "hello world" → "world hello"

# Rename function parameters (single capture group)
csd 'function\((\w+)\)' 'func($1)'
# "function(x)" → "func(x)"

# Add prefix to variable names
csd '(const|let|var) (\w+)' '$1 my_$2'
# "const name" → "const my_name"
```

## Options

| Flag | Description |
|---|---|
| `-i`, `--interactive` | Prompt before each change with a diff |
| `-q`, `--quiet` | Suppress output except errors |
| `-I`, `--no-ignore` | Don't respect `.gitignore`/`.ignore` |
| `--include-git-dir` | Include `.git` directory contents |
| `--dry-run` | Show changes without modifying files |

## License

[Apache-2.0](./LICENSE.md)

## Authors

### Chmouel Boudjnah

- Fediverse - <[@chmouel@chmouel.com](https://fosstodon.org/@chmouel)>
- Twitter - <[@chmouel](https://twitter.com/chmouel)>
- Blog  - <[https://blog.chmouel.com](https://blog.chmouel.com)>
