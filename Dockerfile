# Use Alpine 3.21 as the base image
FROM alpine:3.21

# Install dependencies
RUN apk add --no-cache \
    curl \
    build-base \
    gcc \
    git \
    make \
    musl-dev \
    rustup \
    python3 \
    openssl-dev \
    linux-headers  \
    pkgconfig \
    binutils

ENV PKG_CONFIG_PATH=/usr/lib/pkgconfig \
    OPENSSL_DIR=/usr \
    OPENSSL_LIB_DIR=/usr/lib \
    OPENSSL_INCLUDE_DIR=/usr/include

# Create a non-root user with UID 1000 and named 'rust'
RUN addgroup -g 1000 rust && \
    adduser -D -G rust -u 1000 rust

# Switch to the non-root user
USER rust

# Set environment variables to avoid interactive prompts and to configure Rust
ENV RUSTUP_HOME=/home/rust/rustup \
    CARGO_HOME=/home/rust/cargo \
    PATH=/home/rust/cargo/bin:$PATH

# Install Rust and Cargo
RUN rustup-init -y && \
    rustup update && \
    rustup default stable

# Set the working directory (optional, can be adjusted for your project)
WORKDIR /app

# Add a command for the container (can be modified based on usage)
CMD ["cargo", "--version"]
