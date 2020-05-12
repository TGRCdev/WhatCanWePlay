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
from typing import Mapping, Any, Collection, Dict, List, Optional
import json

api_base = "https://api.steampowered.com/"

config = json.load(open("config.json", "r"))
debug = config.get("debug", config.get("DEBUG", False))

# Fetches public information about a list of Steam users
#
# Returns: Dictionary
# ["errcode"]: Integer code that explains what kind of error occurred, if one did.
#
# get_steam_user_info errcodes:
#    0: no error
#    1: bad api key
#    2: connect timeout (Steam took too long to respond)
#    3: read timeout (Steam responded but took too long to send info)
#    -1: unknown error
#    
# ["users"]: Dictionary of all users retrieved. The key is a steam id, and the value is a dictionary of data.
# User Dictionary Format:
# ["exists"]: Boolean value that is always set. True if the profile exists and has been set up, False otherwise
# ["steam_id"]: Steam ID (integer)
# ["screen_name"]: The user's Steam screen name
# ["avatar_thumb"]: A url to a 32x32 size version of their Steam avatar picture
# ["avatar"]: A url to a 64x64 size version of their Steam avatar picture
# ["visibility"]: The user's profile visibility. 1 = Private, 2 = Friends Only, 3 = Public
def get_steam_user_info(webkey: str, steamids: Collection[int], connect_timeout: Optional[float] = None, read_timeout: Optional[float] = None) -> Dict[int, Dict[str, Any]]:
    if len(steamids) == 0:
        return {"errcode": 0, "users":{}} # Technically not an error
    
    return_dict = {"errcode": 0}
    user_dict = {int(steam_id): {"exists": False} for steam_id in steamids}

    retrieved_users = 0
    while retrieved_users < len(steamids):
        request_users = steamids[retrieved_users:100]
        steamid_str = ",".join(map(str, request_users))
        retrieved_users += len(request_users)
        try:
            r = requests.get(
                api_base + "ISteamUser/GetPlayerSummaries/v2/",
                {"key": webkey, "steamids": steamid_str, "format":"json"},
                timeout=(connect_timeout, read_timeout)
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

        response = r.json()["response"]

        for user in response["players"]:
            if user.get("profilestate", 0) == 1:
                user_info = {"exists": True}
                steam_id = int(user["steamid"])
                user_info["steam_id"] = steam_id
                user_info["screen_name"] = user["personaname"]
                user_info["avatar_thumb"] = user["avatar"]
                user_info["avatar"] = user["avatarmedium"]
                user_info["visibility"] = user["communityvisibilitystate"]
                user_info["online"] = user.get("personastate", 0) != 0
                user_dict[steam_id] = user_info
    
    return_dict["users"] = user_dict

    return return_dict

# Fetch a list of Steam app IDs that a user owns
#
# Returns: Dictionary
# ["errcode"]: Integer code that explains what kind of error occurred, if one did.
#
# get_owned_steam_games errcodes:
#    0: no error
#    1: bad api key
#    2: connect timeout (Steam took too long to respond)
#    3: read timeout (Steam responded but took too long to send info)
#    4: games not visible
#    -1: unknown error
#
# ["games"]: List of owned Steam App IDs
def get_owned_steam_games(webkey: str, steamid: int, include_free_games: bool=False, connect_timeout: float = 0.0, read_timeout: float = 0.0) -> Dict[str, Any]:
    try:
        r = requests.get(
            api_base + "IPlayerService/GetOwnedGames/v0001/",
            {
                "key": webkey,
                "steamid": steamid,
                "include_appinfo": False,
                "include_played_free_games": include_free_games,
                "format": "json"},
                timeout=(connect_timeout, read_timeout)
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

    response = r.json()["response"]
    if not "game_count" in response.keys():
        return {"errcode": 4}
    else:
        return {"errcode": 0, "games": [game["appid"] for game in response.get("games", [])]}

# Fetch a list of Steam IDs that are friends with the user
#
# Returns: Dictionary
# ["errcode"]: Integer code that explains what kind of error occurred, if one did.
#
# get_steam_user_friend_list errcodes:
#    0: no error
#    1: bad api key
#    2: connect timeout (Steam took too long to respond)
#    3: read timeout (Steam responded but took too long to send info)
#    4: friends not visible
#    -1: unknown error
#
# ["friends"]: List of Steam IDs that are friends
def get_steam_user_friend_list(webkey: str, steamid: int, connect_timeout: float = 0, read_timeout: float = 0) -> Dict[str, Any]:
    try:
        r = requests.get(
            api_base + "ISteamUser/GetFriendList/v0001/",
            {"key": webkey,
            "steamid": steamid,
            "relationship": "friend",
            "format": "json"},
            timeout=(connect_timeout, read_timeout)
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
    
    response = r.json()
    if "friendslist" not in response.keys() or "friends" not in response["friendslist"].keys():
        return {"errcode": 4}
    else:
        return {"errcode": 0, "friends": [int(friend["steamid"]) for friend in response["friendslist"]["friends"]]}