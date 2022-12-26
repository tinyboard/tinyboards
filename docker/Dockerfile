FROM rust:1.64 as builder

ARG RUSTRELEASEDIR="release"

WORKDIR /app

COPY ./ ./

RUN cargo build --release

RUN strip ./target/$RUSTRELEASEDIR/tinyboards_server

RUN cp ./target/$RUSTRELEASEDIR/tinyboards_server /app/tinyboards_server

FROM debian:buster-slim as tinyboards

RUN apt-get update && apt-get install libpq-dev -y

COPY --from=builder /app/tinyboards_server /app/tinyboards

COPY --from=builder /app/config/defaults.hjson /config/defaults.hjson

COPY --from=builder /app/assets/ /assets/

RUN sed -i 's/unset/localhost/g' /config/defaults.hjson

EXPOSE 8536

CMD ["/app/tinyboards"]