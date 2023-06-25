FROM rust:1.69-buster as build-env

ARG GITHUB_USER
ARG GITHUB_TOKEN
ARG GITHUB_REPO
ARG PROJECT_NAME

WORKDIR /
RUN apt update
RUN apt install -y pkg-config libssl-dev protobuf-compiler
RUN touch /root/.netrc \
    && echo machine github.com >> /root/.netrc \
    && echo login ${GITHUB_USER} >> /root/.netrc \
    && echo password ${GITHUB_TOKEN} >> /root/.netrc \
    && git clone ${GITHUB_REPO} \
    && rm /root/.netrc \
    && cd ./${PROJECT_NAME}/ \
    && cargo build --release \
    && cp ./target/release/main /root/collector \
    && cp ./config/mqttd.conf /root/mqttd.conf \
    && chmod +x /root/collector

FROM gcr.io/distroless/cc-debian11:nonroot
WORKDIR /
COPY --from=build-env /root/collector .
COPY --from=build-env /root/mqttd.conf .
ENV KRKNC_BROKER_HOST http:127.0.0.1:50055
ENV KRKNC_WEBHOOK_PATH webhook
ENV KRKNC_WEBHOOK_PORT 80
ENV KRKNC_MQTT_HOST 0.0.0.0:1883
ENV KRKNC_MQTT_TOPIC kraken
ENV KRKNC_MQTT_CONFIG_PATH /mqttd.conf
ENV KRKNC_WEBSOCKET_HOST 127.0.0.1:2794
ENV KRKNC_WEBSOCKET_SUB_PROTOCOL kraken-ws
ENV RUST_LOG error,main=debug
EXPOSE 80 1883 2794
ENTRYPOINT ["./collector"]
