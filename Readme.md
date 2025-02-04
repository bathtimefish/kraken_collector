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
[Kraken Collector](https://github.com/bathtimefish/kraken_collector) is an application for collecting data from edge IoT sensors and supports three common protocols.

- HTTP Webhooks
- MQTT
- Websocket

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

sudo apt update
sudo apt install -y protobuf-compiler libudev-dev

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
Build the collector:
```bash
git clone https://github.com/bathtimefish/kraken_collector
cd kraken_collector
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
