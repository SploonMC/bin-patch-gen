FROM rust:bookworm

# avoid any prompts
ARG DEBIAN_FRONTEND=noninteractive

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

RUN apt-get update && apt-get install -y --no-install-recommends \
    wget \
    curl \
    gnupg \
    lsb-release \
    ca-certificates \
    unzip \
    build-essential \
    libssl-dev \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# temurin 8
RUN wget -qO- 'https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u432-b06/OpenJDK8U-jdk_x64_linux_hotspot_8u432b06.tar.gz' | tar -xz -C /opt/
# temurin 16
RUN wget -qO- 'https://github.com/adoptium/temurin16-binaries/releases/download/jdk-16.0.2%2B7/OpenJDK16U-jdk_x64_linux_hotspot_16.0.2_7.tar.gz' | tar -xz -C /opt/
# temurin 17
RUN wget -qO- 'https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.13%2B11/OpenJDK17U-jdk_x64_linux_hotspot_17.0.13_11.tar.gz' | tar -xz -C /opt/
# temurin 21
RUN wget -qO- 'https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.5%2B11/OpenJDK21U-jdk_x64_linux_hotspot_21.0.5_11.tar.gz' | tar -xz -C /opt/

ENV JAVA_HOME_8=/opt/jdk8u432-b06
ENV JAVA_HOME_16=/opt/jdk-16.0.2+7
ENV JAVA_HOME_17=/opt/jdk-17.0.13+11
ENV JAVA_HOME_21=/opt/jdk-21.0.5+11

WORKDIR /app

COPY . .

RUN cargo install --path .

CMD ["bin_patch_gen"]
