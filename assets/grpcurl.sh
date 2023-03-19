#/bin/sh

grpcurl -plaintext -import-path ./proto -proto kraken.proto -d '{"kind": "collector", "provider": "mqtt", "payload": "hello kraken"}' '[::1]:50051' kraken.KrakenMessage/Send
