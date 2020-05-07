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

from flask import Flask, request, jsonify, Response, render_template, redirect, session, url_for
import requests
from urllib import parse
from werkzeug.exceptions import BadRequest
import json
from steam_utils import get_steam_id, get_steam_user_info
from igdb_utils import get_game_info, get_api_status
from exceptions import SteamBadVanityUrlException
from requests import HTTPError
import secrets

# Load config
config = json.load(open("config.json", "r"))
steam_key = config["steam-key"]
igdb_key = config["igdb-key"]
debug = config.get("debug", config.get("DEBUG", False))
enable_api_tests = config.get("enable_api_tests", debug)

# Create uWSGI callable
app = Flask(__name__)
app.debug = debug
app.secret_key = secrets.token_hex()

@app.route('/')
def index():
    return render_template("home.html", steam_info=session.get("steam_info", {}))

@app.route("/prototype")
def prototype():
    return render_template("prototype.html", steam_info=session.get("steam_info", {}))

@app.route("/steam_login", methods=["GET", "POST"])
def login_disabled():
    return (
        'Steam login is currently disabled<br/><a href="/">Click here to go back home</a>',
        403
    )

#@app.route("/steam_login", methods=["GET", "POST"])
def steam_login():
    if request.method == "POST":
        steam_openid_url = 'https://steamcommunity.com/openid/login'
        return_url = url_for("steam_login", _external=True)
        params = {
            'openid.ns': "http://specs.openid.net/auth/2.0",
            'openid.identity': "http://specs.openid.net/auth/2.0/identifier_select",
            'openid.claimed_id': "http://specs.openid.net/auth/2.0/identifier_select",
            'openid.mode': 'checkid_setup',
            'openid.return_to': return_url,
            'openid.realm': return_url
        }
        param_string = parse.urlencode(params)
        auth_url = steam_openid_url + "?" + param_string
        return redirect(auth_url)

    if validate_steam_identity(dict(request.args)):
        steam_id = request.args["openid.identity"].rsplit("/")[-1]
        info = get_steam_user_info(steam_key, steam_id)

        if info and info["exists"]:
            steam_data = {
                "steam_id": info["steamid"],
                "screen_name": info["personaname"],
                "avatar_thumb": info["avatarmedium"],
                "avatar": info["avatarfull"],
            }
            session["steam_info"] = steam_data
    
    return redirect(url_for("index"))

#@app.route("/steam_logout")
def steam_logout():
    session.pop("steam_info")
    return redirect(url_for("index"))

def validate_steam_identity(params):
    steam_login_url = "https://steamcommunity.com/openid/login"
    params["openid.mode"] = "check_authentication"
    r = requests.post(steam_login_url, data=params)
    if "is_valid:true" in r.text:
        return True
    return False

if __name__ == "__main__":
    app.run()