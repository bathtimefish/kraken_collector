#export KRKNC_MQTT_HOST=0.0.0.0:1883
export KRKNC_BROKER_HOST=http://127.0.0.1:50051
export KRKNC_WEBHOOK_PATH=webhook
export KRKNC_WEBHOOK_PORT=80
export KRKNC_MQTT_TOPIC=kraken
export KRKNC_MQTT_CONFIG_PATH=${PWD}/config/rumqttd.toml
export KRKNC_WEBSOCKET_HOST=0.0.0.0:2794
export KRKNC_WEBSOCKET_SUB_PROTOCOL=kraken-ws
export KRKNC_IBEACON_FILTER_DURATION_SEC=1
export KRKNC_IBEACON_ALLOWED_UUID_FILTER_PATH=${PWD}/config/allowed_uuids.yml
export KRKNC_SERIAL_DEVICE_NAME=brave_jig
export KRKNC_SERIAL_PORT=/dev/ttyACM0
export KRKNC_SERIAL_BAUDRATE=38400
export KRKNC_SERIAL_TIMEOUT_SEC=10