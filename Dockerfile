FROM alpine:3.20 AS runtime

ARG BINARY_PATH=docker-build/cqlsh-rs-binary

RUN apk add --no-cache ca-certificates tzdata

COPY ${BINARY_PATH} /usr/local/bin/cqlsh-rs

RUN chmod +x /usr/local/bin/cqlsh-rs

# Run as non-root
RUN adduser -D -u 1000 cqlsh
USER cqlsh

ENTRYPOINT ["cqlsh-rs"]
