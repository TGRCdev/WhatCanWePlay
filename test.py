from steam_utils import *
from igdb_utils import *
import json

if __name__ == "__main__":
    config = json.load(open("config.json", "r"))
    steam_key = config["steamworks-key"]
    igdb_key = config["igdb-key"]

    vanity_urls = [] # Insert vanity URLs here and they'll be converted to Steam IDs

    steam_ids = [] # Insert Steam IDs directly here

    for url in vanity_urls:
        steam_ids.append(get_steam_id(steam_key, url))
    
    print(steam_ids)

    shared_games = intersect_owned_steam_games(steam_key, steam_ids, True, True, igdb_key, True)

    print("Games shared by all steam ids:")
    for game in shared_games.values():
        print(game.get("name", "!!MISSING NAME!!")
        + " (steam_id: {})".format(game["steam_id"])
        + (" (igdb_id: {})".format(game["igdb_id"]) if "igdb_id" in game.keys() else ""))