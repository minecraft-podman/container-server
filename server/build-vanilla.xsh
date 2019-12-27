from utils import get_version_data, get_version_from_buildargs
import requests
print("Downloading server...")

info = get_version_data(get_version_from_buildargs())

with requests.get(info['downloads']['server']['url'], stream=True) as resp:
    resp.raise_for_status()
    with p"/mc/server.jar".open('wb') as dest:
        for chunk in resp.iter_content(chunk_size=8192):
            if chunk:  # Filter out keep-alive new chunks
                dest.write(chunk)


def jar_name():
    return "/mc/server.jar"
