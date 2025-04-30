ARG VARIANT="16"
FROM mcr.microsoft.com/devcontainers/javascript-node:1-${VARIANT}

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends build-essential ca-certificates clang-format-15 clang-tidy-15 cmake curl git libzstd-dev ninja-build python3 python3-pip ssh sudo wabt wget zlib1g-dev wget git-lfs zlib1g-dev wget libffi-dev libncurses5-dev libncursesw5-dev libxml2-dev binaryen unzip

# RUN pip3 install cmake-format lit --no-cache-dir
# RUN cd /usr/bin/ && ln -s python3 python && ln -s clang-format-15 clang-format && ln -s clang-tidy-15 clang-tidy && ln -s run-clang-tidy-15 run-clang-tidy

RUN mkdir -p /opt
WORKDIR /opt

COPY install_llvm16.sh /opt/install_llvm16.sh
RUN chmod +x /opt/install_llvm16.sh

RUN wget https://github.com/llvm/llvm-project/releases/download/llvmorg-16.0.4/clang+llvm-16.0.4-x86_64-linux-gnu-ubuntu-22.04.tar.xz && /opt/install_llvm16.sh

RUN git clone https://github.com/emscripten-core/emsdk.git
WORKDIR /opt/emsdk
RUN ./emsdk install 3.1.69
RUN ./emsdk activate 3.1.69

RUN curl -sSf https://mirrors.ustc.edu.cn/misc/rustup-install.sh | sh -s -- -y
RUN bash -c ". /root/.cargo/env"
RUN bash -c ". ~/.cargo/env && rustup install 1.81.0 && rustup default 1.81.0"
WORKDIR /home/admin

WORKDIR /opt

RUN echo "export PATH=/opt/llvm16/bin:/opt:\$PATH" >> $HOME/.profile
RUN echo "export LLVM_SYS_160_PREFIX=/opt/llvm16" >> $HOME/.profile
ENV PATH=/opt/llvm16/bin:/opt:$PATH
ENV LLVM_SYS_160_PREFIX=/opt/llvm16

ENV RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
ENV RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
COPY install_rust.sh /opt/install_rust.sh
RUN chmod +x /opt/install_rust.sh
RUN /opt/install_rust.sh

RUN mkdir -p /root/.cargo && touch /root/.cargo/env
RUN echo "source \"$HOME/.cargo/env\"" >> $HOME/.profile

# install foundry
RUN curl -L https://foundry.paradigm.xyz | bash
ENV PATH=~/.foundry/bin:$PATH
RUN bash -c "source ~/.bashrc && foundryup"

WORKDIR /opt
