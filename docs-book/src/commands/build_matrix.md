# `build` usage matrix

This matrix maps common scenarios to the flags you pass, what triggers resolve to,
and what the generated `on:` block looks like.

| Scenario                           | CLI flags                                                                                                 | Filters applied                      | Resulting `on:` block                           |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------ | ----------------------------------------------- |
| Instructor-only (manual)           | *(none)*                                                                                                  | —                                    | `on: [repository_dispatch]`                     |
| Instructor preset                  | `--preset instructor`                                                                                     | —                                    | `on: [repository_dispatch, workflow_dispatch]`  |
| Push only                          | `--triggers push`                                                                                         | —                                    | `on: [repository_dispatch, push]`               |
| Pull request only                  | `--triggers pull-request`                                                                                 | —                                    | `on: [repository_dispatch, pull_request]`       |
| Push + PR (student feedback)       | `--triggers push pull-request`                                                                            | —                                    | `on: [repository_dispatch, push, pull_request]` |
| Student push preset                | `--preset student-push`                                                                                   | —                                    | `on: [repository_dispatch, push]`               |
| Student PR preset                  | `--preset student-pr`                                                                                     | —                                    | `on: [repository_dispatch, pull_request]`       |
| Branch-limited (main only)         | `--triggers push pull-request --branches main`                                                            | `branches: [main]`                   | See [Example A](#example-a)                         |
| Branch-limited (main & feature)    | `--triggers push pull-request --branches main feature`                                                    | `branches: [main, feature]`          | See [Example B](#example-b)                               |
| Run only when certain files change | `--triggers push --paths src/** Cargo.toml`                                                               | `paths: [src/**, Cargo.toml]`        | See [Example C](#example-c)                               |
| Ignore doc-only changes            | `--triggers push --paths-ignore README.md docs/**`                                                        | `paths-ignore: [README.md, docs/**]` | See [Example D](#example-d)                             |
| Combine paths + branches           | `--triggers pull-request --branches main --paths src/**`                                                  | `branches: [main], paths: [src/**]`  | See [Example E](#example-e)                             |
| Everything together                | `--triggers push pull-request --branches main release --paths src/** Cargo.toml --paths-ignore README.md` | `branches, paths, paths-ignore`      | See [Example F](#example-f-putting-it-all-together)                               |

## Advanced Build YAML Examples

### Example A

```bash
autograder-setup build --triggers push pull-request --branches main
```

Runs the autograder on both pushes and pull requests, but only for the `main` branch.

```yaml
on:
  repository_dispatch:
  push:
    branches: [main]
  pull_request:
    branches: [main]
```

### Example B

```bash
autograder-setup build --triggers push pull-request --branches main feature
```

Same as Example A, but allow grading on multiple branches — here, both `main` and `feature`.

```yaml
on:
  repository_dispatch:
  push:
    branches: [main, feature]
  pull_request:
    branches: [main, feature]
```

### Example C

```bash
autograder-setup build --triggers push --paths src/** Cargo.toml
```

Run the autograder only when commits modify source files or the manifest (`Cargo.toml`).

```yaml
on:
  repository_dispatch:
  push:
    paths: [src/**, Cargo.toml]
```

### Example D

```bash
autograder-setup build --triggers push --paths-ignore README.md docs/**
```

Ignore pushes that only change documentation or the README.

```yaml
on:
  repository_dispatch:
  push:
    paths-ignore: [README.md, docs/**]
```

### Example E

```bash
autograder-setup build --triggers pull-request --branches main --paths src/**`
```

Run the autograder for pull requests into `main`, but only when code files are modified.

```yaml
on:
  repository_dispatch:
  pull_request:
    branches: [main]
    paths: [src/**]
```

### Example F: Putting It All Together

Using the command

```bash
autograder-setup build --triggers push pull-request \
    --branches main release --paths src/** Cargo.toml \
    --paths-ignore README.md
```

We can combine all filters for fine-grained control: grade only on code changes to `main` or `release` branches, ignoring documentation-only updates.

```yaml
on:
  repository_dispatch:
  push:
    branches: [main, release]
    paths: [src/**, Cargo.toml]
    paths-ignore: [README.md]
  pull_request:
    branches: [main, release]
    paths: [src/**, Cargo.toml]
    paths-ignore: [README.md]
```

All options can be further tweaked in the YAML for additional refinement, such as *grading different branches for pushes and pull requests*

> Do these calls feel verbose? Feel free to open an issue for commonly used build patterns to improve our *presets* options!
