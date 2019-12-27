"""
Assemble a minecraft server from the given build args.

Not really meant to be used outside of the build
"""
import sys
sys.path.append(str(p".".resolve()))

from sys import exit
from utils import get_versions_data

if 'eula' not in ${...}:
    exit("Must accept EULA at https://account.mojang.com/documents/minecraft_eula")

p"/mc".mkdir()

p"/mc/eula.txt".write_text("""
# Generated by assemble.xsh
eula=true
""")

if $version.lower() == 'latest':
    version_num = get_versions_data()['latest']['release']
elif $version.lower() == 'snapshot':
    version_num = get_versions_data()['latest']['snapshot']
else:
    version_num = $version

if not pf"build-{$type}.xsh".exists():
    exit(f"Unknown server type {$type}")


# Overridables
def java_args():
    return []


def jar_name():
    return None


def server_invocation():
    return ["java", *java_args(), "-jar", jar_name(), "nogui"]


def make_bourne_command(args):
    return " ".join(f"'{bit}'" for bit in args)


source build-$type.xsh

p"/mc/launch".write_text(f"""#!/bin/sh
cd /mc
exec {make_bourne_command(server_invocation())} "$*"
""")
p"/mc/launch".chmod(0o755)

p"/mc/healthcheck".write_text(f"""#!/bin/sh
exec rcon "list"
""")
p"/mc/healthcheck".chmod(0o755)
