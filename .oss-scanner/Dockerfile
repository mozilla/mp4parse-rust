FROM rust:1-trixie

WORKDIR /src
COPY . /src

# The scanner clones without submodules. The AVIF tests need both fixture sets.
# cargo fetch also creates the workspace lockfile in a fresh checkout.
RUN git submodule update --init --recursive \
    && cargo fetch

# The scanner runs offline. Check both the default and Firefox feature sets.
ENV CARGO_NET_OFFLINE=true
RUN --network=none cargo build --workspace --locked \
    && cargo test --workspace --locked \
    && cargo test --workspace --locked --features missing-pixi-permitted
