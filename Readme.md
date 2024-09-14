# parallelrun

Runs several commands concurrently.

Heavily inspired by the `nodejs` tool [concurrently](https://www.npmjs.com/package/concurrently). 

Supported Options:
* `--kill-others` (terminates all other commands as soon as one exits)

# Example

```sh
$ parallelrun --kill-others "echo wait 2 && sleep 2" "echo wait 3 && sleep 3"
[0] wait 2
[1] wait 3
[0] echo wait 2 && sleep 2 exited with code 0
--> Sending SIGTERM to other processes..
[1] echo wait 3 && sleep 3 exited with code SIGTERM
```

# TODO

- [ ] forward SIGINT to subprocesses instead of instant kill on `Ctrl+C`
- [ ] windows support
- [ ] support more arguments