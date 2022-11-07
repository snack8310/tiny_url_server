# stage 1 generate a recipe for dependencies
FROM rust:1.63 as planner
ADD ./.cargo $CARGO_HOME/
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# stage 2 build our dependencies
FROM rust:1.63 as cacher
ADD ./.cargo $CARGO_HOME/
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# stage 3 use the main official rust docker image
FROM rust:1.63 as builder
ADD ./.cargo $CARGO_HOME/
COPY . /app
WORKDIR /app

# COPY dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /user/local/cargo

RUN cargo build --release

# FROM dockerhub.test.wacai.info/xamc_ext/ubuntu:20.04
# RUN apt update
# RUN apt install libmariadb3 libmariadb-dev -y
FROM registry.cn-shanghai.aliyuncs.com/snack_on_monday/ubuntu_ssl:20.04
# FROM dockerhub.test.wacai.info/xamc_ext/cc-debian11 

COPY --from=builder /app/target/release/tiny_url_server /app/tiny_url_server
COPY config /app/config

ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000

WORKDIR /app

CMD ["./tiny_url_server"]

# docker run -p 8000:8000 tiny_url_server    