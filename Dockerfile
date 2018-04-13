FROM rust:1.25.0

COPY Cargo.toml /app/Cargo.toml
WORKDIR /app

RUN ls -l && mkdir src && touch src/main.rs cargo install

ADD . /app

CMD [ "cargo", "build", "--release" ]
