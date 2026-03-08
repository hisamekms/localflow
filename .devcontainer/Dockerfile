# syntax=docker/dockerfile:1.6
# Devcontainer向けの、安全性を考慮したDockerfile

FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive
ARG USERNAME=dev
ARG USER_UID=1000
ARG USER_GID=1000

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    git \
    jq \
    fish \
    build-essential \
    unzip \
    zsh \
    sudo \
    # Playwright browsers dependencies
    libglib2.0-0t64 \
    libnss3 \
    libnspr4 \
    libdbus-1-3 \
    libatk1.0-0t64 \
    libatk-bridge2.0-0t64 \
    libcups2t64 \
    libx11-6 \
    libxcomposite1 \
    libxdamage1 \
    libxext6 \
    libxfixes3 \
    libxrandr2 \
    libgbm1 \
    libdrm2 \
    libxcb1 \
    libxkbcommon0 \
    libpango-1.0-0 \
    libcairo2 \
    libasound2t64 \
    libatspi2.0-0t64 \
  && install -dm 755 /etc/apt/keyrings \
  && curl -fSs https://mise.jdx.dev/gpg-key.pub | tee /etc/apt/keyrings/mise-archive-keyring.asc > /dev/null \
  && echo "deb [signed-by=/etc/apt/keyrings/mise-archive-keyring.asc] https://mise.jdx.dev/deb stable main" | tee /etc/apt/sources.list.d/mise.list \
  && apt-get update \
  && apt-get install -y --no-install-recommends mise \
  && rm -rf /var/lib/apt/lists/*

RUN userdel -r "$(getent passwd ${USER_UID} | cut -d: -f1)" 2>/dev/null || true \
  && groupdel "$(getent group ${USER_GID} | cut -d: -f1)" 2>/dev/null || true \
  && groupadd --gid ${USER_GID} ${USERNAME} \
  && useradd -m --uid ${USER_UID} --gid ${USER_GID} -s /bin/bash ${USERNAME} \
  && echo "${USERNAME} ALL=(ALL) NOPASSWD:ALL" >/etc/sudoers.d/${USERNAME} \
  && chmod 0440 /etc/sudoers.d/${USERNAME}

USER ${USERNAME}
WORKDIR /home/${USERNAME}

RUN set -eux; \
  curl -fsSL https://claude.ai/install.sh | bash;
