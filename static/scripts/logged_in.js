var main_user = {};
var users = {};

var selected_users = [];

function logOut() {
    document.cookie = "loggedIn=false; samesite=Strict; max-age=0"
    location.reload()
}

/// Setup variables in global scope
async function setupLoggedInVariables() {
    main_user["elems"] = {
        div: document.getElementById("main-user"),
        image: document.getElementById("main-user-img"),
        name: document.getElementById("main-user-name"),
    }
}

/// Retrieve the main user's info from localStorage.
/// If the Steam ID is missing, user has to log back
/// in.
///
/// If other info is missing, this function will be called again
/// when the user's info is fetched from the server
async function loadUserFromLocalStorage(setup) {
    main_user["steam_id"] = window.localStorage.getItem("steam_id")
    if(main_user["steam_id"] == null)
    {
        logOut()
    }

    main_user["screen_name"] = window.localStorage.getItem("screen_name")
    if(main_user["screen_name"] == null)
    {
        main_user["screen_name"] = ""
    }
    main_user["avatar"] = window.localStorage.getItem("avatar")
    if(main_user["avatar"] == null)
    {
        main_user["avatar"] = "/static/images/default_avatar.png"
    }

    await refreshUser(setup)
}

/// Update the main user's name and image
async function refreshUser(setup) {
    if(setup)
    {
        await setup
    }

    let img = main_user["elems"]["image"]
    img.src = main_user["avatar"]
    img.classList = ["user-img"]
    main_user["elems"]["name"].innerText = main_user["screen_name"]
    if(main_user["gameid"])
    {
        img.classList.add("ingame")
    }
    else
    {
        switch(main_user["user_state"]) {
            case 1: // Online
            case 3: // Away
            case 4: // Snooze
            case 5: // Looking to trade
            case 6: // Looking to play
                img.classList.add("online")
                break
            case 0: break // Offline
            case 2: // Busy
            default:
                break
        }
    }
}

/// Called after retrieving info for the main user
/// and friends.
async function updateMainUser() {
    let steam_id = main_user["steam_id"]
    let user_info = users[steam_id]
    if(user_info)
    {
        window.localStorage.setItem("avatar", user_info["avatar"])
        window.localStorage.setItem("screen_name", user_info["screen_name"])
    }
    main_user = {
        ...main_user,
        ...user_info
    }
    refreshUser()
}

/// Called during initialization to fetch the steam info
/// of the main user and friends. Will also update the
/// main user's info.
async function fetchFriends() {
    const user_response = await fetch("api/get_user_and_friends_info", {
        method: "POST",
        mode: "same-origin",
        cache: "no-cache",
        body: main_user["steam_id"]
    })

    switch(user_response.status) {
        case 200: {
            users = await user_response.json()
            updateMainUser()
            setUpUserObjects()
            break
        }
        default: {
            // TODO
            showError(await user_response.text())
            break
        }
    }
}

/// Loops through the users table, creates 
/// user elements and puts them into the users'
/// respective objects under "elems"
async function setUpUserObjects() {
    let friend_list = document.getElementById("friends")
    //friend_list.innerText = "" // Clear all children

    for (const [steam_id, user] of Object.entries(users)) {
        if(user["elems"] != null)
            continue
        
        let name = document.createElement("div")
        name.classList = ["user-name"]
        name.innerText = user["screen_name"]

        let img = document.createElement("img")
        img.classList.add("user-img")
        img.src = user["avatar"]
        if(user["current_game"])
        {
            img.classList.add("ingame")
        }
        else
        {
            switch(user["online_status"]) {
                
                case 1: // Online
                case 3: // Away
                case 4: // Snooze
                case 5: // Looking to trade
                case 6: // Looking to play
                    img.classList.add("online")
                    break
                case 0: break // Offline
                case 2: // Busy
                default:
                    break
            }
        }

        let button = document.createElement("button")
        button.className = "user-checkbox"
        button.addEventListener("click", () => userCheckboxClicked(user["steam_id"]))
        // TODO: Disable if a user is private
        let button_fill = document.createElement("div")
        button_fill.className = "user-checkbox-fill"
        button_fill.style.display = "none"
        button.appendChild(button_fill)

        let user_div = document.createElement("div")
        user_div.classList.add("user")
        user_div.classList.add("list-item")
        if (user["steam_id"] == main_user["steam_id"])
        {
            user_div.id = "main-user"
            name.innerHTML = "<div>" + user["screen_name"] + "</div><a href='/logout'>Sign out</a>"
            main_user["elems"]["div"].parentNode.replaceChild(user_div, main_user["elems"]["div"])
        }
        else
        {
            friend_list.appendChild(user_div)
        }
        user_div.appendChild(img)
        user_div.appendChild(name)
        user_div.appendChild(button)

        users[steam_id]["elems"] = {
            img: img,
            name: name,
            button: button,
            button_fill: button_fill,
            div: user_div,
        }
    }
}

function userCheckboxClicked(steam_id) {
    let user = users[steam_id]
    switch(user["elems"]["button_fill"].style.display) {
        case "none":
            user["elems"]["button_fill"].style.display = "block"
            selected_users.push(user)
            break
        default:
            user["elems"]["button_fill"].style.display = "none"
            let index = selected_users.findIndex((elem) => elem["steam_id"] == steam_id)
            if(index) {
                selected_users.splice(index, 1)
            }
            break
    }
    console.log(selected_users)
}

window.addEventListener("load", async (event) => {
    let setup = setupLoggedInVariables()
    await loadUserFromLocalStorage(setup)
    await fetchFriends()
    setUpUserObjects()
})