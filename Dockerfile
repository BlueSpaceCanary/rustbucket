# Based on http://whitfin.io/speeding-up-rust-docker-builds/
# **************************** BUILD PHASE **************************
FROM ekidd/rust-musl-builder as BUILD
MAINTAINER bluespacecanary

# create a new empty shell project
RUN USER=root cargo new --bin rustbucket
WORKDIR /home/rust/src/rustbucket

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
# Optimization to make it less annoying to tinker with cargo build
RUN cargo fetch
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN cargo build --release

# **************************** RUN PHASE **************************

FROM alpine:latest

# copy the build artifact from the build stage
COPY --from=build /home/rust/src/rustbucket/target/x86_64-unknown-linux-musl/release/rustbucket .

# set the startup command to run your binary
ENTRYPOINT ["./rustbucket"]
