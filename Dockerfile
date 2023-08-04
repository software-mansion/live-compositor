FROM ubuntu:mantic-20230712

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ARG USERNAME=compositor
ARG NODE_VERSION=18.12.1
ARG NVM_VERSION=0.39.1
ARG RUST_VERSION=1.70

ENV DEBIAN_FRONTEND=noninteractive
ENV WEB_RENDERER_PATH=/home/$USERNAME/project/web_renderer

RUN apt-get update -y -qq && \
  apt-get install -y \
    build-essential curl pkg-config libssl-dev libclang-dev git sudo \
    libnss3 libatk1.0-0 libatk-bridge2.0-0 libgdk-pixbuf2.0-0 libgtk-3-0 \
    xvfb \
    libegl1-mesa libgl1-mesa-dri libxcb-xfixes0-dev mesa-vulkan-drivers \
    ffmpeg libavcodec-dev libavformat-dev libavfilter-dev libavdevice-dev && \
  rm -rf /var/lib/apt/lists/*

RUN useradd -ms /bin/bash $USERNAME && adduser $USERNAME sudo
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers
USER $USERNAME
WORKDIR /home/$USERNAME

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN source ~/.cargo/env && rustup install $RUST_VERSION && rustup default $RUST_VERSION

RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v$NVM_VERSION/install.sh | bash \
  && source ~/.nvm/nvm.sh \
  && nvm install $NODE_VERSION

ENV DISPLAY=:99
ENV XDG_RUNTIME_DIR=/home/$USERNAME/.cache/xdgr

COPY --chown=$USERNAME:$USERNAME . /home/$USERNAME/project
WORKDIR /home/$USERNAME/project

RUN source ~/.cargo/env && cargo build --release && cargo install --path .

ENTRYPOINT ["./docker/entrypoint.sh"]
