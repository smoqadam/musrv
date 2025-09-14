FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
ARG MUSRV_BIN=musrv
COPY ${MUSRV_BIN} /usr/local/bin/musrv
ENTRYPOINT ["musrv"]
CMD ["--help"]
