var user = {};

function logOut() {
    document.cookie = "loggedIn=false; samesite=Strict; max-age=0"
    location.reload()
}

async function setupLoggedInVariables() {
    user["elems"] = {
        div: document.getElementById("main-user"),
        image: document.getElementById("main-user-img"),
        name: document.getElementById("main-user-name"),
    }
}

async function loadUser(setup) {
    user["steam_id"] = window.localStorage.getItem("steam_id")
    if(user["steam_id"] == null)
    {
        logOut()
    }

    user["screen_name"] = window.localStorage.getItem("screen_name")
    if(user["screen_name"] == null)
    {
        logOut()
    }

    user["avatar"] = window.localStorage.getItem("avatar")
    if(user["screen_name"] == null)
    {
        logOut()
    }

    if(setup)
    {
        await setup
    }

    user["elems"]["image"].src = user["avatar"]
    user["elems"]["name"].innerText = user["screen_name"]
}

window.addEventListener("load", async (event) => {
    let setup = setupLoggedInVariables()
    await loadUser(setup)
})