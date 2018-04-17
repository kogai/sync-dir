FROM rust:1.25.0

RUN apt-get update && \
  apt-get install -y \
  libusb-1.0-0-dev

WORKDIR /app

ADD . /app

CMD [ "cargo", "build", "--release" ]
