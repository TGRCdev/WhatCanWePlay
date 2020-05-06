min_desktop_width = 361 // Below this width, switch to mobile

var games_cell

window.onload = function() {
    this.getDimensions();
}

function getDimensions()
{
    var width = window.innerWidth;
    var height = window.innerHeight;
    // TODO: Add layout-switching code
}

var resize_throttled = false;
var resize_delay = 333;

window.addEventListener('resize', function() {
    if(!resize_throttled)
    {
        getDimensions();

        this.setTimeout(function() {
            resize_throttled = false;
        }, resize_delay)
    }
})