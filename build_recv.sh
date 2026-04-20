#!/bin/sh

APP_NAME=${APP_NAME:=retherm}
APP_ARGS="$@"

stop_app() {
   APP_PID=$(pidof ${APP_NAME})
   if [ "${APP_PID}" != "" ]; then
      echo "Stop process ${APP_PID}"
      kill ${APP_PID}
   fi
}

trap stop_app SIGINT

start_app_bg() {
   ./${APP_NAME} ${APP_ARGS} &
   echo "Launched ${APP_NAME} ${APP_ARGS} with PID ${!}"
}

echo "Waiting for ${APP_NAME} data"
if nc -l -p 51234 > /tmp/${APP_NAME}; then
   md5sum /tmp/${APP_NAME}

   stop_app
   cp /tmp/${APP_NAME} .
   chmod +x ${APP_NAME}

   start_app_bg

   APP_NAME=${APP_NAME} ./$0 ${APP_ARGS}
fi
