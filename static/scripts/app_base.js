var error_div;
var error_info;

function setupBaseVariables()
{
    error_div = document.getElementById("error-div")
    error_info = document.getElementById("error-info")
}

window.addEventListener("load", (event) => {
    setupBaseVariables()
})

function showError(error_string)
{
    error_info.innerHTML = "<div>" + error_string + "</div>"
    error_div.style.removeProperty("display")
}

function hideError()
{
    error_div.style.display = "none"
}