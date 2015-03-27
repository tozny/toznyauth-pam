# Demo for Tozny PAM module
#
# This script produces a Docker image that runs an SSH server.  You can log into
# the server to see how Tozny authentication works in practice.
#
# Build the Docker image, place this file in an empty directory and run this
# command:
#
#     $ docker build tozny-pam-demo .
#
# To log into the server, you will need a Tozny account in some realm.  You will
# need the realm id and user id for your account.  You will also need to have
# the Tozny mobile app ready to go.
#
# To run the demo:
#
#     $ CONTAINER=$(docker run -e "REALM_ID=sid_74a40187e2790" -e "USER_ID=sid_c233df00c07b9" -P -d tozny-pam-demo)
#
# The container will include a running SSH server, whose port is forwarded to
# some port in your host OS.  To log in:
#
#     $ ssh -p $(docker port $CONTAINER 22 | cut -d: -f2) gregory@localhost
#
# To stop the server:
#
#     $ docker stop $CONTAINER
#

FROM ubuntu:14.04
MAINTAINER Jesse Hallett <jesse@galois.com>

ENV DEBIAN_FRONTEND noninteractive

## Install dependencies

RUN apt-get update && \
    apt-get install -y openssh-server build-essential python make curl git g++ valgrind libssl-dev libpam-dev && \
    apt-get clean

## Build the PAM module

RUN mkdir /src && \
    curl https://static.rust-lang.org/dist/2015-03-03/rust-nightly-x86_64-unknown-linux-gnu.tar.gz > /src/rust.tar.gz && \
    cd /src && \
    tar -xzf rust.tar.gz && \
    cd rust-nightly-x86_64-unknown-linux-gnu && \
    ./install.sh && \
    rm -rf /src

RUN git clone https://github.com/tozny/toznyauth-pam.git /usr/local/src/toznyauth-pam
WORKDIR /usr/local/src/toznyauth-pam
RUN cargo build

## Install and configure the module

RUN mkdir -p /lib/security
RUN cp target/libtoznyauth_pam*.so /lib/security/toznyauth_pam.so
RUN sed -i "1s/^/auth sufficient toznyauth_pam.so -p\n\n/" /etc/pam.d/sshd
RUN sed -i 's/ChallengeResponseAuthentication no/ChallengeResponseAuthentication yes/' /etc/ssh/sshd_config

## Create a regular user account

RUN adduser --quiet --gecos '' --disabled-password gregory
RUN adduser gregory sudo
RUN mkdir -p ~gregory/.config/tozny

## Run OpenSSH server

RUN mkdir -p /var/run/sshd && chmod 0755 /var/run/sshd

EXPOSE 22
CMD echo "realm_key_id = \"$REALM_ID\"\nauthorized_users = [ \"$USER_ID\" ]" > ~gregory/.config/tozny/authorized.toml && \
    /usr/sbin/sshd -D
