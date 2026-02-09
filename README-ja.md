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
[Kraken Collector](https://github.com/bathtimefish/kraken_collector)はエッジIoTセンサからデータを収集するためのアプリケーションで、複数の通信プロトコルとデータソースをサポートしています。

- HTTP Webhooks
- MQTT
- Websocket
- iBeacon (Bluetooth Low Energy)
- Serial Communication（シリアル通信）
- TextFile Monitoring（テキストファイル監視）
- Camera（USBカメラキャプチャ）
- Email（SMTPサーバー）
- BraveJIG（IoTエッジルーター）

もしあなたの仕事に他のプロトコルが必要な場合、新しい[collector](https://github.com/bathtimefish/kraken_collector/tree/main/src/collectors)を開発することでKraken Collectorを拡張することができます。

Kraken CollectorはRustで開発されており、[Kraken Broker](https://github.com/bathtimefish/kraken_broker_python)に対応するgRPCクライアントとして軽量/スケーラブルに動作します。

# Getting started
ここでは最初のチュートリアルとして、Kraken Collector/Broker をセットアップして起動し、最初のデータを受信してみます。

## Setup Broker
Brokerをcloneします
```bash
git clone https://github.com/bathtimefish/kraken_broker_python
cd kraken_broker_python
```

BrokerをSlackブローカーとして起動するための環境変数を設定します
```bash
export PYTHONDONTWRITEBYTECODE=1 export KRAKENB_DEBUG=1 export KRAKENB_GRPC_HOST=[::]:50051 export KRAKENB_SLACK_URL=[YOUR_SLACK_WEBHOOK_URL]
```

```bash
sudo apt update
sudo apt install -y protobuf-compiler libudev-dev libssl-dev libdbus-1-dev pkg-config
```

Brokerを起動します
```bash
python ./src/main.py
```

以下のようなログが表示されると起動が成功しています
```plaintext
INFO:root:gRPC server was started on `[::]:50051`
INFO:root:KRAKEN BROKER is running as debug mode.
```

## Setup Collector
submoduleを含めてリポジトリをクローンします
```bash
git clone --recurse-submodules https://github.com/bathtimefish/kraken_collector
cd kraken_collector
```

すでにsubmoduleなしでクローンしている場合は、submoduleを初期化します
```bash
git submodule update --init --recursive
```

Collectorをビルドします
```bash
sudo apt install -y protobuf-compiler libudev-dev libssl-dev libdbus-1-dev pkg-config clang
cargo build
```

CollectorをWebhookレシーバとして起動するための環境変数を設定します
```bash
export KRKNC_BROKER_HOST=http://[::1]:50051 export KRKNC_WEBHOOK_PATH=webhook export KRKNC_WEBHOOK_PORT=3000
```

Collectorを起動します
```bash
RUST_LOG=error,main=debug cargo run --bin main
```

以下のようなログが表示されると起動が成功しています
```plaintext
[2024-01-01T00:00:00Z INFO  main] KRAKEN Collector -- The Highlevel Data Collector -- boot sequence start.
[2024-01-01T00:00:00Z DEBUG main::service] starting webhook collector service...
[2024-01-01T00:00:00Z DEBUG main::service] collector service started.
[2024-01-01T00:00:00Z DEBUG main::collectors::webhook] Webhook server was started and is listening on http://0.0.0.0:3000
```

## Send data to Collector
Collectorにデータを送信します
```bash
curl -X POST -H "Content-Type: application/json" -d '{"id":"101", "name":"env-sensor", "temp":"25.6", "hum":"52.4"}' http://localhost:3000/webhook
```

Slackで以下のようなメッセージが受信できたなら、Kraken Collector/Brokerは正常に動作しています
```plaintext
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

## for Broker
### KRKNC_BROKER_HOST
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

## iBeacon
iBeacon機能は `KRKNC_IBEACON_ALLOWED_UUID_FILTER_PATH` を設定することで利用可能となります。
### KRKNC_IBEACON_FILTER_DURATION_SEC
重複したビーコン検出を防ぐためのフィルタ期間を秒単位で設定します。
### KRKNC_IBEACON_ALLOWED_UUID_FILTER_PATH
フィルタリング用の許可されたビーコンUUIDを含むYAMLファイルのパスを指定します。

## シリアル通信
シリアル通信機能は `KRKNC_SERIAL_DEVICE_NAME` を設定することで利用可能となります。
### KRKNC_SERIAL_DEVICE_NAME
シリアルデバイスの説明的な名前を設定します。
### KRKNC_SERIAL_PORT
シリアルポートのパスを指定します。多くの場合、次のような設定で良いはずです。
```bash
KRKNC_SERIAL_PORT=/dev/ttyACM0
```
### KRKNC_SERIAL_BAUDRATE
シリアル通信のボーレートを設定します（デフォルト: 9600）。
### KRKNC_SERIAL_TIMEOUT_SEC
シリアル読み取り操作のタイムアウトを秒単位で指定します。

## テキストファイル監視
テキストファイル監視機能は `KRKNC_TEXTFILE_MONITOR_DIR_PATH` を設定することで利用可能となります。

### KRKNC_TEXTFILE_TARGET_FILE_PATH
読み取り対象のファイルパスを指定します（デフォルト: "data/data.txt"）。

### KRKNC_TEXTFILE_MONITOR_DIR_PATH
ファイル変更を監視するディレクトリパスを指定します（デフォルト: "data/"）。

### KRKNC_TEXTFILE_GET_INTERVAL_SEC
時間ベースの監視の間隔を秒単位で設定します（デフォルト: 10）。

### KRKNC_TEXTFILE_MONITORING_MODE
監視モードを設定します: "time_interval" または "event_driven"（デフォルト: "time_interval"）。

### KRKNC_TEXTFILE_ALLOW_CREATE
ファイル作成イベントの監視を有効にします（デフォルト: true）。

### KRKNC_TEXTFILE_ALLOW_MODIFY
ファイル更新イベントの監視を有効にします（デフォルト: true）。

### KRKNC_TEXTFILE_REMOVE_CREATED_FILE_AFTER_READ
作成されたファイルを読み取り後に削除します（デフォルト: false）。

### KRKNC_TEXTFILE_REMOVE_FILES_EXCEPT_MODIFIED_AFTER_READ
読み取り後に変更されたファイル以外をすべて削除します（デフォルト: false）。

### KRKNC_TEXTFILE_REMOVE_ALL_FILES_AFTER_READ
読み取り後にすべてのファイルを削除します（デフォルト: false）。

### KRKNC_TEXTFILE_REMOVE_ALL_FOLDER_AFTER_READ
読み取り後にフォルダ全体を削除します（デフォルト: false）。

## Camera
Camera機能は `KRKNC_CAMERA_CAPTURE_INTERVAL_SEC` を設定することで利用可能となります。
### KRKNC_CAMERA_CAPTURE_INTERVAL_SEC
カメラスナップショットの間隔を秒単位で設定します。多くの場合、次のような設定で良いはずです。
```bash
KRKNC_CAMERA_CAPTURE_INTERVAL_SEC=5
```

## Email (SMTP Server)
Emailコレクタは組み込みSMTPサーバーを実行し、メールを受信してブローカーに転送します。この機能は `KRKNC_EMAIL_HOST_ADDR` と `KRKNC_EMAIL_SMTP_PORT` を設定することで利用可能となります。

コレクタは受信メールを解析し、以下の情報を抽出します:
- 送信者IPアドレス
- From/To/Cc/Bccアドレス
- 件名と本文（テキストおよびHTML）
- 添付ファイル（Base64エンコーディング）
- メタデータ（タイムスタンプ、メッセージID）

### KRKNC_EMAIL_HOST_ADDR
SMTPサーバーがリッスンするホストアドレスを指定します（デフォルト: "0.0.0.0"）。
```bash
KRKNC_EMAIL_HOST_ADDR=0.0.0.0
```

### KRKNC_EMAIL_SMTP_PORT
SMTPサーバーのポート番号を設定します（デフォルト: 587）。
```bash
KRKNC_EMAIL_SMTP_PORT=587
```

### KRKNC_EMAIL_MAX_MESSAGE_SIZE
メールメッセージの最大サイズをバイト単位で設定します（デフォルト: 10485760 = 10MB）。
```bash
KRKNC_EMAIL_MAX_MESSAGE_SIZE=10485760
```

### KRKNC_EMAIL_MAX_ATTACHMENT_SIZE
添付ファイルの最大サイズをバイト単位で設定します（デフォルト: 5242880 = 5MB）。この制限を超える添付ファイルはスキップされます。
```bash
KRKNC_EMAIL_MAX_ATTACHMENT_SIZE=5242880
```

### KRKNC_EMAIL_DOMAIN
SMTPサーバーのドメイン名を設定します（デフォルト: "localhost"）。将来の実装のために予約されています。
```bash
KRKNC_EMAIL_DOMAIN=localhost
```

### KRKNC_EMAIL_AUTH_REQUIRED
SMTP認証要件を有効にします（デフォルト: false）。将来の実装のために予約されています。
```bash
KRKNC_EMAIL_AUTH_REQUIRED=false
```

### KRKNC_EMAIL_ALLOWED_SENDERS
許可された送信者のメールアドレスまたはドメインをカンマ区切りで指定します。空の場合、すべての送信者が許可されます。
```bash
KRKNC_EMAIL_ALLOWED_SENDERS=trusted@example.com,admin@example.org
```

### KRKNC_EMAIL_TLS_ENABLED
TLS/SSL暗号化を有効にします（デフォルト: false）。将来の実装のために予約されています。
```bash
KRKNC_EMAIL_TLS_ENABLED=false
```

### KRKNC_EMAIL_TLS_CERT_PATH
TLS証明書ファイルのパスを指定します。将来の実装のために予約されています。
```bash
KRKNC_EMAIL_TLS_CERT_PATH=/path/to/cert.pem
```

### KRKNC_EMAIL_TLS_KEY_PATH
TLS秘密鍵ファイルのパスを指定します。将来の実装のために予約されています。
```bash
KRKNC_EMAIL_TLS_KEY_PATH=/path/to/key.pem
```

### KRKNC_EMAIL_TLS_REQUIRE
すべての接続でTLSを必須にします（デフォルト: false）。将来の実装のために予約されています。
```bash
KRKNC_EMAIL_TLS_REQUIRE=false
```

**メールペイロードの例:**
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

## BraveJIG（IoTエッジルーター）
BraveJIGコレクタは、高性能IoTエッジルーターであるBraveJIG USBルーターとの統合を提供します。この機能は `KRKNC_BJIG_DEVICE_PATH` を設定することで自動的に有効になります。

コレクタはルーターからのセンサーデータを監視し、リモートコントロール機能のためにブローカーとの双方向通信をサポートします。

**注意:** BraveJIGコレクタには[bjig_controller](https://github.com/bathtimefish/bjig_controller)ライブラリが必要です。これはgit submoduleとして含まれています。リポジトリをクローンする際は `--recurse-submodules` オプションを使用するか、クローン後に `git submodule update --init --recursive` を実行してください。

### KRKNC_BJIG_DEVICE_PATH
BraveJIGルーターのシリアルデバイスパスを指定します。この変数を設定するとBraveJIGコレクタが有効になります（デフォルト: "/dev/ttyACM0"）。
```bash
KRKNC_BJIG_DEVICE_PATH=/dev/ttyACM0
```

### KRKNC_BJIG_CLI_BIN_PATH
BraveJIG CLIバイナリのパスを設定します。
```bash
KRKNC_BJIG_CLI_BIN_PATH=[BraveJIG CLIのパス]
```

### KRKNC_BJIG_DATA_TIMEOUT_SEC
データタイムアウトを秒単位で設定します。この期間内にデータを受信しない場合、ルーターは自動的に再起動されます（デフォルト: 300）。
```bash
KRKNC_BJIG_DATA_TIMEOUT_SEC=300
```

### KRKNC_BJIG_ACTION_COOLDOWN_SEC
重複処理を防ぐためのアクションコマンド間のクールダウン期間を秒単位で設定します（デフォルト: 30）。
```bash
KRKNC_BJIG_ACTION_COOLDOWN_SEC=30
```

**機能:**
- ルーターの自動起動と初期化
- リアルタイムセンサーデータ監視
- ブローカーとの双方向通信
- データタイムアウト時の自動ルーター再起動
- pause/resume機能を使ったアクションコマンド処理
- アクションコマンドのデバウンスとクールダウン保護

**データフロー:**
1. コレクタがルーターを起動し、監視を開始
2. センサーデータはgRPC経由でブローカーに転送
3. ブローカーはコレクタにアクションコマンドを送信可能
4. コレクタは監視を一時停止し、アクションを処理してから再開
5. ルーター再起動イベント時にアラート通知を送信

**センサーデータペイロードの例:**
```json
{
  "sensor_id": "0121",
  "module_id": "2468800203400004",
  "temperature": 25.6,
  "humidity": 52.4,
  "timestamp": "2024-01-01T12:00:00+00:00"
}
```
