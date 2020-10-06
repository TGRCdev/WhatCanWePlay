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
import json
import sqlite3
from requests.exceptions import ConnectTimeout, ReadTimeout
from typing import Dict, Collection, Any, Mapping, Optional
from datetime import timedelta, datetime, timezone
import os.path

api_base = "https://api.igdb.com/v4/"

config = json.load(open("config.json", "r"))
debug = config.get("debug", config.get("DEBUG", False))
db_filename = config.get("igdb-cache-filename", "igdb-cache.sqlite")

create_db_query = """
    CREATE TABLE IF NOT EXISTS game_info (
        steam_id INTEGER PRIMARY KEY,
        igdb_id INTEGER,
        name TEXT,
        cover_id TEXT,
        has_multiplayer BOOL,
        supported_players TEXT,
        time_cached REAL
    );
"""

info_age_dict = config.get("igdb-cache-info-age", {})
info_age = timedelta(**info_age_dict).total_seconds()

def fetch_and_store_token() -> str:
    r = requests.post(
        "https://id.twitch.tv/oauth2/token",
        json={
            "client_id": config["igdb-client-id"],
            "client_secret": config["igdb-secret"],
            "grant_type": "client_credentials"
        }
    )

    if debug:
        r.raise_for_status()
    
    try:
        token = r.json()
        token_dict = {
            "token": token["access_token"],
            "expire-time": datetime.now(timezone.utc).timestamp() + token["expires_in"],
            "token_type": token["token_type"]
        }
        token_file = open("bearer-token.json", "w")
        json.dump(token_dict, token_file)
        token_file.close()
        return token["access_token"]
    except Exception as e:
        print("Exception thrown while parsing token return")
        print(e)
        return ""

def get_or_refresh_token() -> str:
    try:
        if os.path.exists("bearer-token.json"):
            token_file = open("bearer-token.json")
            token = json.load(token_file)
            token_file.close()
            if datetime.now(timezone.utc).timestamp() >= token.get("expire-time", 0):
                return fetch_and_store_token()
            else:
                return token.get("token", "")
        else:
            return fetch_and_store_token()
    except json.JSONDecodeError as e:
        print("Failed to decode token JSON.")
        print(e)
        return ""
    except Exception as e:
        print("Failed to fetch bearer token.")
        print(e)
        return ""

def get_cached_games(appids: Collection[int]) -> Dict[int, Dict[str, Any]]:
    query = """
    SELECT steam_id, igdb_id, name, cover_id, has_multiplayer, supported_players
    FROM game_info
    WHERE time_cached > ? AND steam_id in ({})
    """.format(",".join(
        ["?" for _ in range(len(appids))]
    ))
    db_handle = sqlite3.connect(db_filename)
    db_handle.execute(create_db_query)

    cursor = db_handle.cursor()
    cursor.execute(query, [datetime.now(timezone.utc).timestamp() - info_age, *list(appids)])
    results = cursor.fetchall()

    return {
        game[0]: {
            "steam_id": game[0],
            "name": game[2],
            "cover_id": game[3],
            "has_multiplayer": game[4],
            "supported_players": game[5]
        } if game[1] else {}
        for game in results
    }

def update_cached_games(game_info: Mapping[int, Mapping[str, Any]]):
    query = """
    INSERT OR REPLACE INTO game_info
    (steam_id, igdb_id, name, cover_id, has_multiplayer, supported_players, time_cached)
    VALUES (?,?,?,?,?,?,?)
    """
    db_handle = sqlite3.connect(db_filename)
    db_handle.execute(create_db_query)
    cursor = db_handle.cursor()
    now = datetime.now(timezone.utc).timestamp()
    cursor.execute("BEGIN TRANSACTION")
    cursor.executemany(query, [[
        game.get("steam_id"),
        game.get("igdb_id"),
        game.get("name"),
        game.get("cover_id"),
        game.get("has_multiplayer"),
        game.get("supported_players"),
        now] for game in game_info.values()]
    )
    cursor.execute("END TRANSACTION")

def get_steam_game_info(webkey: str, appids: Collection[int], connect_timeout: Optional[float] = None, read_timeout: Optional[float] = None) -> Dict[int, Dict[str, Any]]:
    if len(appids) == 0:
        return {"errcode":0, "games":{}}
    
    appid_set = set(appids)
    
    cache_error = False

    try:
        cached_games = get_cached_games(appid_set)
    except Exception as e:
        print("An exception occurred while retrieving cached games: " + str(e))
        cached_games = {}
        cache_error = True

    if len(cached_games) == len(appid_set):
        return {"errcode": 0, "games": cached_games}

    uncached_ids = appid_set - set(cached_games.keys())
    uncached_ids_list = list(uncached_ids)

    token = get_or_refresh_token()
    if not token:
        return {"errcode": 4}

    games_dict = {}
    retrieved_games = 0
    while retrieved_games < len(uncached_ids_list):
        game_slice = uncached_ids_list[retrieved_games:500]

        retrieved_games += len(game_slice)

        try:
            r = requests.post(
                api_base + "external_games",
                data = "fields uid,game.name,game.game_modes,game.multiplayer_modes.onlinemax,game.multiplayer_modes.onlinecoopmax,game.cover.image_id; where uid = ({}) & category = 1; limit {};".format(",".join(map(str, game_slice)), len(game_slice)),
                headers = {
                    "Client-ID": webkey,
                    "Authorization": "Bearer " + token,
                    "Accept": "application/json"
                },
                timeout = (connect_timeout, read_timeout)
            )
        except ConnectTimeout:
            return {"errcode": 2}
        except ReadTimeout:
            return {"errcode": 3}
        if r.status_code == 403:
            return {"errcode": 1}
        elif r.status_code != 200:
            if debug:
                r.raise_for_status()
            return {"errcode": -1}
        
        for game in r.json():
            steam_id = game.get("uid", None)
            if not steam_id:
                continue
            steam_id = int(steam_id)
            game = game.get("game", None)
            if not game:
                continue
            uncached_ids.discard(steam_id)

            game_modes = game.get("game_modes", [])

            is_multiplayer = (2 in game_modes or 5 in game_modes)

            maxplayers = -1
            if is_multiplayer:
                for mode in game.get("multiplayer_modes", []):
                    maxplayers = max(max(mode.get("onlinemax", 1), mode.get("onlinecoopmax", 1)), maxplayers)
            else:
                maxplayers = 1

            games_dict[steam_id] = {
                "steam_id": steam_id,
                "igdb_id": game["id"],
                "name": game["name"],
                "cover_id": game.get("cover", {}).get("image_id", ""),
                "has_multiplayer": is_multiplayer,
                "supported_players": str(maxplayers) if maxplayers > 0 else "?"
            }
    
    # Any games that couldn't be retrieved probably dont exist. Store them so they don't trigger an IGDB fetch.
    for nonexist_id in uncached_ids:
        games_dict[nonexist_id] = {"steam_id": nonexist_id}
    
    if len(games_dict) == 0:
        return {"errcode":0, "games":cached_games}
    
    for id in games_dict.keys():
        cached_games[id] = games_dict[id]
    
    if not cache_error:
        try:
            update_cached_games(games_dict)
        except Exception as e:
            print("An exception occurred while updating cached games: " + str(e))

    return {
        "errcode": 0,
        "games": cached_games
    }