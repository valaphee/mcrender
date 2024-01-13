# Compile swiftshader
FROM ubuntu:22.04 as swiftshader-builder

RUN apt-get update -y && apt-get install -y wget gnupg ca-certificates software-properties-common git cmake g++ gcc libx11-dev zlib1g-dev libxext-dev

RUN git clone https://github.com/google/swiftshader.git
RUN cmake swiftshader/. -Bswiftshader/build/
RUN cmake --build swiftshader/build/. --parallel 8

# Compile
FROM rust:1.75.0 as rust-builder

WORKDIR /usr/src/mcrender
COPY . .
RUN cargo install --path .

# Run
FROM ubuntu:22.04

COPY --from=rust-builder /usr/local/cargo/bin/mcrender /usr/local/bin/mcrender
COPY --from=swiftshader-builder /swiftshader/build/Linux/libvulkan.so.1 /lib/x86_64-linux-gnu/

CMD ["mcrender"]

EXPOSE 8080
