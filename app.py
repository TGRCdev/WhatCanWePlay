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

from flask import Flask, request, jsonify, Response, render_template, redirect, session, url_for, make_response
import requests
from urllib import parse
from werkzeug.exceptions import BadRequest
import json
from steam_utils import get_steam_user_info
from igdb_utils import get_game_info, get_api_status
from exceptions import SteamBadVanityUrlException
from requests import HTTPError
import secrets
from datetime import timezone, datetime, timedelta
from itsdangerous import URLSafeSerializer

# Load config
config = json.load(open("config.json", "r"))
steam_key = config["steam-key"]
igdb_key = config["igdb-key"]
debug = config.get("debug", config.get("DEBUG", False))
enable_api_tests = config.get("enable-api-tests", debug)
cookie_max_age_dict = config.get("cookie-max-age", {})
info_max_age_dict = config.get("info-max-age", {})
source_url = config.get("source-url", "")
contact_email = config["contact-email"]
privacy_email = config.get("privacy-email", contact_email)

# Create uWSGI callable
app = Flask(__name__)
app.debug = debug
app.secret_key = secrets.token_hex()

# Setup cookie max_age
cookie_max_age = timedelta(**cookie_max_age_dict).total_seconds()
if cookie_max_age == 0:
    cookie_max_age = None
info_max_age = timedelta(**info_max_age_dict).total_seconds()

print("cookies set to expire after {} seconds".format(cookie_max_age))
print("steam info set to refresh after {} seconds".format(info_max_age))

def basic_info_dict():
    email_rev = contact_email.split("@")
    return {
        "contact_email_user_reversed": email_rev[0][::-1],
        "contact_email_domain_reversed": email_rev[1][::-1],
        "source_url": source_url
    }

def fetch_steam_info(cookie_data:str): # Returns a tuple of (errcode:int, data:dict)
    if not cookie_data:
        return 1, None # No data

    auth_s = URLSafeSerializer(app.secret_key)
    good_sig, steam_data = auth_s.loads_unsafe(cookie_data)
    if good_sig:
        try:
            steam_info = json.loads(steam_data)
            full_keys = set(["steam_id", "screen_name", "avatar_thumb", "avatar", "expires"])

            if "steam_id" not in steam_info.keys():
                return 4, None # Good signature and loaded proper, but missing information
            elif info_max_age == 0 or steam_info.get("expires", 0) <= datetime.now(timezone.utc).timestamp() or len(steam_info) != len(full_keys):
                return 5, steam_info # Good signature and loaded proper, but needs to be refreshed    
                    
            return 0, steam_info # Cookie has a good signature and loaded proper
        except json.JSONDecodeError:
            return 2, None # Cookie has invalid JSON data, throw out
    else:
        return 3, None # Cookie has a bad signature, throw out

def refresh_steam_cookie(steam_info, response):
    if not steam_info:
        response.set_cookie("steam_info", "", secure=True)
        return {}

    steam_id = steam_info.get("steam_id", None)
    if not steam_id or not response:
        response.set_cookie("steam_info", "", secure=True)
        return {}
    
    info = get_steam_user_info(steam_key, steam_id)

    if info and info["exists"]:
        steam_data = {
            "steam_id": info["steamid"],
            "screen_name": info["personaname"],
            "avatar_thumb": info["avatar"],
            "avatar": info["avatarmedium"],
            "expires": datetime.now(timezone.utc).timestamp() + info_max_age,
        }
        auth_s = URLSafeSerializer(app.secret_key)
        response.set_cookie(
            "steam_info",
            auth_s.dumps(json.dumps(steam_data if info_max_age != 0 else {"steam_id": info["steamid"]}, indent=None)),
            max_age=cookie_max_age,
            secure=True,
            httponly=True,
            samesite="Lax",
        )
        return steam_data
    else:
        response.set_cookie("steam_info", "", secure=True)
        return {}

@app.route('/')
def index():
    errcode, steam_info = fetch_steam_info(request.cookies.get("steam_info", None))

    response = Response()
    if errcode not in (0,1):
        steam_info = refresh_steam_cookie(steam_info, response)
    response.data = render_template("home.html", steam_info=steam_info, **basic_info_dict())
    
    return response

@app.route("/privacy")
def privacy():
    return render_template("privacy.html", privacy_email=privacy_email, **basic_info_dict())

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
    
    response = redirect(url_for("index"))

    if validate_steam_identity(dict(request.args)):
        steam_id = request.args["openid.identity"].rsplit("/")[-1]
        refresh_steam_cookie({"steam_id": steam_id}, response)
    
    return response

#@app.route("/steam_logout")
def steam_logout():
    response = redirect(url_for("index"))
    response.set_cookie("steam_info", "", secure=True)
    return response

def validate_steam_identity(params):
    try:
        steam_login_url = "https://steamcommunity.com/openid/login"
        params["openid.mode"] = "check_authentication"
        r = requests.post(steam_login_url, data=params)
        if "is_valid:true" in r.text:
            return True
        return False
    except:
        return False
    

if __name__ == "__main__":
    app.run()