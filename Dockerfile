# Note: We're cross-compiling from debian to alpine (musl) because of proc_macro
# See https://github.com/rust-lang/rust/issues/40174
FROM rust:buster AS rustbuilder
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update 
RUN apt-get install -y musl-tools


# Build RCON helper
FROM rustbuilder AS build-cmd
COPY cmd /tmp/cmd
COPY localmc /tmp/localmc
WORKDIR /tmp/cmd
RUN cargo build --target x86_64-unknown-linux-musl --release


# Build Server List helper
FROM rustbuilder AS build-status
COPY status /tmp/status
COPY localmc /tmp/localmc
COPY mcproto-min-async /tmp/mcproto-min-async
WORKDIR /tmp/status
RUN cargo build --target x86_64-unknown-linux-musl --release


# Build entrypoint
FROM alpine AS build-entrypoint
RUN apk add tar
ADD https://github.com/itzg/mc-server-runner/releases/download/1.3.3/mc-server-runner_1.3.3_linux_amd64.tar.gz /
RUN tar xaf mc-server-runner_1.3.3_linux_amd64.tar.gz



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
FROM openjdk:8-jre-alpine

COPY --from=build-cmd /tmp/cmd/target/x86_64-unknown-linux-musl/release/cmd /usr/bin/cmd
COPY --from=build-status /tmp/status/target/x86_64-unknown-linux-musl/release/status /usr/bin/status
COPY --from=build-entrypoint /mc-server-runner /mc-server-runner
COPY --from=build-server /mc /mc
VOLUME /mc/world

ENTRYPOINT ["/mc-server-runner", "-shell", "/bin/sh"]
CMD ["/mc/launch"]
HEALTHCHECK --start-period=5m CMD ["status"]
