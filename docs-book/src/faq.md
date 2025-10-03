# FAQ

**Does it run on every push?**  
Only if you use `--grade-on-push`. The default trigger is `repository_dispatch` to keep CI usage low.

**How do I show a grading table in the README?**  
Run `autograder-setup table` (copy to clipboard) or `autograder-setup table --to-readme`.

**Can I award partial credit for commit frequency?**  
Yes – use multiple thresholds with `--require-commits`, e.g., `5 10 20`.

**How do I require students to write additional tests?**  
Use `--require-tests N`. The autograder expects `N` + the number of tests generated from `cargo test` entries.
