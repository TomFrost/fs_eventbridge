#FS-EventBridge
A TCP server to stream file-change events to a remote destination, such as a docker VM with files shared over NFS.

## Building
For use on the local OS:

```
cargo build --release
```

For use in a boot2docker VM:

```
docker run --rm -it -v `pwd`:`pwd` -w `pwd` scorpil/rust cargo build --release
```

In both cases, find the built binary in `target/release/fs_eventbridge`.

## Executing
Copy the binary to the remote location, and run:

```
./fs_eventbridge
```

FS-EventBridge will launch and listen on port 65056 on all interfaces. Command line arguments can be specified to change the port and the bound IP address. Launch with `--help` for details.

## Using
Connect to the TCP server using your favorite language/library/client. Or use `telnet` (replace localhost with the IP of the machine running FS-EventBridge, if not local):

```
telnet localhost 65056
```

All commands are terminated by CR, LF, or any combination thereof. They can be streamed in any fashion; FS-EventBridge will service commands in the order in which they are received, as soon as a newline character is reached.

### Commands

#### `HELP`
Prints a help page of all commands

#### `CHANGE /path/to/file mtime_seconds`
Marks a remote file as changed (triggering the OS-specific filesystem event change notifications). The file's atime and mtime will be set to the current system time by default. Optionally, specifying mtime allows these times to be set to any arbitrary timestamp. It should be provided as Epoch seconds.

## Why?
Engineers using MacOS and Docker have wrestled with not having FSEvents file change notifications propagate to the docker VM's inotify for too long -- particularly those mounting shares via NFS. Hacks that try to fire `touch` commands off via SSH are slow and unreliable. FS-EventBridge was created to run on the root Boot2Docker VM, allowing other tooling (official client coming soon) to listen for changes on the host OS and forward them through the event bridge in real time, over a single low-overhead connection.

## License
FS-EventBridge is distributed under the ISC license. See LICENSE.txt for details.

## Credits
FS-EventBridge was created by Tom Shawver at TechnologyAdvice in 2016.
