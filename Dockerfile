FROM ubuntu:mantic-20230712

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ARG USERNAME=compositor
ARG RUST_VERSION=1.72

ENV DEBIAN_FRONTEND=noninteractive
ENV WEB_RENDERER_PATH=/home/$USERNAME/project/web_renderer

RUN apt-get update -y -qq && \
  apt-get install -y \
    build-essential curl pkg-config libssl-dev libclang-dev git sudo \
    libnss3 libatk1.0-0 libatk-bridge2.0-0 libgdk-pixbuf2.0-0 libgtk-3-0 \
    xvfb \
    libegl1-mesa-dev libgl1-mesa-dri libxcb-xfixes0-dev mesa-vulkan-drivers \
    ffmpeg libavcodec-dev libavformat-dev libavfilter-dev libavdevice-dev && \
  rm -rf /var/lib/apt/lists/*

RUN useradd -ms /bin/bash $USERNAME && adduser $USERNAME sudo
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers
USER $USERNAME
WORKDIR /home/$USERNAME

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN source ~/.cargo/env && rustup install $RUST_VERSION && rustup default $RUST_VERSION

ENV DISPLAY=:99
ENV XDG_RUNTIME_DIR=/home/$USERNAME/.cache/xdgr

COPY --chown=$USERNAME:$USERNAME . /home/$USERNAME/project
WORKDIR /home/$USERNAME/project

RUN source ~/.cargo/env && cargo build --release
ENV MEMBRANE_VIDEO_COMPOSITOR_MAIN_EXECUTABLE_PATH=/home/$USERNAME/project/target/release/main_process
ENV MEMBRANE_VIDEO_COMPOSITOR_PROCESS_HELPER_PATH=/home/$USERNAME/project/target/release/process_helper
ENV LD_LIBRARY_PATH=/home/$USERNAME/project/target/release/lib

ENTRYPOINT ["./docker/entrypoint.sh"]
