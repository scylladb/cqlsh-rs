FROM alpine:3.23

ARG TARGETARCH

RUN apk add --no-cache ca-certificates tzdata

COPY docker-build/cqlsh-rs-${TARGETARCH} /usr/local/bin/cqlsh-rs

RUN chmod +x /usr/local/bin/cqlsh-rs

# Run as non-root
RUN adduser -D -u 1000 cqlsh
USER cqlsh

ENTRYPOINT ["cqlsh-rs"]
