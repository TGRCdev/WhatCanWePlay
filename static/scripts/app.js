var friends;
var user_template;
var game_template;
var search_box;
var submit;
var app;
var back;
var games;
var users_cover;
var error_div;
var error_info;

var user_info = {}

var selected_users = new Set()

var current_slide_timeout;

var fetching = false;

var main_user_id = 0;

window.addEventListener("load", function() {
    submit = document.getElementById("submit-button");
    submit.addEventListener("click", submitButtonClicked);
    back = document.getElementById("back-button");
    back.addEventListener("click", backButtonClicked);
    app = document.getElementById("app");
    friends = document.getElementById("friends");
    games = document.getElementById("games");
    users_cover = document.getElementById("users-cover");
    error_div = document.getElementById("error-div")
    error_info = document.getElementById("error-info")
    default_avatar_url = friends.dataset.defaultAvatar;
    main_user = document.getElementById("main-user");
    user_template = main_user.cloneNode(true);
    this.user_template.id = "";
    game_template = this.document.getElementById("game-template")
    game_template.remove()
    game_template.id = ""

    for(var i = 0; i < user_template.children.length; i++)
    {
        var child = user_template.children[i];
        switch(child.className)
        {
            case "user-img":
                child.src = default_avatar_url;
                break;
            case "user-name":
                child.innerText = "";
                break;
            case "user-checkbox":
                delete child.dataset.steamId;
                break;
        }
    }

    var main_user_info = {}

    for(var i = 0; i < main_user.children.length; i++)
    {
        var child = main_user.children[i];
        if(child.className == "user-checkbox")
        {
            var vis = main_user.dataset.visibility;
            if(vis != "3")
            {
                delete child.dataset.steamId;
                child.disabled = true;
                child.title = "Your Steam profile visibility is set to "
                + (vis == 1 ? "Private" : "Friends Only")
                + ", and cannot be retrieved by this app.";
            }
            else
            {
                userCheckboxClicked(child);
                main_user_id = child.dataset.steamId;
                main_user_info["steam_id"] = main_user_id;
            }
        }
        else if(child.className == "user-name")
        {
            main_user_info["screen_name"] = child.children[0].innerText;
        }
    }

    if(main_user_id != 0)
    {
        user_info[main_user_id] = main_user_info
    }

    // Fetch friends list
    timeout(fetch(
        "/api/v1/get_friend_list", {method: "post"}
    ), 10000).then((response) => {
        if(response.status == 200)
        {
            response.json().then(friendDataFetched)
        }
        else
        {
            response.text().then(apiError)
        }
    }).catch(apiError)

    search_box = document.getElementById("user-search");
    search_box.addEventListener("keyup", function(event) {
        if (event.keyCode == 13) {
            search_box.blur();
            searchFriends(search_box.value);
        }
    });
})

function submitButtonClicked()
{
    error_div.style.display = "none"
    fetching = true;
    submit.disabled = true;
    submit.innerText = "Fetching..."
    back.disabled = true;
    back.innerText = "Fetching..."
    users_cover.style.display = "block";

    if(app.className.includes("on-users") || app.className.includes("slide-to-users"))
    {
        app.classList.remove("on-users");
        app.classList.remove("slide-to-users");
        app.classList.add("slide-to-games");
        clearTimeout(current_slide_timeout);
        current_slide_timeout = setTimeout(
            function() {
                app.classList.remove("slide-to-games")
                app.classList.add("on-games")
            },
        600);
    }

    body = {
        steamids: Array.from(selected_users)
    }

    Array.from(games.children).forEach(function(child) { // Clear old results
        if(child.id != "error-div")
        {
            child.remove();
        }
    });

    games_fetch = timeout(fetch(
        "/api/v1/intersect_owned_games", {
            method: "post",
            body: JSON.stringify(body)
        }
    ), 30000)

    games_fetch.then((response) => response.json())
    .then(intersectResponse)
    .catch(apiError)
    .finally(function() {
        submit.disabled = false;
        submit.innerText = "Find Games";
        back.disabled = false;
        back.innerText = "Back";
        fetching = false;
        users_cover.style.display = "none"
    })
}

function intersectResponse(data) {
    if(data["errcode"] == 1)
    { // User has non-visible games list
        displayError("WhatCanWePlay cannot access the games list of <span class='err-user-name'>" + user_info[data["user"]]["screen_name"] + "</span>. This either means that their Game details visibility is not Public, or they are being rate-limited by Steam for having too many requests. You can try one of the following fixes:\
        <br><br>- Ask <span class='err-user-name'>" + user_info[data["user"]]["screen_name"] + "</span> to set their Game details to Public\
        <br>- Remove <span class='err-user-name'>" + user_info[data["user"]]["screen_name"] + "</span> from your selected users\
        <br>- Try again later\
        ");
        return;
    }
    else if(data["errcode"] == 2)
    { // User has empty games list
        displayError("<span class='err-user-name'>" + user_info[data["user"]]["screen_name"] + "</span> has an empty games list, and cannot possibly share any common games with the selected users. Please deselect <span class='err-user-name'>" + user_info[data["user"]]["screen_name"] + "</span> and try again.")
        return;
    }
    else if(data["errcode"] != 0)
    {
        displayError(data["message"]);
        return;
    }
    else
    {
        gameinfo = Array.from(data["games"])

        if(gameinfo.length == 0)
        {
            displayError("Looks like these users don't have any games shared between all of them.")
        }

        // Sort data
        // TODO: Filter options
        gameinfo.sort(function(a, b) {
            // This is intentionally a weak sort. It sorts games into three sections in descending order:
            //
            // Top: Games with supported user counts above the selected user count (intentionally unordered)
            // Middle: Games with unknown supported user counts (Could be above, could be below?)
            // Bottom: Games below the selected user count
            //
            // The Top section is intentionally unsorted because we aren't looking for games with the highest
            // player count, we're looking for a game to play with friends.
            a_val = a["supported_players"]
            b_val = b["supported_players"]

            a_compval = (a_val == "?" ? selected_users.size : parseInt(a_val))
            b_compval = (b_val == "?" ? selected_users.size : parseInt(b_val))

            if(a_val == "?")
            {
                if(b_val == "?")
                {
                    return 0;
                }
                else if(b_compval < selected_users.size)
                {
                    return -1;
                }
                else
                {
                    return 1;
                }
            }
            else if(b_val == "?")
            {
                if(a_compval < selected_users.size)
                {
                    return 1;
                }
                else
                {
                    return -1;
                }
            }
            else
            {
                if(a_compval < selected_users.size)
                {
                    if(b_compval < selected_users.size)
                    {
                        return 0;
                    }
                    else
                    {
                        return 1;
                    }
                }
                else if(b_compval < selected_users.size)
                {
                    return -1;
                }
                else
                {
                    return 0;
                }
            }
        })

        gameinfo.forEach(function(game) { // Render new results
            if(!game["name"])
            {
                return;
            }

            gamediv = game_template.cloneNode(true);
            Array.from(gamediv.children).forEach(function(child) {
                switch(child.className)
                {
                    case "game-cover":
                        if(game["cover_id"])
                        {
                            child.src = "https://images.igdb.com/igdb/image/upload/t_cover_small/" + game["cover_id"] + ".jpg"
                        }
                        break;
                    case "game-title":
                        child.innerText = game["name"]
                        break;
                    case "user-count":
                        Array.from(child.children).forEach(function(child) {
                            if(child.className == "user-number")
                            {
                                child.innerText = game["supported_players"]
                                if(game["supported_players"] == "0")
                                {
                                    child.innerText = "?"
                                    child.classList.add("short")
                                    child.title = "WhatCanWePlay was unable to retrieve the player count for this game from the IGDB"
                                }
                                else if(game["supported_players"] == "1")
                                {
                                    child.classList.add("short")
                                    child.title = "This game is singleplayer"
                                }
                                else if(game["supported_players"] < selected_users.size)
                                {
                                    child.classList.add("short")
                                    child.title = "This game has less supported users than the number of selected users"
                                }
                            }
                        });
                        break;
                }
            });
            games.appendChild(gamediv)
        });
    }
}

function displayError(message) {
    error_div.style.display = "flex";
    error_info.innerHTML = message;
}

function backButtonClicked()
{
    if(app.className.includes("on-games") || app.className.includes("slide-to-games"))
    {
        app.classList.remove("on-games");
        app.classList.remove("slide-to-games");
        app.classList.add("slide-to-users");
        clearTimeout(current_slide_timeout);
        current_slide_timeout = setTimeout(
            function() {
                app.classList.remove("slide-to-users")
                app.classList.add("on-users")
            },
        600);
    }
}

function apiError(error)
{
    displayError(String(error));
    // TODO
}

function friendDataFetched(data)
{
    data = Object.values(data);

    data = data.filter(function(user) {
        if(
            !user["exists"] ||
            !user["screen_name"] ||
            !user["avatar"]
        )
        {
            console.error("user with steam id " + String(user["steam_id"]) + " is missing info. dropping from list")
            return false;
        }

        return true;
    })

    if(data.length == 0)
    {
        displayError("Your Friend List is empty! You need at least one friend to compare games with!")
        return;
    }

    data.sort(
        (a, b) => {
            if(a["visibility"] == 3)
            {
                if(b["visibility"] != 3)
                {
                    return -1
                }
            }
            else if(b["visibility"] == 3)
            {
                return 1;
            }

            if(a["online"])
            {
                if(!b["online"])
                {
                    return -1;
                }
            }
            else if(b["online"])
            {
                return 1;
            }

            return a["screen_name"].localeCompare(b["screen_name"]);
        }
    );

    data.forEach(function(user) {
        if(!user["exists"])
        {
            return;
        }

        var user_div = user_template.cloneNode(true);
        user_div.dataset.steamId = user["steam_id"];
        user_div.dataset.name = user["screen_name"];
        user["div"] = user_div;
        Array.from(user_div.children).forEach(function(child) {
            switch(child.className)
            {
                case "user-img":
                    child.src = user["avatar"];
                    if(!user["online"])
                    {
                        child.classList.add("offline");
                    }
                    break;
                case "user-name":
                    child.innerText = user["screen_name"];
                    break;
                case "user-checkbox":
                    if(user["visibility"] != 3)
                    {
                        //child.onclick = null;
                        child.title = "This user's Steam profile visibility is set to "
                        + (user["visibility"] == 1 ? "Private" : "Friends Only")
                        + ", and cannot be retrieved by this app.";
                        child.disabled = true;
                    }
                    else
                    {
                        child.dataset.steamId = user["steam_id"];
                    }
                    break;
            }
        });
        friends.appendChild(user_div);
        user_info[user["steam_id"]] = user;
    });
}

function userCheckboxClicked(box)
{
    steamid = box.dataset.steamId
    if(steamid)
    {
        if(!fetching)
        {
            fill = box.children[0]
            if(selected_users.has(steamid))
            {
                fill.style.display = "none"
                selected_users.delete(steamid)
            }
            else
            {
                if(selected_users.size < 10)
                {
                    fill.style.display = "block"
                    selected_users.add(steamid)
                }
                else
                {
                    alert("Only 10 users can be intersected at a time.")
                }
            }

            len = selected_users.size;
            if(len >= 2)
            {
                submit.disabled = false;
                submit.innerText = "Find Games"
            }
            else
            {
                submit.disabled = true;
                submit.innerText = "Select " + (len == 0 ? "Two Users" : "One User")
            }
        }
        else
        {
            alert("Cannot modify selected users while fetching")
        }
    }
    else
    {
        alert(box.title)
    }
}

function searchFriends(search_str)
{
    search_str = search_str.trim().toLowerCase()

    for(var i = 0; i < friends.children.length; i++)
    {
        var user = friends.children[i]
        var name = user.dataset.name;
        if(!search_str || search_str.length == 0 || name.toLowerCase().includes(search_str))
        {
            user.style.display = "flex"
        }
        else
        {
            user.style.display = "none"
        }
    }
}