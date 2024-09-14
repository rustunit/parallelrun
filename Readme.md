# parallelrun

Runs several commands concurrently.

Heavily inspired by the `nodejs` tool [concurrently](https://www.npmjs.com/package/concurrently). 

Supported Options:
* `--kill-others` (terminates all other commands as soon as one exits)

# TODO

- [ ] forward SIGINT to subprocesses instead of instant kill on `Ctrl+C`
- [ ] windows support
- [ ] support more arguments