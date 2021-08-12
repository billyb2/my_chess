FROM gitpod/workspace-full-vnc
RUN sudo apt update && sudo apt upgrade -y && sudo apt install clang lld cmake libssl-dev build-essential pkg-config libx11-dev libasound2-dev libudev-dev mesa-vulkan-drivers firefox libxi-dev libgl1-mesa-dev -y
RUN cargo install -f cargo-make && cargo install -f basic-http-server
RUN rustup install nightly && rustup install stable && rustup component add clippy
RUN rustup target install wasm32-unknown-unknown
RUN rustup component add rust-src
