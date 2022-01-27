var id_box;
var submit_id;

function setupLoggedOutVariables() {
    submit_id = document.getElementById("submit-id")
    id_box = document.getElementById("id-box")
    submit_id.disabled = (id_box.value.length === 0)
    id_box.addEventListener("input", (event) => {
        submit_id.disabled = (id_box.value.length === 0)
    })
}

window.addEventListener("load", (event) => {
    setupLoggedOutVariables()
})

async function submitId() {
    submit_id.disabled = true
    id_box.disabled = true

    let id = id_box.value
    if(id != undefined && id.length > 0)
    {
        const response = await fetch("api/interpret_id_input", {
            method: "POST",
            mode: "same-origin",
            body: id,
        })
        console.log(response)

        switch(response.status) {
            case 200: {
                const json = await response.json()
                window.localStorage.setItem("steam_info", json)
                document.cookie = "loggedIn=true; samesite=Strict; max-age=31536000"
                location.reload()
                break
            }
            case 404: {
                showError(`
                    The id you entered couldn't be resolved. Please enter one of the following:
                    <ul>
                        <li>Your Steam profile URL</li>
                        <li>Your SteamID64 number</li>
                        <li>Your vanity url ID</li>
                    </ul>
                `)
                submit_id.disabled = false
                id_box.disabled = false
                break
            }
            default: {
                showError(`
                    An unknown error occurred while interpreting the id you entered. Please try again later.
                `)
                submit_id.disabled = false
                id_box.disabled = false
                break
            }
        }
    }
}