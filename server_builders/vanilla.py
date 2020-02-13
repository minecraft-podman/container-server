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

        self.container.add_url(info['downloads']['server']['url'], "/mc/server.jar")
