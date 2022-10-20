# Build the project
FROM clux/muslrust:1.59.0 as builder

ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
ARG RUSTRELEASEDIR="release"

WORKDIR /app

COPY ./ ./

RUN cargo build --release

# reduce binary size
RUN strip ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/porpl_server

RUN cp ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/porpl_server /app/porpl_server

# The alpine runner
FROM alpine:3 as porpl

# Install libpq for postgres
RUN apk add libpq

# Copy resources
COPY --from=builder /app/porpl_server /app/porpl

EXPOSE 8536
CMD ["/app/porpl"]
