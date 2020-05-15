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

api_base = "https://api-v3.igdb.com/"

config = json.load(open("config.json", "r"))
debug = config.get("debug", config.get("DEBUG", False))
db_filename = config.get("igdb-cache-filename", "igdb-cache.sqlite")

info_age_dict = config.get("igdb-cache-info-age", {})
info_age = timedelta(**info_age_dict).total_seconds()

def get_cached_games(appids: Collection[int]) -> Dict[int, Dict[str, Any]]:
    query = """
    SELECT steam_id, name, cover_id, has_multiplayer, supported_players
    FROM game_info
    WHERE time_cached < ? AND steam_id in ({})
    """.format(",".join(
        ["?" for _ in range(len(appids))]
    ))
    db_handle = sqlite3.connect(db_filename)
    db_handle.execute("""
    CREATE TABLE IF NOT EXISTS game_info (
        steam_id INTEGER PRIMARY KEY,
        igdb_id INTEGER NOT NULL,
        name TEXT NOT NULL,
        cover_id TEXT NOT NULL,
        has_multiplayer BOOL NOT NULL,
        supported_players TEXT NOT NULL DEFAULT('?'),
        time_cached REAL NOT NULL
    );
    """)
    cursor = db_handle.cursor()
    cursor.execute(query, [datetime.now(timezone.utc).timestamp() - info_age, *list(appids)])
    results = cursor.fetchall()

    return {
        game[0]: {
            "steam_id": game[0],
            "name": game[1],
            "cover_id": game[2],
            "has_multiplayer": game[3],
            "supported_players": game[4]
        } for game in results
    }

def update_cached_games(game_info: Mapping[int, Mapping[str, Any]]):
    query = """
    INSERT OR REPLACE INTO game_info
    (steam_id, igdb_id, name, cover_id, has_multiplayer, supported_players, time_cached)
    VALUES (?,?,?,?,?,?,?)
    """
    db_handle = sqlite3.connect(db_filename)
    db_handle.execute("""
    CREATE TABLE IF NOT EXISTS game_info (
        steam_id INTEGER PRIMARY KEY,
        igdb_id INTEGER NOT NULL,
        name TEXT NOT NULL,
        cover_id TEXT NOT NULL,
        has_multiplayer BOOL NOT NULL,
        supported_players TEXT NOT NULL DEFAULT('?'),
        time_cached REAL NOT NULL
    );
    """)
    cursor = db_handle.cursor()
    cursor.execute("BEGIN TRANSACTION")
    cursor.executemany(query, [[
        game["steam_id"],
        game["igdb_id"],
        game["name"],
        game["cover_id"],
        game["has_multiplayer"],
        game["supported_players"],
        datetime.now(timezone.utc).second] for game in game_info.values()]
    )
    cursor.execute("END TRANSACTION")

def get_steam_game_info(webkey: str, appids: Collection[int], connect_timeout: Optional[float] = None, read_timeout: Optional[float] = None) -> Dict[int, Dict[str, Any]]:
    if len(appids) == 0:
        return {"errcode":0, "games":{}}
    
    appid_set = set(appids)
    
    cached_games = get_cached_games(appid_set)

    if len(cached_games) == len(appid_set):
        return cached_games

    uncached_ids = list(appid_set - set(cached_games.keys()))

    games_dict = {}
    retrieved_games = 0
    while retrieved_games < len(uncached_ids):
        game_slice = uncached_ids[retrieved_games:500]

        retrieved_games += len(game_slice)

        try:
            r = requests.post(
                api_base + "external_games",
                data = "fields uid,game.name,game.game_modes,game.multiplayer_modes.onlinemax,game.multiplayer_modes.onlinecoopmax,game.cover.image_id; where uid = ({}) & category = 1; limit {};".format(",".join(map(str, game_slice)), len(game_slice)),
                headers = {
                    "user-key": webkey, "Accept": "application/json"
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
            steam_id = game["uid"]
            game = game["game"]

            maxplayers = -1
            for mode in game.get("multiplayer_modes", []):
                maxplayers = max(max(mode.get("onlinemax", 1), mode.get("onlinecoopmax", 1)), maxplayers)

            game_modes = game.get("game_modes", [])

            is_multiplayer = (2 in game_modes or 5 in game_modes)

            games_dict[steam_id] = {
                "steam_id": steam_id,
                "igdb_id": game["id"],
                "name": game["name"],
                "cover_id": game["cover"]["image_id"],
                "has_multiplayer": is_multiplayer,
                "supported_players": str(maxplayers) if maxplayers > 0 else "?"
            }
    
    if len(games_dict) == 0:
        return {"errcode":0, "games":{}}
    
    update_cached_games(games_dict)

    return {
        "errcode": 0,
        "games": games_dict
    }