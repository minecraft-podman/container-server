TODO: Container Entrypoint
====================

Needs to:

* Spawn given process inside of a process group
* Forward signals to process group
* If given `/mc/launch`, on SIGTERM etc send `stop` to rcon, and wait for process to exit
* Reap zombies
