FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev tzdata
WORKDIR /build
COPY . .
RUN cargo install --path . --root ./dist

FROM scratch
ENV TZ=UTC
COPY --from=builder /usr/share/zoneinfo /usr/share/zoneinfo
COPY --from=builder /build/dist/bin/docker-maid /usr/local/bin/docker-maid
ENTRYPOINT ["/usr/local/bin/docker-maid"]