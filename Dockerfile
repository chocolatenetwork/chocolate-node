FROM rust:1.59.0

WORKDIR /node
ENV NODE_LOCATION="/node"
ENV PORT=9944

RUN  apt update
RUN  apt install -y cmake pkg-config libssl-dev git gcc build-essential git clang libclang-dev
RUN  rustup default stable 

RUN  rustup update nightly 
RUN  rustup target add wasm32-unknown-unknown --toolchain nightly

COPY . .
RUN cargo build --release


EXPOSE 9944

WORKDIR /node

CMD ["$NODE_LOCATION/target/release/chocolate","--dev","--ws-external","--tmp","--ws-port","$PORT"]