FROM ubuntu:18.04

RUN apt-get -y update && apt-get -y install papi-tools curl && apt-get -y clean
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y