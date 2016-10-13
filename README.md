#FS-EventBridge [![Build Status](https://travis-ci.org/TechnologyAdvice/fs_eventbridge.svg?branch=master)](https://travis-ci.org/TechnologyAdvice/fs_eventbridge)
A TCP server to stream file-change events to a remote destination, such as a docker VM with files shared over NFS.

## Installing into a docker-machine on Mac
FS-EventBridge's biggest use case is running it inside of a boot2docker docker-machine image that's been configured to mount volumes over NFS. NFS provides superior read and write speed for volumes, but does not communicate the Mac's fsevents file change notifications to the Linux VM's inotify system.

Assuming you have [Homebrew](http://brew.sh) installed and a docker-machine already created and running, do the following to configure it for NFS (note, this assumes the docker-machine is named "default". Change this below as appropriate):

```
# Install and run docker-machine-nfs to configure the docker-machine to use NFS mounts
brew install docker-machine-nfs
docker-machine-nfs default
# Re-initialize env vars
eval $(docker-machine env default)
```

Once that's complete, or if you already had NFS set up, download and execute the FS-EventBridge installer! No need to clone this repo.

```
curl https://raw.githubusercontent.com/TechnologyAdvice/fs_eventbridge/master/scripts/boot2docker_install.sh > /tmp/boot2docker_install.sh
chmod +x /tmp/boot2docker_install.sh
/tmp/boot2docker_install.sh
```

Once this completes, you're all set. No need to read the _Building_ or _Executing_ sections below.

## Clients
- [fsbridge](https://github.com/TechnologyAdvice/fsbridge): A simple CLI client for watching a file or folder for changes and forwarding them over the bridge.
- [DevLab](https://github.com/TechnologyAdvice/DevLab): A docker-compose alternative for streamlined docker-based development. FS-EventBridge support is built in, just set `FS_EVENTBRIDGE_PORT=65056` in your environment variables.
- [fs-eventbridge-js](http://github.com/TechnologyAdvice/fs-eventbridge-js): A Node.js library that watches for file changes and streams them to the FS-EventBridge server. 

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
Engineers using MacOS and Docker have wrestled with not having FSEvents file change notifications propagate to the docker VM's inotify for too long -- particularly those mounting shares via NFS. Hacks that try to fire `touch` commands off via SSH are slow and unreliable. FS-EventBridge was created to run on the root Boot2Docker VM, allowing other tooling to listen for changes on the host OS and forward them through the event bridge in real time, over a single low-overhead connection.

## License
FS-EventBridge is distributed under the ISC license. See LICENSE.txt for details.

## Credits
FS-EventBridge was created by Tom Shawver at TechnologyAdvice in 2016.

Portions of the boot2docker install script were copied from [docker-machine-nfs](https://github.com/adlogix/docker-machine-nfs) by Toni Van de Voorde (MIT, 2015).
