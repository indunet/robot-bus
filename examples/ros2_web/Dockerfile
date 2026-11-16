FROM node:22-bullseye-slim AS node
FROM ros:humble-ros-base-jammy
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential curl ca-certificates unzip pkg-config libzmq3-dev \
    python3-venv python3-pip ros-humble-example-interfaces \
    && rm -rf /var/lib/apt/lists/*
COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY --from=node /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s ../lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s ../lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx
ARG RUST_VERSION=1.95.0
RUN curl --fail --silent --show-error https://sh.rustup.rs -o /tmp/rustup.sh \
    && sh /tmp/rustup.sh -y --profile minimal --default-toolchain "$RUST_VERSION"
ENV PATH="/opt/venv/bin:/root/.cargo/bin:${PATH}"
RUN arch="$(uname -m)" && case "$arch" in aarch64) arch=aarch_64;; x86_64) ;; *) exit 1;; esac \
    && curl --fail --location --retry 3 -o /tmp/protoc.zip \
    "https://github.com/protocolbuffers/protobuf/releases/download/v35.1/protoc-35.1-linux-${arch}.zip" \
    && unzip /tmp/protoc.zip -d /usr/local \
    && python3 -m venv --system-site-packages /opt/venv \
    && pip install 'maturin>=1.7,<2' 'protobuf>=7.35,<8'
WORKDIR /workspace
COPY bindings/typescript/package.json bindings/typescript/package-lock.json bindings/typescript/
RUN npm ci --prefix bindings/typescript
COPY examples/ros2_web/package.json examples/ros2_web/package-lock.json examples/ros2_web/
WORKDIR /workspace/examples/ros2_web
RUN npm ci && npx playwright install --with-deps chromium
WORKDIR /workspace
COPY . .
RUN python3 scripts/generate_python_msgs.py \
    && python3 scripts/generate_rust_msgs.py \
    && python3 scripts/generate_typescript_msgs.py \
    && npm run build:ts --prefix bindings/typescript
ENV VIRTUAL_ENV=/opt/venv
RUN cd bindings/python && maturin develop --release --no-default-features \
    --features extension-module,ws,console-api
WORKDIR /workspace/examples/ros2_web
RUN npm run build
ENV ROS_LOCALHOST_ONLY=1 ROS_DOMAIN_ID=73 RMW_IMPLEMENTATION=rmw_fastrtps_cpp
EXPOSE 8080 15560
# ros_entrypoint.sh sources Humble before starting the Python application.
CMD ["python3", "-u", "run.py", "--host", "0.0.0.0"]
