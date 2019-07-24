FROM amazonlinux:2

RUN yum install -y gcc openssl-devel pkgconfig systemd-devel

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

WORKDIR /build
COPY Cargo.lock Cargo.toml ./
COPY src/ ./src/
COPY tools/ ./tools/

ENTRYPOINT tools/in_container_run.sh
