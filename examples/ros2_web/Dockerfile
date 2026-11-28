ARG ROS_BASE=ros:humble-ros-base-jammy
FROM node:22-bullseye-slim AS node
FROM rust:1.95.0-bullseye AS rust
FROM ${ROS_BASE}
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential curl ca-certificates unzip pkg-config \
    python3-venv python3-pip "ros-${ROS_DISTRO}-example-interfaces" \
    && rm -rf /var/lib/apt/lists/*
COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY --from=node /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s ../lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s ../lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx
COPY --from=rust /usr/local/rustup /usr/local/rustup
COPY --from=rust /usr/local/cargo/bin /usr/local/cargo/bin
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo
ENV PATH="/opt/venv/bin:/usr/local/cargo/bin:${PATH}"
# Python codegen uses GENCODE_PROTOC. Rust and TypeScript use CPP_PROTOC.
# The cpp binary is also `protoc` on PATH for diagnostics in test.sh.
COPY scripts/protobuf-versions.env /tmp/protobuf-versions.env
RUN arch="$(uname -m)" && case "$arch" in aarch64) arch=aarch_64;; x86_64) ;; *) exit 1;; esac \
    && ver() { grep -E "^${1}=" /tmp/protobuf-versions.env | tail -n 1 | cut -d= -f2- | tr -d "'\""; } \
    && gencode="$(ver GENCODE_PROTOC)" && cpp="$(ver CPP_PROTOC)" \
    && test -n "$gencode" && test -n "$cpp" \
    && install_protoc() { \
         version="$1"; dest="$2"; \
         curl --fail --location --connect-timeout 15 --retry 3 --retry-all-errors -o /tmp/protoc.zip \
           "https://github.com/protocolbuffers/protobuf/releases/download/v${version}/protoc-${version}-linux-${arch}.zip" \
         && mkdir -p "$dest" \
         && unzip -qo /tmp/protoc.zip -d "$dest" \
         && rm -f /tmp/protoc.zip \
         && "$dest/bin/protoc" --version; \
       } \
    && install_protoc "$gencode" /opt/protoc-gencode \
    && install_protoc "$cpp" /opt/protoc-cpp \
    && ln -sfn /opt/protoc-cpp/bin/protoc /usr/local/bin/protoc \
    && python3 -m venv --system-site-packages /opt/venv \
    && pip install 'maturin>=1.7,<2' 'protobuf>=5.29.3,<9'
WORKDIR /workspace
COPY bindings/typescript/package.json bindings/typescript/package-lock.json bindings/typescript/
RUN npm ci --prefix bindings/typescript
COPY examples/ros2_web/package.json examples/ros2_web/package-lock.json examples/ros2_web/
WORKDIR /workspace/examples/ros2_web
RUN npm ci && npx playwright install --with-deps chromium
WORKDIR /workspace
COPY . .
RUN PROTOC=/opt/protoc-gencode/bin/protoc python3 scripts/generate_python_msgs.py \
    && PROTOC=/opt/protoc-cpp/bin/protoc python3 scripts/generate_rust_msgs.py \
    && PROTOC=/opt/protoc-cpp/bin/protoc python3 scripts/generate_typescript_msgs.py \
    && npm run build:ts --prefix bindings/typescript
ENV VIRTUAL_ENV=/opt/venv
RUN cd bindings/python && maturin develop --release --no-default-features \
    --features extension-module,ws,console-api
WORKDIR /workspace/examples/ros2_web
RUN npm run build
ENV ROS_LOCALHOST_ONLY=1 ROS_DOMAIN_ID=73 RMW_IMPLEMENTATION=rmw_fastrtps_cpp
EXPOSE 8080 15560
# ros_entrypoint.sh sources the base image's ROS distribution.
CMD ["python3", "-u", "run.py", "--host", "0.0.0.0"]
