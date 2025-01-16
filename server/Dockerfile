FROM rust:1.75-slim as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
# Install Python 3.13 and its dependencies
RUN apt-get update && apt-get install -y \
    wget \
    build-essential \
    libssl-dev \
    zlib1g-dev \
    libffi-dev \
    && rm -rf /var/lib/apt/lists/*

# Download and install Python 3.13
RUN wget https://www.python.org/ftp/python/3.13.0/Python-3.13.0a4.tgz \
    && tar xzf Python-3.13.0a4.tgz \
    && cd Python-3.13.0a4 \
    && ./configure --enable-optimizations \
    && make -j $(nproc) \
    && make install \
    && cd .. \
    && rm -rf Python-3.13.0a4* \
    && ln -s /usr/local/bin/python3.13 /usr/local/bin/python3 \
    && ln -s /usr/local/bin/python3.13 /usr/local/bin/python

# Copy the Rust binary from builder
COPY --from=builder /usr/src/app/target/release/portico /usr/local/bin/
CMD ["portico"]