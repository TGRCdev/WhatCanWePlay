# This file is a part of WhatCanWePlay
# Copyright (C) 2020 TGRCDev

# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published
# by the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.

# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.

# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

import requests
import igdb_utils
from exceptions import SteamUserCouldntGetGamesException, SteamUserNoGamesException, SteamBadVanityUrlException, SteamAPIException

api_base = "https://api.steampowered.com/"

def get_steam_id(webkey: str, vanityurl: str) -> str:
    r = requests.get(api_base + "ISteamUser/ResolveVanityURL/v1/", {"key": webkey, "vanityurl": vanityurl})
    r.raise_for_status()
    response = r.json()["response"]
    if response["success"] == 42:
        raise SteamBadVanityUrlException(vanityurl)
    elif response["success"] != 1:
        raise SteamAPIException(response["success"])
    else:
        return r.json()["response"]["steamid"]

def get_steam_user_info(webkey: str, steamids):
    if isinstance(steamids,(str,int)):
        steamids = [str(steamids)]
    if len(steamids) == 0:
        return {}
    
    steamid_str = ",".join(map(str, steamids))
    
    r = requests.get(api_base + "ISteamUser/GetPlayerSummaries/v2/", {"key": webkey, "steamids": steamid_str})
    r.raise_for_status()
    response = r.json()["response"]

    if len(steamids) == 1:
        user = {}
        if len(response["players"]) > 0:
            user = response["players"][0]
            user["exists"] = True
        else:
            user["exists"] = False
        return user

    user_dict = {}

    for steam_id in steamids:
        user_dict[str(steam_id)] = {"exists": False}

    for user in response["players"]:
        steam_id = str(user["steamid"])
        user_dict[steam_id] = user
        user_dict[steam_id]["exists"] = True

    return user_dict

def get_owned_steam_games(webkey: str, steamid: str, include_free_games: bool=False, include_appinfo: bool=False) -> list:
    r = requests.get(api_base + "IPlayerService/GetOwnedGames/v0001/", {"key": webkey, "steamid": steamid, "include_appinfo": include_appinfo, "include_played_free_games": include_free_games, "format": "json"})
    r.raise_for_status()

    response = r.json()["response"]
    if not "game_count" in response.keys():
        raise SteamUserCouldntGetGamesException(steamid)
    elif response["game_count"] == 0:
        raise SteamUserNoGamesException(steamid)
    games = response["games"]

    return games

def intersect_owned_steam_games(steamkey: str, steamids: list, include_free_games: bool=False, include_appinfo: bool=False, igdbkey: str="", remove_non_igdb: bool=False) -> dict:
    if len(steamids) == 0:
        return {}
    
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
        if remove_non_igdb:
            game_info = more_game_details
        else:
            for id in more_game_details.keys():
                game_info[id] = more_game_details[id]
    
    shared_games_dict = {}

    for key in shared_games:
        if key in game_info.keys():
            shared_games_dict[key] = game_info[key]

    return shared_games_dict