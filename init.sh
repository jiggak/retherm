#!/bin/sh

# Source common library
. /etc/init.d/functions

NAME=retherm
APP_DIR=/retherm
APP_PATH="${APP_DIR}/${NAME}"

start_retherm() {
   ${STARTDAEMON} -b -x ${APP_PATH} -- --config ${APP_DIR}/config.toml --syslog INFO
}

stop_retherm() {
   ${STOPDAEMON} -x ${APP_PATH}
}

case "${1}" in
'start')
   echo_and_log "ReTherm:\c"
   start_retherm
   echo_and_log "."
   ;;
'stop')
   stop_retherm
   ;;
'restart')
   ${0} stop
   ${0} start
   ;;
*)
   echo "Usage: ${0} <start | stop | restart>"
   exit 1
   ;;
esac
