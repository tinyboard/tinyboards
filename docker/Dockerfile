FROM rust:1.64 as builder

ARG RUSTRELEASEDIR="release"

WORKDIR /app

COPY ./ ./

RUN cargo build --release

RUN strip ./target/$RUSTRELEASEDIR/porpl_server

RUN cp ./target/$RUSTRELEASEDIR/porpl_server /app/porpl_server

FROM debian:buster-slim as porpl

RUN apt-get update && apt-get install libpq-dev -y

COPY --from=builder /app/porpl_server /app/porpl

COPY --from=builder /app/config/defaults.hjson /config/defaults.hjson

RUN sed -i 's/unset/localhost/g' /config/defaults.hjson

EXPOSE 8536

CMD ["/app/porpl"]