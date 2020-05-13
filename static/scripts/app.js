var friends;
var user_template;
var search_box;
var submit;
var app;
var back;
var games;

var user_info = {}

var selected_users = new Set()

var current_slide_timeout;

var fetching = false;

window.addEventListener("load", function() {
    submit = document.getElementById("submit-button");
    submit.addEventListener("click", submitButtonClicked);
    back = document.getElementById("back-button");
    back.addEventListener("click", backButtonClicked);
    app = document.getElementById("app");
    friends = document.getElementById("friends");
    games = document.getElementById("games")
    default_avatar_url = friends.dataset.defaultAvatar;
    main_user = document.getElementById("main-user");
    user_template = main_user.cloneNode(true);
    this.user_template.id = "";

    for(var i = 0; i < user_template.children.length; i++)
    {
        var child = user_template.children[i];
        switch(child.className)
        {
            case "user-img":
                child.src = default_avatar_url;
                break;
            case "user-name":
                child.innerHTML = "";
                break;
            case "user-checkbox":
                delete child.dataset.steamId;
                break;
        }
    }

    for(var i = 0; i < main_user.children.length; i++)
    {
        var child = main_user.children[i];
        if(child.className == "user-checkbox")
        {
            var vis = main_user.dataset.visibility;
            if(vis != "3")
            {
                delete child.dataset.steamId;
                child.classList.add("inactive");
                child.title = "Your Steam profile visibility is set to "
                + (vis == 1 ? "Private" : "Friends Only")
                + ", and cannot be retrieved by this app.";
            }
            else
            {
                userCheckboxClicked(child);
            }
        }
    }

    // Fetch friends list
    timeout(fetch(
        "/api/v1/get_friend_list", {method: "post"}
    ), 10000).then((response) => response.json()).then(friendsFetched)//.catch(apiTimeout)

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
    fetching = true;
    submit.disabled = true;
    submit.innerHTML = "Fetching..."
    back.disabled = true;
    back.innerHTML = "Fetching..."

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

    games_fetch = timeout(fetch(
        "/api/v1/intersect_owned_games", {
            method: "post",
            body: JSON.stringify(body)
        }
    ), 30000)

    games_fetch.then((response) => response.json()).then(function(data) {
        games.innerHTML = JSON.stringify(data)
    }).catch(apiError).finally(function() {
        submit.disabled = false;
        submit.innerHTML = "Find Games";
        back.disabled = false;
        back.innerHTML = "Back";
        fetching = false;
    })
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
    console.error("Backend API returned an error: " + String(error));
    // TODO
}

function friendsFetched(data)
{
    timeout(fetch(
        "/api/v1/get_steam_user_info",
        {
            method: "post",
            body: JSON.stringify({
                steamids: data
            })
        }
    ), 10000).then((response) => response.json()).then(friendDataFetched).catch(apiError);
}

function friendDataFetched(data)
{
    data = Object.values(data);

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
                    child.innerHTML = user["screen_name"];
                    break;
                case "user-checkbox":
                    if(user["visibility"] != 3)
                    {
                        //child.onclick = null;
                        child.title = "This user's Steam profile visibility is set to "
                        + (user["visibility"] == 1 ? "Private" : "Friends Only")
                        + ", and cannot be retrieved by this app.";
                        child.classList.add("inactive");
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
                fill.style.display = "block"
                selected_users.add(steamid)
            }

            len = selected_users.size;
            if(len >= 2)
            {
                submit.disabled = false;
                submit.innerHTML = "Find Games"
            }
            else
            {
                submit.disabled = true;
                submit.innerHTML = "Select " + (len == 0 ? "Two Users" : "One User")
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