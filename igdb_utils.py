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
from requests.exceptions import ConnectTimeout, ReadTimeout
from typing import Dict, Collection

api_base = "https://api-v3.igdb.com/"

# Fetches the IGDB IDs for the provided Steam App IDs
#
# Returns: Dictionary
# ["errcode"]: Integer code that explains what kind of error occured, if one did.
#
# get_steam_game_igdb_id errcodes:
#    0: no error
#    1: bad api key
#    2: connect timeout (IGDB took too long to respond)
#    3: read timeout (IGDB responded but took too long to send info)
#    -1: unknown error

def get_steam_game_igdb_id(apikey: str, appids: Collection[int], connect_timeout: float = 0.0, read_timeout: float = 0.0):
    game_data = {"errcode": 0, "games":{}}
    
    retrieved_games = 0
    while retrieved_games < len(appids):
        request_games = appids[:500]
        games_str = ",".join(request_games)
        retrieved_games += len(request_games)

        try:
            r = requests.post(api_base + "external_games",
                data="fields uid,game; where uid = ({}) & category = 1; limit {};".format(games_str, len(request_games)),
                headers={"user-key": apikey, "Accept": "application/json"},
                timeout=()
            )
        except ConnectTimeout:
            return {"errcode": 2}
        except ReadTimeout:
            return {"errcode": 3}
        if r.status_code == 403:
            return {"errcode": 1}
        elif r.status_code != 200:
            return {"errcode": -1}

    return game_data