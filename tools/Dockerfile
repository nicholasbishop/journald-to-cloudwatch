FROM ekidd/rust-musl-builder:stable

USER root
RUN apt-get update && apt-get install -y libsystemd-dev
USER rust

WORKDIR /home/rust/src
COPY Cargo.lock Cargo.toml ./
COPY src/ ./src/
COPY tools/ ./tools/

ENTRYPOINT tools/in_container_run.sh
