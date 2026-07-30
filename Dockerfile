FROM alpine:3.23

ARG TARGETARCH

# `less` is the interactive pager for query results. Alpine only provides a
# BusyBox applet by default, which lacks GNU flags (-P) and inverts -R, so the
# real GNU less is installed explicitly.
RUN apk add --no-cache ca-certificates tzdata less

COPY docker-build/cqlsh-rs-${TARGETARCH} /usr/local/bin/cqlsh-rs

RUN chmod +x /usr/local/bin/cqlsh-rs

# Run as non-root
RUN adduser -D -u 1000 cqlsh
USER cqlsh

ENTRYPOINT ["cqlsh-rs"]
