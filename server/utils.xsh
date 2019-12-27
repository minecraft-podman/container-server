import requests
import functools


@functools.lru_cache()
def get_versions_data():
    resp = requests.get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
    resp.raise_for_status()
    return resp.json()


@functools.lru_cache()
def get_version_data(num):
    summary = [d for d in get_versions_data()['versions'] if d['id'] == num][0]

    resp = requests.get(summary['url'])
    resp.raise_for_status()
    return resp.json()


def get_version_from_buildargs():
    if $version.lower() == 'latest':
        return get_versions_data()['latest']['release']
    elif $version.lower() == 'snapshot':
        return get_versions_data()['latest']['snapshot']
    else:
        return $version
