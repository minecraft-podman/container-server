import shutil
from urllib.request import urlopen

from ._common import ServerBuilder, get_version_data, get_version_from_buildargs


class Builder(ServerBuilder):
    def jar_name(self):
        return "/mc/server.jar"

    __steps__ = [
        'step_download_server',
    ]

    def step_download_server(self):
        """Downloading server"""

        info = get_version_data(get_version_from_buildargs(self.version))

        with urlopen(info['downloads']['server']['url']) as src:
            with (self.root / "mc" / "server.jar").open('wb') as dest:
                shutil.copyfileobj(src, dest)
