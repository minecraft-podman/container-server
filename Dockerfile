# Build RCON helper
FROM rust:alpine AS build-rcon
WORKDIR /tmp
COPY rcon .
RUN cargo build --release


# Gather the server
FROM xonsh/xonsh:alpine AS build-server
# Set to anything to accept the Mojang EULA
ARG eula
# Minecraft version ("latest"|"snapshot"|version number, default latest)
ARG version=latest
# Server type (bukkit|spigot|paper|forge|ftb|curse|vanilla|sponge|custom)
ARG type=vanilla

ENV XONSH_SHOW_TRACEBACK True
RUN xpip install requests
WORKDIR /tmp
COPY server .
RUN xonsh assemble.xsh


# Assemble the final container
FROM java:8-alpine

COPY --from=build-rcon /tmp/target/release/rcon /usr/bin/rcon
COPY --from=build-server /mc /mc
VOLUME /mc/world
