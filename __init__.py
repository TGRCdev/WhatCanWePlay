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

try:
    from .lib.bin import whatcanweplay as wcwp
except:
    import shutil, platform, os
    system = platform.system()
    try:
        os.makedirs("lib/bin", exist_ok=True)
        if system == "Windows":
            shutil.copyfile("lib/wcwp_rust/target/release/whatcanweplay.dll", "lib/bin/whatcanweplay.pyd")
        else:
            shutil.copyfile("lib/wcwp_rust/target/release/libwhatcanweplay.so", "lib/bin/whatcanweplay.so")
        from .lib.bin import whatcanweplay as wcwp
    except Exception as e:
        print("Failed to load WhatCanWePlay rust library. Please go to \"lib/wcwp_rust\" and run \"cargo build --release\"")
        print(e)
        exit(1)
print("WhatCanWePlay rust lib loaded (" + str(wcwp.__file__) + ")")

from flask import Flask, request, jsonify, Response, render_template, redirect, session, url_for, make_response
import requests
from urllib import parse
from werkzeug.exceptions import BadRequest
import json
#from .steam_utils import get_steam_user_info, get_steam_user_friend_list, get_owned_steam_games
#from .igdb_utils import get_steam_game_info
from requests import HTTPError
import secrets
from datetime import timezone, datetime, timedelta
from itsdangerous import URLSafeSerializer
from os import path
import traceback

# Load config
def create_app():
    root_path = path.dirname(__file__)
    config = json.load(open(path.join(root_path, "config.json"), "r"))
    steam_key = config["steam-key"]
    igdb_key = config["igdb-client-id"]
    igdb_secret = config["igdb-secret"]
    debug = config.get("debug", config.get("DEBUG", False))
    enable_api_tests = config.get("enable-api-tests", debug)
    cookie_max_age_dict = config.get("cookie-max-age", {})
    info_max_age_dict = config.get("info-max-age", {})
    source_url = config.get("source-url", "")
    contact_email = config["contact-email"]
    privacy_email = config.get("privacy-email", contact_email)
    connect_timeout = config.get("connect-timeout", 0.0)
    donate_url = config.get("donate-url", "")
    if connect_timeout <= 0.0:
        connect_timeout = None
    read_timeout = config.get("read-timeout", 0.0)
    if read_timeout <= 0.0:
        read_timeout = None

    # Create uWSGI callable
    app = Flask(__name__)
    app.debug = debug
    app.secret_key = config.get("secret-key", secrets.token_hex()) # If not set, cookies will be invalidated every time the app is reloaded

    # Hide requests to /steam_login to prevent linking Steam ID to IP in logs
    from werkzeug import serving
    parent_log_request = serving.WSGIRequestHandler.log_request
    def log_request(self, *args, **kwargs):
        if self.path.startswith("/steam_login"):
            self.log("info", "[request to /steam_login hidden]")
            return
        
        parent_log_request(self, *args, **kwargs)
    serving.WSGIRequestHandler.log_request = log_request

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
            "source_url": source_url,
            "donate_url": donate_url
        }

    # Tries to fetch the Steam info cookie, returns an errcode and a dict
    #
    # Errcodes:
    #     0: no error, info returned. (will also return this if no cookie is present)
    #     1: bad cookie sig
    #     2: bad cookie JSON format
    #     3: info returned, but out of date (refresh recommended)
    def fetch_steam_cookie(request):
        cookie_str = request.cookies.get("steam_info")

        if not cookie_str:
            return 0, {}
        
        ser = URLSafeSerializer(app.secret_key)
        loaded, cookie_json = ser.loads_unsafe(cookie_str)

        if not loaded:
            return 1, {}
        
        try:
            steam_info = json.loads(cookie_json)
        except json.JSONDecodeError:
            return 2, {}
        
        if "expires" not in steam_info.keys() or steam_info["expires"] <= datetime.now(timezone.utc).timestamp():
            return 3, steam_info
        else:
            return 0, steam_info

    def refresh_steam_cookie(steamid: int, response):
        if steamid <= 0:
            response.set_cookie("steam_info", "", secure=True)
            return {}
        
        info = {}
        try:
            info = wcwp.steam.get_steam_users_info(steam_key, [steamid])[0]
        except wcwp.steam.BadWebkeyException:
            response.set_cookie("steam_info", "", secure=True)
            return {}
        except IndexError:
            response.set_cookie("steam_info", "", secure=True)
            return {}

        if info_max_age:
            info["expires"] = (datetime.now(timezone.utc) + timedelta(seconds=info_max_age)).timestamp()
        ser = URLSafeSerializer(app.secret_key)
        response.set_cookie(
            "steam_info",
            ser.dumps(json.dumps(info)),
            secure=True,
            httponly=True,
            max_age=cookie_max_age
        )
        return info

    @app.route('/')
    def index():
        errcode, steam_info = fetch_steam_cookie(request)
        response = Response()

        if errcode == 3:
            steam_info = refresh_steam_cookie(steam_info.get("steam_id", -1), response)
        elif errcode != 0:
            response.set_cookie("steam_info", "", secure=True)
            steam_info = {}
        
        response.data = render_template("home.html", steam_info=steam_info, **basic_info_dict())
        return response

    @app.route("/privacy")
    def privacy():
        return render_template("privacy.html", privacy_email=privacy_email, **basic_info_dict())

    @app.route("/steam_login", methods=["GET", "POST"])
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
        response.headers["X-Robots-Tag"] = "none"

        if validate_steam_identity(dict(request.args)):
            steam_id = int(request.args["openid.identity"].rsplit("/")[-1])
            refresh_steam_cookie(steam_id, response)
        
        return response

    @app.route("/steam_logout")
    def steam_logout():
        response = redirect(url_for("index"))
        response.headers["X-Robots-Tag"] = "none"
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

    # API stuff

    # Get the Friend List of the currently signed in user
    # Returns: A list of strings of Steam IDs on this users' friends list
    @app.route("/api/v1/get_friend_list", methods=["GET", "POST"] if enable_api_tests else ["POST"])
    def get_friend_list_v1():
        if request.method == "GET":
            return render_template(
                "api_test.html",
                api_function_name="get_friend_list",
                api_version="v1",
                api_function_params=[],
                steam_info=fetch_steam_cookie(request)[1],
                **basic_info_dict()
            )
        errcode, steam_info = fetch_steam_cookie(request)
        if "steam_id" not in steam_info.keys():
            return ("Not signed in to Steam", 403)
        
        try:
            friends_info = wcwp.steam.get_friend_list(
                steam_key,
                steam_info["steam_id"]
            )
            
            for user in friends_info:
                if "steam_id" in user.keys():
                    user["steam_id"] = str(user["steam_id"])
                    user["exists"] = True

            return jsonify(friends_info)
        except wcwp.steam.BadWebkeyException:
            return ("Site has bad Steam API key. Please contact us about this error at " + contact_email, 500)
        except wcwp.steam.SteamException as e:
            print(e)
            return ("An unknown error occurred. Please try again later.", 500)
        except Exception as e:
            print(traceback.format_exc())
            return ("An unknown error occurred. Please try again later.", 500)

    def refresh_igdb_token():
        try:
            token_path = path.join(root_path, "bearer-token.json")
            token = wcwp.igdb.fetch_twitch_token(igdb_key, igdb_secret)
            token["expiry"] = datetime.now(timezone.utc).timestamp() + token.get("expires_in", 0)
            token.pop("expires_in")
            json.dump(token, open(token_path, "w"))
            return token.get("access_token", "")
        except Exception:
            traceback.print_exc()
            return ""

    def get_igdb_token():
        try:
            token_path = path.join(root_path, "bearer-token.json")
            if path.exists(token_path):
                token_file = open(token_path)
                token = json.load(token_file)
                token_file.close()
                if datetime.now(timezone.utc).timestamp() >= token.get("expiry", 0):
                    return refresh_igdb_token()
                else:
                    return token.get("access_token", "")
            else:
                return refresh_igdb_token()
        except Exception:
            traceback.print_exc()
            return ""

    # Errcodes
    # -1: An error occurred with a message. Additional fields: "message"
    # 0: No error
    # 1: User has private games list. Additional fields: "user"
    # 2: User has empty games list. Additional fields: "user"
    @app.route("/api/v1/intersect_owned_games", methods=["POST", "GET"] if enable_api_tests else ["POST"])
    def intersect_owned_games_v1():
        if request.method == "GET":
            params = [
                {"name": "steamids", "type":"csl:string"},
                {"name": "include_free_games", "type":"bool", "default": False}
            ]
            return render_template(
                "api_test.html",
                api_function_name="intersect_owned_games",
                api_version="v1",
                api_function_params=json.dumps(params),
                steam_info=fetch_steam_cookie(request)[1],
                **basic_info_dict()
            )
        
        errcode, steam_info = fetch_steam_cookie(request)
        if "steam_id" not in steam_info.keys():
            return (
                json.dumps({"message": "Not signed in to Steam. Please refresh the page and try again.", "errcode": -1}),
                200
            )
        
        body = request.get_json(force=True, silent=True)

        if not isinstance(body, dict):
            return (
                json.dumps({"message": "Received a bad request. Please refresh the page and try again.", "errcode": -1}),
                200
            )

        if not body or "steamids" not in body.keys():
            return (
                json.dumps({"message": "Received a bad request. Please refresh the page and try again.", "errcode": -1}),
                200
            )
        
        try:
            steamids = set([int(id) for id in body["steamids"]])
        except (ValueError, TypeError):
            return (
                json.dumps({"message": "Received a bad request. Please refresh the page and try again.", "errcode": -1}),
                200
            )
        
        if len(steamids) < 2:
            return (
                json.dumps({"message": "Must have at least 2 users to intersect games.", "errcode": -1}),
                200
            )
        
        if len(steamids) > 10:
            return (
                json.dumps({"message": "Games intersection is capped at 10 users.", "errcode": -1}),
                200
            )

        try:
            token = get_igdb_token()
            game_info = wcwp.intersect_owned_games(steam_key, igdb_key, token, list(steamids))

            return jsonify({
                "message": "Intersected successfully",
                "games": list(game_info),
                "errcode": 0
            })
        except Exception:
            traceback.print_exc()
            return (
                json.dumps({"message": "An unknown error has occurred", "errcode": -1}),
                500
            )

    return app