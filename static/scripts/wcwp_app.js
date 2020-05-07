var app;
var submit;

function submit_clicked()
{
    scroll_to_games()
}

function scroll_to_games()
{
    app.className = "scroll-to-games"
}

function scroll_to_users()
{
    app.className = "scroll-to-users"
}

window.addEventListener('load', function() {
    app = document.getElementById("app")
    submit = document.getElementById("submit")

    submit.addEventListener("click", submit_clicked)
})