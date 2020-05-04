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

from flask import Flask, request, jsonify, Response, render_template
from werkzeug.exceptions import BadRequest
import json
from steam_utils import get_steam_id, get_steam_user_info
from exceptions import SteamBadVanityUrlException
app = Flask(__name__)

# Load config
config = json.load(open("config.json", "r"))
steam_key = config["steam-key"]
igdb_key = config["igdb-key"]
app.debug = config.get("DEBUG", False)

@app.route('/')
def hello_world():
    return 'Hello, World!\n' + str(request.args)

# get_steam_user_info - POST
# parameters (encoded JSON body):
# steam_ids - array of integers or strings
# vanity_urls - array of strings
#
# returns (encoded JSON response):
# dictionary
# ["users"] - dictionary, key is steam id, value is user data dictionary
# ["vanity_to_steam_ids"] - dictionary, key is vanity url, value is steam id or "" if invalid
@app.route("/api/v1/get_steam_user_info", methods=["POST"])
def get_steam_user_info_v1():
    try:
        data = request.get_json(force=True)

        vanity_urls = data.get("vanity_urls", [])
        if not isinstance(vanity_urls, list):
            return ("vanity_urls must be an array of strings", 400)
        vanity_urls = set(vanity_urls)
        vanity_urls.discard("")

        steam_ids = data.get("steam_ids", [])
        if not isinstance(steam_ids, list):
            return ("steam_ids must be an array of strings or integers", 400)
        steam_ids = set(steam_ids)
        steam_ids.discard("")

        if len(vanity_urls) == 0 and len(steam_ids) == 0:
            return ("get_steam_user_info requires either \"vanity_urls\" or \"steam_ids\"", 400)
        
        vanity_to_steamid = {}

        if len(vanity_urls) != 0:
            for vanity_url in vanity_urls:
                try:
                    steam_id = get_steam_id(steam_key, vanity_url)
                    vanity_to_steamid[vanity_url] = steam_id
                    steam_ids.add(steam_id)
                except SteamBadVanityUrlException:
                    vanity_to_steamid[vanity_url] = ""

        user_info = {"users": get_steam_user_info(steam_key, steam_ids), "vanity_to_steam_ids": vanity_to_steamid}
        
        return jsonify(user_info)
    except BadRequest as b:
        return (b.description, 400)
    
    # REMOVE
    return ("", 200)

if app.debug:
    @app.route("/api/v1/get_steam_user_info", methods=["GET"])
    def get_steam_user_info_v1_test():
        params = [
            {"name": "steam_ids", "type": "csl"},
            {"name": "vanity_urls", "type": "csl"},
        ]

        return render_template("api_test.html",
        api_function_name="get_steam_user_info",
        api_version="v1",
        api_function_params=json.dumps(params)
        )

if __name__ == "__main__":
    app.run()