Server Container
================

This is the tooling to build a full-featured Minecraft server container. It
downloads everything as part of the build process, so the resulting image
launches immediately. (Note: This does not absolve you of the distribution
restrictions of Mojang's licenses.)

This process is customizable via build arguments.

Example:

```
$ docker build --build-arg eula=yes .
```

Build Args
----------

* `eula`: Set this to accept the Mojang EULA
* `type`: Set this to the type of server, one of: `bukkit`, `curse`, `papermc`, `forge`, `ftb`, `multimc`, `spigot`, `sponge`, `vanilla`, `custom` (default: `vanilla`)
* `version`: Set this to the Minecraft version you want, or `latest` or `snapshot` (default: `latest`)

Note: Only the `vanilla` and `papermc` types are implemented

Contents
--------

In addition to a basic Minecraft server (in `/mc`), this container holds:

* `cmd`: A program to run commands via rcon. Automatically reads connection information from `server.properties` (Fails if rcon is not enabled)
* Healthcheck: A healthcheck via rcon (TODO: Implement direct server query)
* Entrypoint: An entrypoint is implemented that will gracefully stop the server on container shutdown

Credit
======

This is heavily inspired by (and somewhat based on) [itzg/docker-minecraft-server](https://github.com/itzg/docker-minecraft-server).
