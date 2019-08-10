# Based on http://whitfin.io/speeding-up-rust-docker-builds/
# **************************** BUILD PHASE **************************
FROM rust:1.36 as build
MAINTAINER bluespacecanary

# create a new empty shell project
RUN USER=root cargo new --bin rustbucket
WORKDIR /rustbucket

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

run rustup target add x86_64-unknown-linux-musl
run apt-get update
run apt-get install -y musl-tools musl-dev libssl-dev

# this build step will cache your dependencies
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/rustbucket*
RUN cargo build --release --target x86_64-unknown-linux-musl

# **************************** RUN PHASE **************************

FROM alpine

# copy the build artifact from the build stage
COPY --from=build /rustbucket/target/release/rustbucket .

# set the startup command to run your binary
ENTRYPOINT ["./rustbucket"]
