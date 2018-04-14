FROM rust:1.25.0

RUN apt-get update && \
  apt-get install -y \
  libudev-dev

WORKDIR /app
ADD . /app

CMD [ "cargo", "build", "--release" ]
