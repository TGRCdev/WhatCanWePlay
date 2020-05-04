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

class SteamUserException(Exception):
    steam_id = None
    steam_name = None

    def __init__(self, steam_id:str, steam_name:str=None):
        self.steam_id = steam_id
        if steam_name:
            steam_name = steam_name
    
    def __str__(self):
        if self.steam_name:
            return "SteamUserException, An exception occurred with the Steam user {} (Steam ID {})".format(self.steam_name, self.steam_id)
        else:
            return "SteamUserException, An exception occurred with the Steam ID is {}".format(self.steam_id)

class SteamUserCouldntGetGamesException(SteamUserException):
    def __str__(self):
        if self.steam_name:
            return "SteamUserRateLimitedException, The Steam user with the Steam name {} (Steam ID {}) is either rate-limited or set to Private, and can't have their owned games retrieved".format(self.steam_name, self.steam_id)
        else:
            return "SteamUserRateLimitedException, The Steam user with the Steam ID {} is either rate-limited or set to Private, and can't have their owned games retrieved".format(self.steam_id)

class SteamUserNoGamesException(SteamUserException):
    def __str__(self):
        if self.steam_name:
            return "SteamUserNoGamesException, The Steam user with the Steam name {} (Steam ID {}) has no games".format(self.steam_name, self.steam_id)
        else:
            return "SteamUserNoGamesException, The Steam user with the Steam ID {} has no games".format(self.steam_id)

class SteamBadVanityUrlException(Exception):
    vanity_url = None

    def __init__(self, vanity_url:str):
        self.vanity_url = vanity_url
    
    def __str__(self):
        return "SteamBadVanityUrlException, The vanity url \"{}\" is not associated with a Steam user".format(self.vanity_url)

class SteamAPIException(Exception):
    error_code = None

    def __init__(self, error_code: int):
        self.error_code = error_code
    
    def __str__(self):
        return "SteamAPIException, The Steam API returned the error code {}".format(self.error_code)