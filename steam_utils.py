import requests
import igdb_utils
from exceptions import SteamUserCouldntGetGamesException, SteamUserNoGamesException, SteamBadVanityUrlException, SteamAPIException

api_base = "https://api.steampowered.com/"

def get_steam_id(webkey: str, vanityurl: str):
    r = requests.get(api_base + "ISteamUser/ResolveVanityURL/v1/", {"key": webkey, "vanityurl": vanityurl})
    r.raise_for_status()
    response = r.json()["response"]
    if response["success"] == 42:
        raise SteamBadVanityUrlException(vanityurl)
    elif response["success"] != 1:
        raise SteamAPIException(response["success"])
    else:
        return r.json()["response"]["steamid"]

def get_owned_steam_games(webkey: str, steamid: str, include_free_games: bool=False, include_appinfo: bool=False):
    r = requests.get(api_base + "IPlayerService/GetOwnedGames/v0001/", {"key": webkey, "steamid": steamid, "include_appinfo": include_appinfo, "include_played_free_games": include_free_games, "format": "json"})
    r.raise_for_status()

    response = r.json()["response"]
    if not "game_count" in response.keys():
        raise SteamUserCouldntGetGamesException(steamid)
    elif response["game_count"] == 0:
        raise SteamUserNoGamesException(steamid)
    games = response["games"]

    return games

def intersect_owned_steam_games(steamkey: str, steamids: list, include_free_games: bool=False, include_appinfo: bool=False, igdbkey: str=""):
    if len(steamids) == 0:
        return set()
    
    game_info = {} # Game info by steam ID
    
    shared_games = None

    for steamid in steamids:
        games_list = get_owned_steam_games(steamkey, steamid, True, True)
        games_set = set()

        for game in games_list:
            appid = str(game.pop("appid"))
            game_info[appid] = {"steam_id": appid, "name": game["name"]}
            games_set.add(appid)
        
        if shared_games == None:
            print("constructing games set")
            shared_games = games_set
        else:
            print("intersecting games set")
            shared_games = shared_games & games_set
        print("remaining games: {}".format(len(shared_games)))
        if len(shared_games) == 0:
            print("reached 0 remaining games. bailing")
            return {}
    
    if igdbkey != "":
        more_game_details = igdb_utils.get_steam_game_info(igdbkey, shared_games)
        for id in more_game_details.keys():
            game_info[id] = more_game_details[id]
    
    shared_games_dict = {}

    for key in shared_games:
        shared_games_dict[key] = game_info[key]

    return shared_games_dict