# Kraken Collector
Data Collection/Broker Application for IoT

![logo](./assets/kraken-logo-300.png)

# Introduction
Kraken Collector was developed as a data collection application for IoT. It can be used in combination with [Kraken Broker](https://github.com/bathtimefish/kraken_broker_python/).

Using Kraken Collector/Broker, you can receive data sent from edge IoT sensors via HTTP or MQTT in cloud or on-premise environments. This setup enables data-driven processing, such as data transformation, database storage, and user notifications, tailored to specific business needs.

If we compare what Kraken can achieve to existing services, it resembles a simplified combination of AWS IoT and Lambda. Kraken Collector/Broker allows for a compact, open-source implementation of these capabilities.

# Why Kraken?
Having worked on IoT systems for clients for many years, I have observed that while many projects are well-suited to the robust features offered by cloud services like AWS IoT and Azure IoT Hub, some are not.

Certain projects prioritize operational costs or control over scalability and stability, with requirements like "minimizing subscription costs," "distrusting cloud services," or "keeping all resources managed within the worksite." These needs are especially prevalent in certain industries where introducing sensing technology and data storage can be highly beneficial, but cloud solutions become overly complex and costly.

Kraken was developed to address these needs, allowing IoT systems to start small. Kraken Collector/Broker can implement IoT solutions, typically achievable through AWS IoT and Lambda, on on-premise, low-resource computers like Raspberry Pi.

The features of Kraken were selected from the most frequently used functions and experiences in IoT system development. The focus is on providing only the essential functions needed for quickly building IoT background systems rather than offering extensive features.

I hope Kraken can deliver its benefits to areas where IoT has yet to reach.

# Kraken Collector
[Kraken Collector](https://github.com/bathtimefish/kraken_collector) is an application for collecting data from edge IoT sensors and supports multiple communication protocols and data sources.

- HTTP Webhooks
- MQTT
- Websocket
- iBeacon (Bluetooth Low Energy)
- Serial Communication
- TextFile Monitoring
- Camera (USB Camera Capture)
- Email (SMTP Server)
- BraveJIG (IoT Edge Router)
- TCP Server

If your work requires other protocols, you can extend Kraken Collector by developing a new [collector](https://github.com/bathtimefish/kraken_collector/tree/main/src/collectors).

Kraken Collector is developed in Rust and operates as a lightweight/scalable gRPC client compatible with [Kraken Broker](https://github.com/bathtimefish/kraken_broker_python).

# Getting Started
This tutorial guides you through setting up Kraken Collector/Broker, starting them, and receiving your first data.

## Setup Broker
Clone the broker:
```bash
git clone https://github.com/bathtimefish/kraken_broker_python
cd kraken_broker_python
```

Set environment variables to launch the broker as a Slack broker:
```bash
export PYTHONDONTWRITEBYTECODE=1 export KRAKENB_DEBUG=1 export KRAKENB_GRPC_HOST=[::]:50051 export KRAKENB_SLACK_URL=[YOUR_SLACK_WEBHOOK_URL]
```

```bash
sudo apt update
sudo apt install -y protobuf-compiler libudev-dev libssl-dev libdbus-1-dev pkg-config
```

Start the broker:
```bash
python ./src/main.py
```

If you see the following log, the startup was successful:
```plaintext
INFO:root:gRPC server was started on `[::]:50051`
INFO:root:KRAKEN BROKER is running as debug mode.
```

## Setup Collector
Clone the repository with submodules:
```bash
git clone --recurse-submodules https://github.com/bathtimefish/kraken_collector
cd kraken_collector
```

If you've already cloned without submodules, initialize them:
```bash
git submodule update --init --recursive
```

Build the collector:
```bash
sudo apt install -y protobuf-compiler libudev-dev libssl-dev libdbus-1-dev pkg-config clang
cargo build
```

Set environment variables to launch the collector as a webhook receiver:
```bash
export KRKNC_BROKER_HOST=http://[::1]:50051 export KRKNC_WEBHOOK_PATH=webhook export KRKNC_WEBHOOK_PORT=3000
```

Start the collector:
```bash
RUST_LOG=error,main=debug cargo run --bin main
```

If you see the following log, the startup was successful:
```plaintext
[2024-01-01T00:00:00Z INFO  main] KRAKEN Collector -- The Highlevel Data Collector -- boot sequence start.
[2024-01-01T00:00:00Z DEBUG main::service] starting webhook collector service...
[2024-01-01T00:00:00Z DEBUG main::service] collector service started.
[2024-01-01T00:00:00Z DEBUG main::collectors::webhook] Webhook server was started and is listening on http://0.0.0.0:3000
```

## Send Data to Collector
Send data to the collector:
```bash
curl -X POST -H "Content-Type: application/json" -d '{"id":"101", "name":"env-sensor", "temp":"25.6", "hum":"52.4"}' http://localhost:3000/webhook
```

If you receive a message like the following on Slack, Kraken Collector/Broker is working correctly:
```plaintext
kind=collector, provider=webhook, payload={"id":"101", "name":"env-sensor", "temp":"25.6", "hum":"52.4"}
```

# Collector Settings
The functionality of the collector is configured through environment variables. Currently, the following environment variables are defined:

- `KRKNC_BROKER_HOST`
- `KRKNC_WEBHOOK_PATH`
- `KRKNC_WEBHOOK_PORT`
- `KRKNC_MQTT_HOST`
- `KRKNC_MQTT_TOPIC`
- `KRKNC_MQTT_CONFIG_PATH`
- `KRKNC_WEBSOCKET_HOST`
- `KRKNC_WEBSOCKET_SUB_PROTOCOL`
- `KRKNC_IBEACON_FILTER_DURATION_SEC`
- `KRKNC_IBEACON_ALLOWED_UUID_FILTER_PATH`
- `KRKNC_SERIAL_DEVICE_NAME`
- `KRKNC_SERIAL_PORT`
- `KRKNC_SERIAL_BAUDRATE`
- `KRKNC_SERIAL_TIMEOUT_SEC`
- `KRKNC_TEXTFILE_TARGET_FILE_PATH`
- `KRKNC_TEXTFILE_MONITOR_DIR_PATH`
- `KRKNC_TEXTFILE_GET_INTERVAL_SEC`
- `KRKNC_TEXTFILE_MONITORING_MODE`
- `KRKNC_TEXTFILE_ALLOW_CREATE`
- `KRKNC_TEXTFILE_ALLOW_MODIFY`
- `KRKNC_TEXTFILE_REMOVE_CREATED_FILE_AFTER_READ`
- `KRKNC_TEXTFILE_REMOVE_FILES_EXCEPT_MODIFIED_AFTER_READ`
- `KRKNC_TEXTFILE_REMOVE_ALL_FILES_AFTER_READ`
- `KRKNC_TEXTFILE_REMOVE_ALL_FOLDER_AFTER_READ`
- `KRKNC_CAMERA_CAPTURE_INTERVAL_SEC`
- `KRKNC_EMAIL_HOST_ADDR`
- `KRKNC_EMAIL_SMTP_PORT`
- `KRKNC_EMAIL_MAX_MESSAGE_SIZE`
- `KRKNC_EMAIL_MAX_ATTACHMENT_SIZE`
- `KRKNC_EMAIL_DOMAIN`
- `KRKNC_EMAIL_AUTH_REQUIRED`
- `KRKNC_EMAIL_ALLOWED_SENDERS`
- `KRKNC_EMAIL_TLS_ENABLED`
- `KRKNC_EMAIL_TLS_CERT_PATH`
- `KRKNC_EMAIL_TLS_KEY_PATH`
- `KRKNC_EMAIL_TLS_REQUIRE`
- `KRKNC_BJIG_DEVICE_PATH`
- `KRKNC_BJIG_CLI_BIN_PATH`
- `KRKNC_BJIG_DATA_TIMEOUT_SEC`
- `KRKNC_BJIG_ACTION_COOLDOWN_SEC`
- `KRKNC_TCP_HOST`
- `KRKNC_TCP_PORT`
- `KRKNC_TCP_BUFFER_SIZE`

## for Broker
### KRKNC_BROKER_HOST
Specify the broker’s URL. In most cases, the following setting should be sufficient:
```bash
KRKNC_BROKER_HOST=http://[::1]:50051
```

## Webhooks
The Webhook feature is enabled by setting `KRKNC_WEBHOOK_PATH` and `KRKNC_WEBHOOK_PORT`.
### KRKNC_WEBHOOK_PATH
Set the path for the webhook URL. For example, if `KRKNC_WEBHOOK_PATH=webhook`, the webhook URL will be `http://localhost/webhook`.
### KRKNC_WEBHOOK_PORT
Specify the port number for the webhook.

## MQTT
The MQTT Broker feature is enabled by setting `KRKNC_MQTT_HOST`, `KRKNC_MQTT_TOPIC`, and `KRKNC_MQTT_CONFIG_PATH`.
### KRKNC_MQTT_HOST
Specify the host address of the MQTT Broker. In most cases, the following setting should be sufficient:
```bash
KRKNC_MQTT_HOST=0.0.0.0:1883
```
### KRKNC_MQTT_TOPIC
Set the MQTT topic name.

### KRKNC_MQTT_CONFIG_PATH
The MQTT Broker functionality of the collector is based on [rumqttd](https://github.com/bytebeamio/rumqtt/tree/main/rumqttd). `KRKNC_MQTT_CONFIG_PATH` specifies the path to the custom configuration file for rumqttd.

## Websocket
The Websocket Server feature is enabled by setting `KRKNC_WEBSOCKET_HOST` and `KRKNC_WEBSOCKET_SUB_PROTOCOL`.
### KRKNC_WEBSOCKET_HOST
Specify the host address of the Websocket Server. In most cases, the following setting should be sufficient:
```bash
KRKNC_WEBSOCKET_HOST=0.0.0.0:2794
```
### KRKNC_WEBSOCKET_SUB_PROTOCOL
Specify the sub-protocol name for the Websocket Server.

## iBeacon
The iBeacon feature is enabled by setting `KRKNC_IBEACON_ALLOWED_UUID_FILTER_PATH`.
### KRKNC_IBEACON_FILTER_DURATION_SEC
Set the filter duration in seconds to prevent duplicate beacon detections.
### KRKNC_IBEACON_ALLOWED_UUID_FILTER_PATH
Specify the path to a YAML file containing allowed beacon UUIDs for filtering.

## Serial Communication
The Serial Communication feature is enabled by setting `KRKNC_SERIAL_DEVICE_NAME`.
### KRKNC_SERIAL_DEVICE_NAME
Set a descriptive name for the serial device.
### KRKNC_SERIAL_PORT
Specify the serial port path. In most cases, the following setting should be sufficient:
```bash
KRKNC_SERIAL_PORT=/dev/ttyACM0
```
### KRKNC_SERIAL_BAUDRATE
Set the baud rate for serial communication (default: 9600).
### KRKNC_SERIAL_TIMEOUT_SEC
Specify the timeout in seconds for serial read operations.

## TextFile Monitoring
The TextFile monitoring feature is enabled by setting `KRKNC_TEXTFILE_MONITOR_DIR_PATH`.

### KRKNC_TEXTFILE_TARGET_FILE_PATH
Specify the target file path to read (default: "data/data.txt").

### KRKNC_TEXTFILE_MONITOR_DIR_PATH
Specify the directory path to monitor for file changes (default: "data/").

### KRKNC_TEXTFILE_GET_INTERVAL_SEC
Set the interval in seconds for time-based monitoring (default: 10).

### KRKNC_TEXTFILE_MONITORING_MODE
Set the monitoring mode: "time_interval" or "event_driven" (default: "time_interval").

### KRKNC_TEXTFILE_ALLOW_CREATE
Enable monitoring of file creation events (default: true).

### KRKNC_TEXTFILE_ALLOW_MODIFY
Enable monitoring of file modification events (default: true).

### KRKNC_TEXTFILE_REMOVE_CREATED_FILE_AFTER_READ
Remove created files after reading (default: false).

### KRKNC_TEXTFILE_REMOVE_FILES_EXCEPT_MODIFIED_AFTER_READ
Remove all files except modified ones after reading (default: false).

### KRKNC_TEXTFILE_REMOVE_ALL_FILES_AFTER_READ
Remove all files after reading (default: false).

### KRKNC_TEXTFILE_REMOVE_ALL_FOLDER_AFTER_READ
Remove entire folder after reading (default: false).

## Camera
The Camera feature is enabled by setting `KRKNC_CAMERA_CAPTURE_INTERVAL_SEC`.
### KRKNC_CAMERA_CAPTURE_INTERVAL_SEC
Set the interval in seconds between camera snapshots. In most cases, the following setting should be sufficient:
```bash
KRKNC_CAMERA_CAPTURE_INTERVAL_SEC=5
```

## Email (SMTP Server)
The Email collector runs an embedded SMTP server that receives emails and forwards them to the broker. This feature is enabled by setting `KRKNC_EMAIL_HOST_ADDR` and `KRKNC_EMAIL_SMTP_PORT`.

The collector parses incoming emails and extracts:
- Sender IP address
- From/To/Cc/Bcc addresses
- Subject and body (text and HTML)
- Attachments (with Base64 encoding)
- Metadata (timestamp, message ID)

### KRKNC_EMAIL_HOST_ADDR
Specify the host address for the SMTP server to listen on (default: "0.0.0.0").
```bash
KRKNC_EMAIL_HOST_ADDR=0.0.0.0
```

### KRKNC_EMAIL_SMTP_PORT
Set the port number for the SMTP server (default: 587).
```bash
KRKNC_EMAIL_SMTP_PORT=587
```

### KRKNC_EMAIL_MAX_MESSAGE_SIZE
Set the maximum email message size in bytes (default: 10485760 = 10MB).
```bash
KRKNC_EMAIL_MAX_MESSAGE_SIZE=10485760
```

### KRKNC_EMAIL_MAX_ATTACHMENT_SIZE
Set the maximum attachment size in bytes (default: 5242880 = 5MB). Attachments exceeding this limit will be skipped.
```bash
KRKNC_EMAIL_MAX_ATTACHMENT_SIZE=5242880
```

### KRKNC_EMAIL_DOMAIN
Set the SMTP server domain name (default: "localhost"). Reserved for future implementation.
```bash
KRKNC_EMAIL_DOMAIN=localhost
```

### KRKNC_EMAIL_AUTH_REQUIRED
Enable SMTP authentication requirement (default: false). Reserved for future implementation.
```bash
KRKNC_EMAIL_AUTH_REQUIRED=false
```

### KRKNC_EMAIL_ALLOWED_SENDERS
Specify a comma-separated list of allowed sender email addresses or domains. If empty, all senders are allowed.
```bash
KRKNC_EMAIL_ALLOWED_SENDERS=trusted@example.com,admin@example.org
```

### KRKNC_EMAIL_TLS_ENABLED
Enable TLS/SSL encryption (default: false). Reserved for future implementation.
```bash
KRKNC_EMAIL_TLS_ENABLED=false
```

### KRKNC_EMAIL_TLS_CERT_PATH
Specify the path to the TLS certificate file. Reserved for future implementation.
```bash
KRKNC_EMAIL_TLS_CERT_PATH=/path/to/cert.pem
```

### KRKNC_EMAIL_TLS_KEY_PATH
Specify the path to the TLS private key file. Reserved for future implementation.
```bash
KRKNC_EMAIL_TLS_KEY_PATH=/path/to/key.pem
```

### KRKNC_EMAIL_TLS_REQUIRE
Require TLS for all connections (default: false). Reserved for future implementation.
```bash
KRKNC_EMAIL_TLS_REQUIRE=false
```

**Example Email Payload:**
```json
{
  "ipaddr": "192.168.1.100",
  "from": "sender@example.com",
  "to": ["recipient@example.com"],
  "subject": "Test Email",
  "body": "This is a test email.",
  "timestamp": "2024-01-01T12:00:00+00:00",
  "message_id": "<abc123@example.com>",
  "attachments": [
    {
      "name": "document.pdf",
      "mime_type": "application/pdf",
      "size": 1024,
      "data": "Base64EncodedData..."
    }
  ]
}
```

## BraveJIG (IoT Edge Router)
The BraveJIG collector provides integration with BraveJIG USB Router, a high-performance IoT edge router. This feature is automatically enabled when `KRKNC_BJIG_DEVICE_PATH` is set.

The collector monitors sensor data from the router and supports bidirectional communication with the broker for remote control capabilities.

**Note:** The BraveJIG collector requires the [bjig_controller](https://github.com/bathtimefish/bjig_controller) library, which is included as a git submodule. Make sure to clone the repository with `--recurse-submodules` or run `git submodule update --init --recursive` after cloning.

### KRKNC_BJIG_DEVICE_PATH
Specify the serial device path for the BraveJIG router. Setting this variable enables the BraveJIG collector (default: "/dev/ttyACM0").
```bash
KRKNC_BJIG_DEVICE_PATH=/dev/ttyACM0
```

### KRKNC_BJIG_CLI_BIN_PATH
Set the path to the BraveJIG CLI binary.
```bash
KRKNC_BJIG_CLI_BIN_PATH=[path_to_bravejig_cli]
```

### KRKNC_BJIG_DATA_TIMEOUT_SEC
Set the data timeout in seconds. If no data is received within this period, the router will be automatically restarted (default: 300).
```bash
KRKNC_BJIG_DATA_TIMEOUT_SEC=300
```

### KRKNC_BJIG_ACTION_COOLDOWN_SEC
Set the cooldown period in seconds between action commands to prevent duplicate processing (default: 30).
```bash
KRKNC_BJIG_ACTION_COOLDOWN_SEC=30
```

**Features:**
- Automatic router startup and initialization
- Real-time sensor data monitoring
- Bidirectional communication with broker
- Automatic router restart on data timeout
- Action command processing with pause/resume capability
- Debounce and cooldown protection for action commands

**Data Flow:**
1. Collector starts router and begins monitoring
2. Sensor data is forwarded to broker via gRPC
3. Broker can send action commands back to collector
4. Collector pauses monitoring, processes action, then resumes
5. Alert notifications sent on router restart events

**Example Sensor Data Payload:**
```json
{
  "sensor_id": "0121",
  "module_id": "2468800203400004",
  "temperature": 25.6,
  "humidity": 52.4,
  "timestamp": "2024-01-01T12:00:00+00:00"
}
```

## TCP Server
The TCP Server collector listens for incoming TCP connections and forwards received raw byte data to the broker. This feature is enabled by setting `KRKNC_TCP_HOST`.

The collector accepts multiple simultaneous TCP client connections and forwards each received data chunk to the broker as binary payload (`application/octet-stream`). The primary use case is receiving raw binary data from IoT sensors.

**Note:** TCP is a stream-oriented protocol and does not guarantee message boundaries. If a client sends data larger than the buffer size, it will be split into multiple gRPC messages. For small sensor data payloads that fit within the buffer size, this is not a concern in practice.

### KRKNC_TCP_HOST
Specify the host address for the TCP server to listen on. Setting this variable enables the TCP collector (default: "0.0.0.0").
```bash
KRKNC_TCP_HOST=0.0.0.0
```

### KRKNC_TCP_PORT
Set the port number for the TCP server (default: 9000).
```bash
KRKNC_TCP_PORT=9000
```

### KRKNC_TCP_BUFFER_SIZE
Set the read buffer size in bytes per connection (default: 4096). Increase this value if you expect to receive large data payloads in a single transmission.
```bash
KRKNC_TCP_BUFFER_SIZE=4096
```

**Metadata sent to broker:**
```json
{
  "peer_addr": "192.168.1.100:54321"
}
```

**Example usage with netcat:**
```bash
echo "sensor data" | nc 127.0.0.1 9000
printf '\x00\x01\x02\x03\xFF' | nc 127.0.0.1 9000
```
