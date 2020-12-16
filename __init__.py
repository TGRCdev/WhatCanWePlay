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
    import shutil, platform, os
    path_prefix = os.path.dirname(os.path.abspath(__file__))
    source = ""
    dest = ""
    if platform.system() == "Windows":
        source = "lib/wcwp_rust/target/release/whatcanweplay.dll"
        dest = "lib/bin/whatcanweplay.pyd"
    else:
        source = "lib/wcwp_rust/target/release/libwhatcanweplay.so"
        dest = "lib/bin/whatcanweplay.so"
    source = os.path.join(path_prefix, source)
    dest = os.path.join(path_prefix, dest)

    if not os.path.exists(dest):
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        shutil.copy2(source, dest)
    else:
        source_time = os.path.getmtime(source)
        dest_time = os.path.getmtime(dest)
        try:
            if dest_time < source_time:
                print("Updating WhatCanWePlay rust library with newer library file...")
                shutil.copy2(source, dest)
        except Exception:
            print("Failed to update WhatCanWePlay rust library. It will be re-attempted when next launched.")
    
    from .lib.bin import whatcanweplay as wcwp
except Exception as e:
    print("Failed to load WhatCanWePlay rust library. Please go to \"lib/wcwp_rust\" and run \"cargo build --release\"")
    print(e)
    raise e
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
    cache_max_age_dict = config.get("igdb-cache-max-age", config.get("igdb-cache-info-age", {}))
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
    cache_file = config.get("igdb-cache-file")
    if not os.path.isabs(cache_file):
        cache_file = os.path.join(root_path, cache_file)

    # Create uWSGI callable
    app = Flask(__name__)
    app.debug = debug
    app.secret_key = config["secret-key"]

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
    
    # Setup cache info max age
    cache_max_age = 0.0
    if cache_file:
        cache_max_age = timedelta(**cache_max_age_dict).total_seconds()

    print("cookies set to expire after %f seconds" % cookie_max_age)
    print("cache set to expire after %f seconds" % cache_max_age)

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
            response.set_cookie("steam_info", "", secure=True, httponly=True)
            return {}
        
        info = {}
        try:
            info = wcwp.steam.get_steam_users_info(steam_key, [steamid])[0]
        except wcwp.steam.BadWebkeyException:
            response.set_cookie("steam_info", "", secure=True, httponly=True)
            return {}
        except IndexError:
            response.set_cookie("steam_info", "", secure=True, httponly=True)
            return {}

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
            response.set_cookie("steam_info", "", secure=True, httponly=True)
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
            return_url = request.base_url
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
        response.set_cookie("steam_info", "", secure=True, httponly=True)
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
        except Exception:
            traceback.print_exc()
            if debug:
                return (
                    json.dumps({"message": traceback.format_exc(), "errcode": -1}),
                    500
                )
            else:
                traceback.print_exc()
                return (
                    json.dumps({"message": "An unknown error has occurred", "errcode": -1}),
                    500
                )

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

    cache_init_query = """
    CREATE TABLE IF NOT EXISTS game (
        steam_id INTEGER PRIMARY KEY,
        igdb_id INTEGER,
        name STRING,
        supported_players INTEGER DEFAULT(0),
        cover_id STRING,
        has_multiplayer BOOLEAN,
        expiry REAL DEFAULT(0.0)
    );
    """

    CACHE_VERSION = 1

    def initialize_cache():
        import sqlite3
        cache = sqlite3.connect(cache_file)
        cache.execute(cache_init_query)
        #cache.execute("PRAGMA user_version = ?;", [CACHE_VERSION]) # Doesn't work?
        cache.execute("PRAGMA user_version = %d" % CACHE_VERSION)
        return cache
    
    def cache_is_correct_version(cache):
        return cache.execute("PRAGMA user_version;").fetchone()[0] == CACHE_VERSION

    def update_cached_games(game_info):
        if not cache_file:
            return
        
        try:
            import sqlite3
            cache = None

            if os.path.exists(cache_file):
                cache = sqlite3.connect(cache_file)
                # Check if cache is correct version
                if not cache_is_correct_version(cache):
                    # Cache is the wrong version, rebuild
                    print("Cache file is the wrong version! Rebuilding... ")
                    cache.close()
                    os.remove(cache_file)
                    cache = initialize_cache()
            else:
                cache = initialize_cache()
            
            insert_info = [
                [
                    game.get("steam_id"),
                    game.get("igdb_id"),
                    game.get("name"),
                    game.get("supported_players"),
                    game.get("cover_id"),
                    game.get("has_multiplayer"),
                    datetime.now(timezone.utc).timestamp() + cache_max_age
                ] for game in game_info
            ]
            
            cache.executemany(
                "INSERT OR REPLACE INTO game VALUES (?,?,?,?,?,?,?);",
                insert_info
            )

            cache.commit()
            cache.close()
        except Exception:
            print("FAILED TO UPDATE CACHE DB")
            traceback.print_exc()
            return
    
    # returns [info of cached games], (set of uncached ids)
    def get_cached_games(steam_ids):
        if not cache_file:
            return [], set(steam_ids)

        game_info = []
        uncached = set(steam_ids)
        
        try:
            import sqlite3
            cache = sqlite3.connect(cache_file)
            cache.row_factory = sqlite3.Row

            if not cache_is_correct_version(cache):
                return [], set(steam_ids)
            
            query_str = "SELECT * FROM game WHERE steam_id IN (%s)" % ("?" + (",?" * (len(steam_ids) - 1))) # Construct a query with arbitrary parameter length
            
            cursor = cache.execute(
                query_str,
                steam_ids
            )
            for row in cursor.fetchall():
                game = dict(row)
                if datetime.now(timezone.utc).timestamp() < game.pop("expiry"):
                    # Info hasn't expired
                    game_info.append(game)
                    uncached.remove(game["steam_id"])

                # Expired info gets updated during update_cached_games()
        except Exception:
            print("EXCEPTION THROWN WHILE QUERYING GAME CACHE!")
            traceback.print_exc()
            return [], set(steam_ids)
        
        return game_info, uncached
        

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
            game_ids = wcwp.steam.intersect_owned_game_ids(steam_key, list(steamids))

            game_info, uncached_ids = get_cached_games(game_ids)

            fetched_game_count = 0
            cached_game_count = len(game_info)

            if uncached_ids:
                fetched_info, not_found = wcwp.igdb.get_steam_game_info(igdb_key, token, list(uncached_ids))

                cache_info_update = fetched_info
                if not_found:
                    for uncached_id in [id for id in not_found]:
                        cache_info_update.append({"steam_id": uncached_id}) # Cache empty data to prevent further IGDB fetch attempts
                update_cached_games(cache_info_update) # TODO: Spin up separate process for caching?

                game_info += fetched_info
                fetched_game_count = len(fetched_info)
            
            print("Intersection resulted in %d games (%d from cache, %d from IGDB)" % (len(game_info), cached_game_count, fetched_game_count))

            return jsonify({
                "message": "Intersected successfully",
                "games": game_info,
                "errcode": 0
            })
        except Exception:
            traceback.print_exc()
            if debug:
                return (
                    json.dumps({"message": traceback.format_exc(), "errcode": -1}),
                    500
                )
            else:
                return (
                    json.dumps({"message": "An unknown error has occurred", "errcode": -1}),
                    500
                )

    return app