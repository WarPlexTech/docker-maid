FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY . .
RUN cargo install --path . --root ./dist

FROM alpine:latest
COPY --from=builder /build/dist/bin/docker-maid /usr/local/bin/docker-maid
ENTRYPOINT ["/usr/local/bin/docker-maid"]