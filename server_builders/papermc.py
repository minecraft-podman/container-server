import json
import shutil
from urllib.request import urlopen

import requests

from ._common import ServerBuilder, get_version_from_buildargs


class Builder(ServerBuilder):
    def jar_name(self):
        return "/mc/server.jar"

    __steps__ = [
        'step_download_server', 'step_update_volumes',
    ]

    def step_download_server(self):
        """Downloading server"""

        version = get_version_from_buildargs(self.version)

        # For whatever reason, the PaperMC API and urllib don't get along
        with requests.get(f'https://papermc.io/api/v1/paper/{version}/') as r:
            r.raise_for_status()
            buildinfo = r.json()

        build = buildinfo['builds']['latest']

        with requests.get(f"https://papermc.io/api/v1/paper/{version}/{build}/download", stream=True) as src:
            with (self.root / "mc" / "server.jar").open('wb') as dest:
                for chunk in src.iter_content(chunk_size=None):
                    dest.write(chunk)

    def step_update_volumes(self):
        self.container.volumes |= {
            '/mc/paper.yml',
            '/mc/bukkit.yml',
            '/mc/spigot.yml',
            '/mc/cache',
        }
