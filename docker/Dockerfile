FROM rust:1.65 as builder

ARG RUSTRELEASEDIR="release"

WORKDIR /app

RUN mkdir -p tinyboards/media

COPY ./ ./

RUN cargo build --release

RUN strip ./target/$RUSTRELEASEDIR/tinyboards_server

RUN cp ./target/$RUSTRELEASEDIR/tinyboards_server /app/tinyboards_server

FROM debian:buster-slim as tinyboards

RUN apt-get update && apt-get install libpq-dev -y

RUN apt-get install ca-certificates -y

RUN update-ca-certificates

COPY --from=builder /app/tinyboards_server /app/tinyboards

COPY --from=builder /app/config/defaults.hjson /config/defaults.hjson

RUN sed -i 's/unset/localhost/g' /config/defaults.hjson

EXPOSE 8536

CMD ["/app/tinyboards"]