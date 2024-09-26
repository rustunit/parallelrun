# parallelrun

[![crates.io][sh_crates]][lk_crates]
[![ci][sh_ci]][lk_ci]
[![discord][sh_discord]][lk_discord]

[sh_crates]: https://img.shields.io/crates/v/parallelrun.svg
[lk_crates]: https://crates.io/crates/parallelrun
[sh_ci]: https://github.com/rustunit/parallelrun/workflows/ci/badge.svg
[lk_ci]: https://github.com/rustunit/parallelrun/actions
[sh_discord]: https://img.shields.io/discord/1176858176897953872?label=discord&color=5561E6
[lk_discord]: https://discord.gg/rQNeEnMhus

Runs several commands concurrently.

Heavily inspired by the `nodejs` tool [concurrently](https://www.npmjs.com/package/concurrently). 

Supported and tested on Linux, MacOS and Windows.

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
- [ ] support more `concurrently` arguments