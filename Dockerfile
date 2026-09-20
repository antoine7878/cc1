FROM ubuntu:24.04

RUN dpkg --add-architecture i386 && \
    apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential \
    libc6-dev-i386 \
    gcc-multilib \
    binutils \
    bison \
    clang \
    llvm \
    wget \
    make \
    python3 \
    flex \
    curl \
    file \
    zsh \
    git \
    vim \
    gdb

RUN rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

RUN /root/.cargo/bin/cargo install cargo-nextest --locked

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended && \
    git clone https://github.com/jeffreytse/zsh-vi-mode /root/.oh-my-zsh/custom/plugins/zsh-vi-mode && \
    echo "plugins+=(zsh-vi-mode)" >> /root/.zshrc && \
    echo 'source $ZSH/oh-my-zsh.sh' >> /root/.zshrc && \
    echo 'alias t="make test"' >> /root/.zshrc && \
    echo 'alias re="make re"' >> /root/.zshrc

WORKDIR /work

CMD ["sleep", "infinity"]
