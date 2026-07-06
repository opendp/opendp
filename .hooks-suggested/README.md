# Suggested git hooks

Optional git hooks for rust fmt.

```
git config core.hooksPath .hooks-suggested
```

To stop using them:

```
git config --unset core.hooksPath
```

Hooks are added in .hooks-suggested/ to make them opt in.

## `pre-commit`

Runs `cargo fmt --all -- --check` on the opendp/rust/`

## Troubleshooting

If the hook finds issues, the commit will be staged. It can then be unstaged with
```
git restore --staged rust/src/...
```
