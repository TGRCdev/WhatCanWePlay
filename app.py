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

from flask import Flask, request, jsonify, Response, render_template, redirect
from werkzeug.exceptions import BadRequest
import json
from steam_utils import get_steam_id, get_steam_user_info
from igdb_utils import get_game_info, get_api_status
from exceptions import SteamBadVanityUrlException
from requests import HTTPError

# Load config
config = json.load(open("config.json", "r"))
steam_key = config["steam-key"]
igdb_key = config["igdb-key"]
debug = config.get("DEBUG", False)
enable_api_tests = config.get("enable_api_tests", debug)

# Create uWSGI callable
app = Flask(__name__)
app.debug = debug

@app.route('/')
def hello_world():
    root_str = ""
    if request.args:
        root_str = str(request.args.to_dict(flat=True))
    return render_template("home.html", request_info=root_str)

@app.route("/privacy")
def privacy_page():
    return render_template("privacy.html")

@app.route("/privacy.html")
def redirect_to_privacy():
    return redirect("/privacy", 301)

# get_steam_user_info
# parameters:
# steam_ids - array of integers or strings
# vanity_urls - array of strings
#
# returns:
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

# get_igdb_game_info
# parameters:
# igdb_ids - array of strings or integers
# fields - array of strings
#
# returns:
# dictionary, keys are igdb_ids, values are dictionaries
# [igdb_id]["exists"] - "true" if the game was found in the IGDB, "false" otherwise. all other values exist only if "exists" is true.
@app.route("/api/v1/get_igdb_game_info", methods=["POST"])
def get_igdb_game_info_v1():
    valid_fields = set([
        "age_ratings",
        "aggregated_rating",
        "aggregated_rating_count",
        "alternative_names",
        "artworks",
        "bundles",
        "category",
        "collection",
        "cover",
        "created_at",
        "dlcs",
        "expansions",
        "external_games",
        "first_release_date",
        "follows",
        "franchise",
        "franchises",
        "game_engines",
        "game_modes",
        "genres",
        "hypes",
        "involved_companies",
        "keywords",
        "multiplayer_modes",
        "name",
        "parent_game",
        "platforms",
        "player_perspectives",
        "popularity",
        "pulse_count",
        "rating",
        "rating_count",
        "release_date",
        "screenshots",
        "similar_games",
        "slug",
        "standalone_expansions",
        "status",
        "storyline",
        "summary",
        "tags",
        "themes",
        "time_to_beat",
        "total_rating",
        "total_rating_count",
        "updated_at",
        "url",
        "version_parent",
        "version_title",
        "videos",
        "websites"
    ])
    try:
        data = request.get_json(force=True)
        print(request.get_data(as_text=True))

        igdb_ids = data.get("igdb_ids", [])
        if not isinstance(igdb_ids, list):
            return ("igdb_ids must be an array of integers", 400)
        igdb_ids = set(igdb_ids)
        igdb_ids.discard("")
        if len(igdb_ids) == 0:
            return ("get_igdb_game_info requires igdb_ids", 400)
        try:
            igdb_ids = set(map(int, igdb_ids))
            for id in igdb_ids:
                print(id)
                if id > 2147483647: # IGDB uses 32-bit integer keys
                    return ("IGDB IDs cannot be larger than 2,147,483,647", 400)
                elif id <= 0:
                    return ("IGDB IDs must be greater than 0", 400)
            igdb_ids = set(map(str, igdb_ids))
        except TypeError:
            return ("igdb_ids must contain integers only", 400)
        
        print(igdb_ids)

        fields = data.get("fields", [])
        fields = set(fields)
        fields.discard("")
        invalid_fields = fields - valid_fields
        if len(invalid_fields) > 0:
            return ("invalid fields passed: " + ", ".join(invalid_fields), 400)
        
        if len(fields) == 0:
            igdb_info = get_game_info(igdb_key, igdb_ids)
        else:
            fields = ",".join(map(str.strip, fields))
            igdb_info = get_game_info(igdb_key, igdb_ids, fields)
        
        for id in igdb_ids:
            if not id in igdb_info.keys():
                igdb_info[id] = {"exists": False}
            else:
                igdb_info[id]["exists"] = True
        
        return jsonify(igdb_info)
    except BadRequest as b:
        return (b.description, 400)
    except HTTPError:
        return ("an unknown error occurred while requesting the IGDB api", 400)

@app.route("/api/v1/get_igdb_api_status", methods=["POST"])
def get_igdb_api_status_v1():
    return jsonify(get_api_status(igdb_key))

if enable_api_tests:
    @app.route("/api/v1/get_steam_user_info", methods=["GET"])
    def get_steam_user_info_v1_test():
        params = [
            {"name": "steam_ids", "type": "csl:string"},
            {"name": "vanity_urls", "type": "csl:string"},
        ]

        return render_template("api_test.html",
        api_function_name="get_steam_user_info",
        api_version="v1",
        api_function_params=json.dumps(params)
        )
    @app.route("/api/v1/get_igdb_game_info", methods=["GET"])
    def get_igdb_game_info_v1_test():
        params = [
            {"name": "igdb_ids", "type": "csl:int"},
            {"name": "fields", "type": "csl:string"},
        ]

        return render_template("api_test.html",
        api_function_name="get_igdb_game_info",
        api_version="v1",
        api_function_params=json.dumps(params)
        )
    @app.route("/api/v1/get_igdb_api_status", methods=["GET"])
    def get_igdb_api_status_v1_test():
        params = []

        return render_template("api_test.html",
        api_function_name="get_igdb_api_status",
        api_version="v1",
        api_function_params=json.dumps(params)
        )

if __name__ == "__main__":
    app.run()