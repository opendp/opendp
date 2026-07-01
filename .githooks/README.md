# Suggested git hooks

Optional git hooks for rust fmt.

```
git config core.hooksPath .githooks
```

To stop using them:

```
git config --unset core.hooksPath
```

Hooks are added in .githooks/ to make them opt in.

## `pre-commit`

Runs `cargo fmt --all -- --check` on the opendp/rust/
Fix with `cargo fmt --manifest-path rust/Cargo.toml --all`.

## Troubleshooting

If the hook finds issues, the commit will be staged. It can then be unstaged with
```
git restore --staged rust/src/...
```

Git can only run the pre-commit hook if the file allows for execution.
The following command allows the file to be used as an executable.

```
chmod +x .githooks/pre-commit
```
