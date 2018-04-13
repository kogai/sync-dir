FROM rust:1.25.0

COPY Cargo.toml /app/Cargo.toml
WORKDIR /app

RUN apt-get update && \
  apt-get install -y libudev-dev

RUN mkdir src && \
  touch src/main.rs && \
  cargo install

ADD . /app

CMD [ "cargo", "build", "--release" ]
