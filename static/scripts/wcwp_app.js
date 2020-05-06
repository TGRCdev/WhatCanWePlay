var min_desktop_width = 700 // Below this width, switch to mobile

var in_mobile = false;

var games_cell = null;
var users_cell = null;
var submit_cell = null;
var row_1 = null;
var row_2 = null;

var loaded = false;

function getDimensions()
{
    if(!loaded)
        return;
    
    var width = window.innerWidth;
    console.log(width)
    if(width < min_desktop_width && !in_mobile)
    {
        // Switch to mobile
        submit_cell.remove()
        games_cell.remove()

        users_cell.rowSpan = 1
        games_cell.rowSpan = 2

        row_1.appendChild(games_cell)
        row_2.appendChild(submit_cell)

        in_mobile = true;
    }
    else if(width >= min_desktop_width && in_mobile)
    {
        // Switch to desktop
        submit_cell.remove()
        games_cell.remove()

        users_cell.rowSpan = 2;
        games_cell.rowSpan = 1;

        row_1.appendChild(submit_cell)
        row_2.appendChild(games_cell)

        in_mobile = false;
    }
}

var resize_throttled = false;
var resize_delay = 250;

window.addEventListener('resize', function() {
    if(!resize_throttled)
    {
        getDimensions();

        resize_throttled = true;

        this.setTimeout(function() {
            resize_throttled = false;
        }, resize_delay)
    }
})

window.addEventListener('load', function() {
    games_cell = document.getElementById("games-cell")
    users_cell = document.getElementById("users-cell")
    submit_cell = document.getElementById("submit-cell")
    row_1 = document.getElementById("app-row-1")
    row_2 = document.getElementById("app-row-2")

    loaded = true;
    this.getDimensions();
})