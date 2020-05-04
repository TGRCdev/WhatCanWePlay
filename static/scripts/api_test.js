function createTypeInput(type)
{
    var input;
    switch(type)
    {
        case "csl":
            input = document.createElement("input")
            input.placeholder = "comma,separated,values"
            input.type = "text"
            break;
        case "string":
            input = document.createElement("input")
            input.placeholder = "string"
            input.type = "text"
            break;
        case "int":
            input = document.createElement("input")
            input.type = "number"
            input.placeholder = "int"
        case "bool":
            input = document.createElement("input")
            input.type = "checkbox"
        default:
            input = document.createElement("input")
            input.type = "text"
    }

    return input;
}

var params_data;

window.onload = function() {
    var function_data = document.getElementById("function-data").dataset
    this.console.log(function_data.functionParams)
    params_data = JSON.parse(function_data.functionParams)
    this.console.log(typeof(params_data))
    var param_table = document.getElementById("function-params-table")

    var type_strings = {
        "csl": "comma-separated list"
    }

    for(var i = 0; i < params_data.length; i++)
    {
        var param = params_data[i]
        this.console.log(param)
        var row = param_table.insertRow()
        var type = param["type"]
        var name = param["name"]
        var type_str = type_strings[type]
        if(type_str == null)
        {
            type_str = type
        }
        var name_cell = row.insertCell()
        name_cell.innerHTML = name
        var type_cell = row.insertCell()
        type_cell.innerHTML = type_str
        var value_cell = row.insertCell()
        var value_input = createTypeInput(type)
        value_input.id = name
        value_cell.appendChild(value_input)
    }

    var submit_row = param_table.insertRow()
    var submit_cell = submit_row.insertCell()
    submit_cell.colSpan = 3
    submit_cell.style = "text-align:center;"
    var submit_button = document.createElement("button")
    submit_button.style = "display:inline-block;"
    submit_button.innerHTML = "Submit POST Request"
    submit_button.onclick = getAndSubmit
    submit_button.id = "submit-request"
    submit_cell.appendChild(submit_button)
}

function responseReceived(response)
{
    var response_box = document.getElementById("response")
    var submit_button = document.getElementById("submit-request")
    response.text().then(function(text){response_box.innerHTML = text})
    // i hate futures so much
    submit_button.disabled = false;
}

function getAndSubmit()
{
    var submit_button = document.getElementById("submit-request")
    var response_box = document.getElementById("response")
    response_box.placeholder = "Submitting request..."
    response_box.innerHTML = ""
    submit_button.disabled = true;

    var request_data = {}
    for(var i = 0; i < params_data.length; i++)
    {
        var param = params_data[i]
        var name = param["name"]
        var type = param["type"]
        var elem = document.getElementById(name)
        var value = elem.value
        switch(type)
        {
            case "int":
                value = parseInt(value)
                break;
            case "csl":
                var list = value.split(",")
                for(var j = 0; j < list.length; j++)
                {
                    list[j] = list[j].trim()
                }
                value = list
                break;
            default:
                break;
        }

        request_data[name] = value
    }

    response_box.placeholder = "Waiting for response..."
    console.log(JSON.stringify(request_data))
    fetch("", {
        method: 'post',
        body: JSON.stringify(request_data)
    }).then(responseReceived)
}