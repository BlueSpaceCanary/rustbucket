FROM rust:1.36
MAINTAINER bluespacecanary
WORKDIR /usr/src/rustbucket
COPY . .
RUN cargo install --path .
CMD ["rustbucket"]