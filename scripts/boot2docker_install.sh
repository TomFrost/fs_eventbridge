#!/bin/sh
###############################################################
#
# Copyright (c) 2016 TechnologyAdvice, LLC
# See LICENSE.txt for software license details.
#
# Portions taken from the fantastic docker-machine-nfs,
# which is Copyright 2015 Toni Van de Voorde (MIT). Find it
# here: https://github.com/adlogix/docker-machine-nfs
#
###############################################################

set -o errexit

INSTALL_DIR=/var/lib/docker/bin
START_CMD="sudo start-stop-daemon -S -b -x $INSTALL_DIR/fs_eventbridge"
START_SCRIPT=/var/lib/boot2docker/bootlocal.sh

hello() {
  cat <<EOF

                    ##         .
              ## ## ##        ==            _____ ____        _____ ____
           ## ## ## ## ##    ===           |  ___/ ___|      | ____| __ )
       /"""""""""""""""""\___/ ===         | |_  \___ \ _____|  _| |  _ \\
  ~~~ {~~ ~~~~ ~~~ ~~~~ ~~~ ~ /  ===- ~~~  |  _|  ___) |_____| |___| |_) |
       \______ o           __/             |_|   |____/      |_____|____/
         \    \         __/
          \____\_______/


This script will install FS-EventBridge into the currently active docker-machine,
starting it on port 65056 and setting it to restart every time the VM boots. The
host operating system, where you're running this command right now, will not be
written to or changed in any way. This script will never ask for root access.

EOF
  if [ "$1" != "--noprompt" ]; then
    read -p "Press Enter to proceed, or CTRL+C to cancel."
  fi
}

echoError() {
  echo "\033[0;31mFAIL\n\n$1 \033[0m"
}

echoWarn() {
  echo "\033[0;33m$1 \033[0m"
}

echoSuccess() {
  echo "\033[0;32m$1 \033[0m"
}

echoInfo() {
  printf "\033[1;34m[INFO] \033[0m$1"
}

checkDockerMachine() {
  echoInfo "docker-machine exists ... \t\t"

  if type docker-machine >/dev/null 2>&1; then
    echoSuccess "OK"
  else
    echoError "docker-machine was not found on the PATH"
    exit 1
  fi
}

checkDockerMachineName() {
  echoInfo "docker-machine env vars set ... \t\t"

  if [ -z "$DOCKER_MACHINE_NAME" ]; then
    echoError "Not found. Run: eval \$(docker-machine env MACHINE_NAME)"
    exit 1
  fi

  echoSuccess "OK"
}

checkMachineRunning() {
  echoInfo "machine running ... \t\t\t"

  machine_state=$(docker-machine ls | sed 1d | grep "^$1\s" | awk '{print $4}')

  if [ "Running" != "${machine_state}" ]; then
    echoError "The machine '$1' is not running but '${machine_state}'!"
    exit 1
  fi

  echoSuccess "OK"
}

installEventBridge() {
  echoInfo "Compiling and copying FS-EventBridge into $DOCKER_MACHINE_NAME VM"
  echo #EMPTY LINE
  docker run --rm -it \
    -v $INSTALL_DIR:/root/.cargo/bin -w /tmp \
    -e PATH=/usr/bin:/usr/local/bin:/root/.cargo/bin \
    scorpil/rust cargo install fs_eventbridge --force
  res=$?
  if [ "$res" -ne "0" ]; then
    echoError "Install failed. Please post the above log to https://github.com/TechnologyAdvice/fs_eventbridge/issues"
    exit 2
  fi
  echoSuccess "fs_eventbridge successfully copied to $INSTALL_DIR"
}

installAutoStart() {
  echoInfo "Installing FS-EventBridge ... \t\t"

  docker-machine ssh $DOCKER_MACHINE_NAME \
    "echo -e '\n$START_CMD\n' | sudo tee -a $START_SCRIPT && sudo chmod +x $START_SCRIPT && sync" \
    > /dev/null
  sleep 2

  echoSuccess "OK"
}

startServer() {
  echoInfo "Starting FS-EventBridge ... \t\t"

  docker-machine ssh $DOCKER_MACHINE_NAME "$START_CMD; echo" > /dev/null
  sleep 1

  echoSuccess "OK"
}

donezo() {
  echo "\033[0;36m"
  echo "--------------------------------------------"
  echo
  echo " The docker-machine '$DOCKER_MACHINE_NAME'"
  echo " is now running FS-EventBridge!"
  echo
  echo " ENJOY realtime file change notifications!"
  echo
  echo "--------------------------------------------"
  echo "\033[0m"
}

hello $1
checkDockerMachine
checkDockerMachineName
checkMachineRunning $DOCKER_MACHINE_NAME
installEventBridge
installAutoStart
startServer
donezo
