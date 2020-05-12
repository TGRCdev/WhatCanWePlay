var friends;
var user_template;
var search_box;

window.addEventListener("load", function() {
    friends = document.getElementById("friends")
    default_avatar_url = friends.dataset.defaultAvatar;
    user_template = document.getElementById("main-user").cloneNode(true)
    this.user_template.id = ""

    for(var i = 0; i < user_template.children.length; i++)
    {
        var child = user_template.children[i]
        user_template.dataset.steamId = ""
        switch(child.className)
        {
            case "user-img":
                child.src = default_avatar_url
                break;
            case "user-name":
                child.innerHTML = ""
                break;
        }
    }

    // Fetch friends list
    timeout(fetch(
        "/api/v1/get_friend_list", {method: "post"}
    ), 10000).then((response) => response.json()).then(friendsFetched)//.catch(apiTimeout)

    search_box = document.getElementById("user-search")
    search_box.addEventListener("keyup", function(event) {
        if (event.keyCode == 13) {
            searchFriends(search_box.value)
        }
    })
})

function apiTimeout()
{
    console.error("Couldn't connect to the backend. Please try again later.")
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
    ), 10000).then((response) => response.json()).then(friendDataFetched)//.catch(apiTimeout)
}

function friendDataFetched(data)
{
    data = Object.values(data)

    data.sort(
        (a, b) => {
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
            return a["screen_name"].localeCompare(b["screen_name"])
        }
    )

    data.forEach(function(user) {
        if(!user["exists"])
        {
            return;
        }

        var user_div = user_template.cloneNode(true)
        user_div.dataset.steamId = user["steam_id"]
        user_div.dataset.name = user["screen_name"]
        Array.from(user_div.children).forEach(function(child) {
            switch(child.className)
            {
                case "user-img":
                    child.src = user["avatar"]
                    if(!user["online"])
                    {
                        child.classList.add("offline")
                    }
                    break;
                case "user-name":
                    child.innerHTML = user["screen_name"]
                    break;
            }
        });
        friends.appendChild(user_div)
    });
}

function searchFriends(search_str)
{
    search_str = search_str.trim()

    for(var i = 0; i < friends.children.length; i++)
    {
        var user = friends.children[i]
        var name = user.dataset.name;
        if(!search_str || search_str.length == 0)
        {
            user.style.display = "flex"
        }
        else
        {
            if(name.toLowerCase().includes(search_str))
            {
                user.style.display = "flex"
            }
            else
            {
                user.style.display = "none"
            }
        }
    }
}