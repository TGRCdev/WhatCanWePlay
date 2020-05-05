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

api_base = "https://api-v3.igdb.com/"

def get_steam_game_igdb_id(apikey: str, appids):
    if isinstance(appids, (str, int)): # Single game id
        appids = [str(appids)]
    
    game_data = {}
    
    i = 0
    id_str = "("
    for appid in appids:
        id_str += '"' + appid + '"'
        i += 1
        if i % 500 == 0 or i == len(appids):
            id_str += ")"

            r = requests.post(api_base + "external_games",
                data="fields uid,game; where uid = {} & category = 1; limit {};".format(id_str, min(500, len(appids))),
                headers={"user-key": apikey, "Accept": "application/json"}
            )
            r.raise_for_status()

            id_str = "("

            response = r.json()
            for game in response:
                game_info = {}
                appid = str(game["uid"])
                igdb_id = str(game["game"])
                game_info["steam_id"] = appid
                game_info["igdb_id"] = igdb_id
                game_data[appid] = game_info
        else:
            id_str += ","

    return game_data

def get_game_info(apikey: str, igdb_ids, fields="name,platforms,tags,game_modes,genres"):
    if isinstance(igdb_ids, (str, int)): # Single game id
        igdb_ids = [str(igdb_ids)]
    if len(igdb_ids) == 0:
        return {}
    if fields == "":
        return {}
    
    id_str = "("

    game_data = {}

    i = 0
    for id in igdb_ids:
        id_str += str(id)
        i += 1
        if i % 500 == 0 or i == len(igdb_ids):
            id_str += ")"

            print("fields {}; where id = {}; limit {};".format(fields, id_str, min(500, len(igdb_ids))))

            r = requests.post(api_base + "games",
                data="fields {}; where id = {}; limit {};".format(fields, id_str, min(500, len(igdb_ids))),
                headers={"Accept": "application/json", "user-key": apikey}
            )
            r.raise_for_status()

            id_str = "("

            response = r.json()
            
            for game in response:
                igdb_id = str(game.pop("id"))
                game["igdb_id"] = igdb_id
                game_data[igdb_id] = game
        else:
            id_str += ","
    
    return game_data

# Returns the IGDB info for all the defined steam games. If no info is found, sets the key to an empty dictionary
def get_steam_game_info(apikey: str, appids, fields="name,platforms,tags,game_modes,genres"):
    if isinstance(appids, (str, int)): # Single game id
        appids = [str(appids)]
    if len(appids) == 0:
        return {}

    game_data = {} # Key is steam app id
    igdb_to_steam = {} # Key is IGDB id
    
    # Step 1: Convert steam_id to igdb_id
    game_data = get_steam_game_igdb_id(apikey, appids)
    bad_entries = []
    for game in game_data.values():
        if game["igdb_id"] in igdb_to_steam.keys():
            print("DUPLICATE FOUND! (IGDB ID: {}, First Steam ID: {}, Duplicate Steam ID: {}) Ignoring...".format(game["igdb_id"], igdb_to_steam[game["igdb_id"]], game["steam_id"]))
            bad_entries.append(game["steam_id"])
        else:
            igdb_to_steam[game["igdb_id"]] = game["steam_id"]
    
    for steam_id in bad_entries:
        game_data.pop(steam_id)
    
    # From this point, do not use appids. Only use the game_data dictionary.

    # Step 2: Get info for all games in the IGDB
    # Setup IDs
    if len(game_data) == 0:
        return {}

    igdb_data = get_game_info(apikey, igdb_to_steam.keys())
    for igdb_game in igdb_data.values():
        igdb_key = igdb_game["igdb_id"]
        steam_key = igdb_to_steam[igdb_key]
        igdb_game["steam_id"] = steam_key
        game_data[steam_key] = igdb_game

    return game_data

def get_api_status(apikey:str):
    r = requests.get(api_base + "api_status", headers={"user-key": apikey})
    r.raise_for_status()

    return r.json()