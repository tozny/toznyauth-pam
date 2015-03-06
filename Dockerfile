FROM ubuntu:14.04

ENV DEBIAN_FRONTEND noninteractive

RUN apt-get update && \
    apt-get install -y build-essential python make curl git g++ valgrind libssl-dev libpam-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN mkdir /src && \
    curl https://static.rust-lang.org/dist/2015-03-03/rust-nightly-x86_64-unknown-linux-gnu.tar.gz > /src/rust.tar.gz && \
    cd /src && \
    tar -xzf rust.tar.gz && \
    cd rust-nightly-x86_64-unknown-linux-gnu && \
    ./install.sh && \
    rm -rf /src

RUN mkdir /code
ADD . /code

WORKDIR /code

RUN cargo build
