FROM ubuntu:22.04
RUN sed -i s/archive.ubuntu.com/mirrors.aliyun.com/g /etc/apt/sources.list
RUN sed -i s/security.ubuntu.com/mirrors.aliyun.com/g /etc/apt/sources.list
RUN apt update -y && apt install -y --no-install-recommends build-essential ca-certificates clang-format-15 clang-tidy-15 cmake curl git libzstd-dev ninja-build python3 python3-pip ssh sudo wabt wget zlib1g-dev wget git-lfs zlib1g-dev wget libffi-dev libncurses5-dev libncursesw5-dev libxml2-dev binaryen unzip && rm -rf /var/lib/apt/lists/*
RUN pip3 install cmake-format lit --no-cache-dir
RUN cd /usr/bin/ && ln -s python3 python && ln -s clang-format-15 clang-format && ln -s clang-tidy-15 clang-tidy && ln -s run-clang-tidy-15 run-clang-tidy
RUN useradd -m -u 500 -U -G sudo -s /bin/bash admin &&     passwd -d admin
RUN mkdir -p /opt
WORKDIR /opt
RUN wget https://github.com/llvm/llvm-project/releases/download/llvmorg-16.0.4/clang+llvm-16.0.4-x86_64-linux-gnu-ubuntu-22.04.tar.xz
# RUN wget https://github.com/llvm/llvm-project/releases/download/llvmorg-15.0.0/clang+llvm-15.0.0-x86_64-linux-gnu-rhel-8.4.tar.xz
RUN git clone https://github.com/emscripten-core/emsdk.git
WORKDIR /opt/emsdk
RUN ./emsdk install 3.1.69
RUN ./emsdk activate 3.1.69
USER root
RUN curl -sSf https://mirrors.ustc.edu.cn/misc/rustup-install.sh | sh -s -- -y
RUN bash -c ". /root/.cargo/env"
COPY cargo_config /root/.cargo/config
RUN bash -c ". ~/.cargo/env && rustup install 1.81.0 && rustup default 1.81.0"
WORKDIR /home/admin
USER root
WORKDIR /opt
COPY install_llvm16.sh /opt/install_llvm16.sh
RUN chmod +x /opt/install_llvm16.sh
# COPY install_llvm15.sh /opt/install_llvm15.sh
# RUN chmod +x /opt/install_llvm15.sh

RUN echo "export PATH=/opt/llvm15/bin:/opt:\$PATH" >> ~/.bash_profile
RUN echo "export LLVM_SYS_160_PREFIX=/opt/llvm16" >> ~/.bash_profile
ENV PATH=/opt/llvm16/bin:/opt:$PATH
ENV LLVM_SYS_160_PREFIX=/opt/llvm16

ENV RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
ENV RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
COPY install_rust.sh /opt/install_rust.sh
RUN chmod +x /opt/install_rust.sh
RUN /opt/install_rust.sh

RUN mkdir -p /root/.cargo && touch /root/.cargo/env

# install solidity tools
# install foundry
RUN curl -L https://foundry.paradigm.xyz | bash
ENV PATH=~/.foundry/bin:$PATH
RUN bash -c "source ~/.bashrc && foundryup"

WORKDIR /opt
