# Based on http://whitfin.io/speeding-up-rust-docker-builds/
# **************************** BUILD PHASE **************************
FROM rust:1.36 as build
MAINTAINER bluespacecanary

# create a new empty shell project
RUN pwd
RUN USER=root cargo new --bin rustbucket
RUN ls
WORKDIR /rustbucket

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/rustbucket*
RUN cargo build --release

# **************************** RUN PHASE **************************

FROM alpine

# copy the build artifact from the build stage
COPY --from=build /rustbucket/target/release/rustbucket .

# set the startup command to run your binary
CMD ["./rustbucket"]
