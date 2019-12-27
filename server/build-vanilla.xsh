from utils import get_version_data
import requests
print("Downloading server...")

info = get_version_data(version_num)

with requests.get(info['downloads']['server']['url'], stream=True) as resp:
    resp.raise_for_status()
    with p"/mc/server.jar".open('wb') as dest:
        for chunk in resp.iter_content(chunk_size=8192):
            if chunk:  # Filter out keep-alive new chunks
                dest.write(chunk)
