# Kraken Collector
IoTのためのデータ収集/ブローカー アプリケーション

![logo](./assets/kraken-logo-300.png)

# Introduction
Kraken CollectorはIoT向けのデータ収集アプリケーションとして開発されました。[Kraken Broker](https://github.com/bathtimefish/kraken_broker_python/)と組み合わせて利用することができます。

Kraken Collector/Brokerを利用するとエッジIoTセンサからHTTPやMQTT経由で送信されるデータをクラウドおよびオンプレミスな環境で受け取り、データの加工、データベースへの格納、ユーザーへの通知等の業務に応じた処理をデータドリブンで実現することができます。

Krakenでできることをよく使われている既存のサービスに例えるならば、AWS IoTとLambdaのシンプルな組み合わせに似ています。Kraken Collector/Brokerはそれをオープンソースでコンパクトに実装できます。

# Why Kraken?
私は長年クライアントのニーズに応じたIoTシステムの開発に携わってきました。その多くはAWS IoTやAzure IoT Hubなどのクラウドサービスが持つ豊富な機能によって開発可能なものでしたが、いくつかのプロジェクトにはそれらクラウドサービスがフィットしないものがありました。

"サブスクリプションコストを最小化したい"、"クラウドサービスが未だに信用できない"、"すべてのリソースを作業現場内で管理したい"など、クラウドサービスが持つスケーラビリティや安定性よりも運用コストや専有性を優先するニーズは特定の業種では未だに多くあります。

そして、そのような業種ほどセンシングテクノロジーやデータ蓄積を導入するメリットが大きい場合がありますが、クラウドサービスを利用した時点でそれらのニーズを叶えることができません。IoTをスタートするためにはセンサー、クラウド、アプリケーションなどに複数のコストと準備期間を費やす必要があり、特にクラウドの仕組みが大掛かりで too muchとなるケースが多くあります。

Krakenはこの問題を解決し、小さくIoTをスタートするために開発されました。Kraken Collector/BrokerはAWS IoTやLamdaを使って実現できるIoTの仕組みをRaspberry Piのような低リソースなコンピュータ上でオンプレミスに小さく動作させることができます。

Krakenの各機能は、私がIoTシステムを開発してきた中で利用してきた機能や体験の中でよく使ったもののみをコンパクトに実装しました。多機能性よりもIoTバックグランドシステムに必要最低限の機能を迅速に構築できることにフォーカスしています。

今までIoTが行き届いていなかった業務に対して、Krakenがそのメリットを届けられることを期待しています。

# Kraken Collector
[Kraken Collector](https://github.com/bathtimefish/kraken_collector)はエッジIoTセンサからデータを収集するためのアプリケーションで、IoTセンサが一般的によく利用する3つのプロトコルをサポートしています。

HTTP Webhooks
MQTT
Websocket

もしあなたの仕事に他のプロトコルが必要な場合、新しい[collector](https://github.com/bathtimefish/kraken_collector/tree/main/src/collectors)を開発することでKraken Collectorを拡張することができます。

Kraken CollectorはRustで開発されており、[Kraken Broker](https://github.com/bathtimefish/kraken_broker_python)に対応するgRPCクライアントとして軽量/スケーラブルに動作します。

# Getting started
ここでは最初のチュートリアルとして、Kraken Collector/Broker をセットアップして起動し、最初のデータを受信してみます。

## Setup Broker
Brokerをcloneします
```bash
git clone https://github.com/bathtimefish/kraken_broker_python
kraken_broker_python
```

BlockerをSlackブローカーとして起動するための環境変数を設定します
```bash
export PYTHONDONTWRITEBYTECODE=1 \
export KRAKENB_DEBUG=1 \
export KRAKENB_GRPC_HOST=[::]:50051 \
export KRAKENB_SLACK_URL=[YOUR_SLACK_WEBHOOK_URL]
```

```bash
sudo apt update
sudo apt install -y protobuf-compiler libudev-dev libssl-dev libdbus-1-dev pkg-config
```

Blokerを起動します
```bash
python ./src/main.py
```

以下のようなログが表示されると起動が成功しています
```bash
INFO:root:gRPC server was started on `[::]:50051`
INFO:root:KRAKEN BROKER is running as debug mode.
```

## Setup Collector
Collectorをビルドします
```bash
sudo apt install -y protobuf-compiler libudev-dev libssl-dev libdbus-1-dev pkg-config
git clone https://github.com/bathtimefish/kraken_collector
cd kraken_collector
cargo build
```

CollectorをWebhookレシーバとして起動するための環境変数を設定します
```bash
export KRKNC_BROKER_HOST=http://[::1]:50051 \
exoprt KRKNC_WEBHOOK_PATH=webhook \
export KRKNC_WEBHOOK_PORT=3000
```

Collectorを起動します
```bash
RUST_LOG=error,main=debug cargo run --bin main
```

以下のようなログが表示されると起動が成功しています
```bash
[2024-01-01T00:00:00Z INFO  main] KRAKEN Collector -- The Highlevel Data Collector -- boot squence start.
[2024-01-01T00:00:00Z DEBUG main::service] starting webhook collector service...
[2024-01-01T00:00:00Z DEBUG main::service] collector service started.
[2024-01-01T00:00:00Z DEBUG main::collectors::webhook] Webhook server was started that is listening on http://0.0.0.0:3000
```

## Send data to Collector
Collectorにデータを送信します
```bash
curl -X POST -H "Content-Type: application/json" -d '{"id":"101", "name":"env-sensor", "temp":"25.6", "hum":"52.4"}' http://localhost:3000/webhook
```

Slackで以下のようなメッセージが受信できたなら、Kraken Collector/Brokerは正常に動作しています
```bash
kind=collector, provider=webhook, payload={"id":"101", "name":"env-sensor", "temp":"25.6", "hum":"52.4"}
```

# Collector settings
Collectorの機能は環境変数で設定します。現在以下の環境変数が定義されています

- `KRKNC_BROKER_HOST`
- `KRKNC_WEBHOOK_PATH`
- `KRKNC_WEBHOOK_PORT`
- `KRKNC_MQTT_HOST`
- `KRKNC_MQTT_TOPIC`
- `KRKNC_MQTT_CONFIG_PATH`
- `KRKNC_WEBSOCKET_HOST`
- `KRKNC_WEBSOCKET_SUB_PROTOCOL`

## for Broker
## KRKNC_BROKER_HOST
BrokerのURLを指定します。多くの場合次のような設定で良いはずです。

```bash
KRKNC_BROKER_HOST=http://[::1]:50051
```
## Webhooks
Webhook機能は `KRKNC_WEBHOOK_PATH` `KRKNC_WEBHOOK_PORT`を設定することで利用可能となります。
### KRKNC_WEBHOOK_PATH
Webhook URLのパスを設定します。`KRKNC_WEBHOOK_PATH=webhook` の場合、 `http://localhost/webhook` がWebhook URLとなります。
### KRKNC_WEBHOOK_PORT

Webhookのポート番号を設定します。
## MQTT
MQTT Broker機能は `KRKNC_MQTT_HOST` `KRKNC_MQTT_TOPIC` `KRKNC_MQTT_CONFIG_PATH` を設定することで利用可能となります。
### KRKNC_MQTT_HOST
MQTT Brokerのホストアドレスを指定します。多くの場合、次のような設定で良いはずです。

```bash
KRKNC_MQTT_HOST=0.0.0.0:1883
```
### KRKNC_MQTT_TOPIC
MQTTトピック名を設定します。

### KRKNC_MQTT_CONFIG_PATH
CollectorのMQTT Broker機能は[rumqttd](https://github.com/bytebeamio/rumqtt/tree/main/rumqttd)をベースにしています。`KRKNC_MQTT_CONFIG_PATH`はrumqttdのカスタムコンフィグファイルのパスを指定します。
## Websocket
Websocket Server機能は `KRKNC_WEBSOCKET_HOST` `KRKNC_WEBSOCKET_SUB_PROTOCOL` を設定することで利用可能となります。
### KRKNC_WEBSOCKET_HOST
Websocket Serverのホストアドレスを指定します。多くの場合、次のような設定で良いはずです。

```bash
KRKNC_WEBSOCKET_HOST=0.0.0.0:2794
```
### KRKNC_WEBSOCKET_SUB_PROTOCOL
Websocket Serverのサブプロトコル名を指定します。
