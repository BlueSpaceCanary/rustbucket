FROM rust:1.31
MAINTAINER bluespacecanary
WORKDIR /usr/src/rustbucket
COPY . .
RUN cargo install --path .
CMD ["rustbucket"]