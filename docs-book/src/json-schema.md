# `.autograder/autograder.json` Schema

| Field                 | Type   | Req | Description                                                                 |
| --------------------- | ------ | --- | --------------------------------------------------------------------------- |
| `meta.name`           | string | yes | Display name in the workflow and test filter                                |
| `meta.description`    | string | yes | Student-facing description (supports `##` placeholder for counts)           |
| `meta.points`         | number | yes | Max score for this test (default 1)                                         |
| `meta.timeout`        | number | yes | Seconds for the autograder step (default 10)                                |
| `type`                | string | yes | One of: `cargo_test`, `clippy`, `commit_count`, `test_count`                |
| `manifest_path`       | string | no  | Path to `Cargo.toml` (for `cargo_test`, `clippy`, `test_count`)             |
| `min_commits`         | number | no  | Required commits (only for `commit_count`)                                  |
| `min_tests`           | number | no  | Required tests (only for `test_count`)                                      |

## Example

```json
[
  {
     "meta": { "name": "test_func_1", "description": "a test function", "points": 1, "timeout": 10 },
     "type": "cargo_test",
     "manifest_path": "Cargo.toml"
  },
  {
    "meta": { "name": "COMMIT_COUNT_1", "description": "Ensure at least ## commits.", "points": 1, "timeout": 10 },
    "type": "commit_count",
    "min_commits": 5
  },
  {
    "meta": { "name": "TEST_COUNT", "description": "Ensure at least ## tests exist.", "points": 1, "timeout": 10 },
    "type": "test_count",
    "min_tests": 3
  }
]
```
