/**
 * This file is a part of WhatCanWePlay
 * Copyright (C) 2020 TGRCDev
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 * 
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 * 
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

var fields = {}

var raw_box;
var response_box;

function showTab(tabname)
{
    var tabs = document.getElementsByClassName("tab");
    Array.from(tabs).forEach(function(tab) {
        if(tab.id == tabname + "-tab")
        {
            tab.classList.add("selected");
        }
        else
        {
            tab.classList.remove("selected");
        }
    });

    var tabcontents = document.getElementsByClassName("tab-content");
    Array.from(tabcontents).forEach(function(tab) {
        if(tab.id == tabname)
        {
            tab.classList.add("active");
        }
        else
        {
            tab.classList.remove("active");
        }
    });
}

function createInput(type)
{
    var input;
    switch(type)
    {
        case "csl:string":
            input = document.createElement("input")
            input.placeholder = "comma,separated,strings"
            input.type = "text"
            break;
        case "csl:int":
            input = document.createElement("input")
            input.placeholder = "123,456,789"
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
            break;
        case "bool":
            input = document.createElement("input")
            input.type = "checkbox"
            break;
        default:
            input = document.createElement("input")
            input.type = "text"
            break;
    }

    return input;
}

function getTypeString(type)
{
    switch(type)
    {
        case "csl:string":
            return "comma-separated strings";
        case "csl:int":
            return "comma-separated ints";
        default:
            return type;
    }
}

function getInputValue(input, type)
{
    switch(type)
    {
        case "csl:string":
        case "csl:int": // The endpoints should take ints as strings too
            var strings = []
            var input_txt = input.value;
            input_txt.split(",").forEach(function(str) {
                strings.push(str)
            });
            return strings;
        case "bool":
            return input.checked;
        default:
            return input.value;
    }
}

function setDefaultValue(input, param)
{
    if(!param || param["default"] == undefined)
    {
        return;
    }

    switch(param["type"])
    {
        case "bool":
            input.checked = param["default"]
            break;
        default:
            input.value = param["default"]
            break;
    }
}

function inputIsBlank(input, param)
{
    if(param["default"] != undefined && getInputValue(input, param["type"]) == param["default"])
    {
        return true;
    }

    switch(param["type"])
    {
        case "bool":
            return false;
        default:
            return !input.value;
    }
}

window.addEventListener("load", function() {
    var params = document.getElementById("field-table");
    raw_box = document.getElementById("raw-input");
    response_box = document.getElementById("response");

    if(param_info && param_info.length > 0)
    {
        document.getElementById("no-params").remove();

        param_info.forEach(function(param) {
            var name = document.createElement("div");
            name.innerHTML = param["name"];
            params.appendChild(name);

            var type = document.createElement("div");
            type.innerHTML = getTypeString(param["type"]);
            params.appendChild(type);

            var input = createInput(param["type"]);
            param["input"] = input;
            fields[param["name"]] = param;

            if(param["type"] == "bool")
            {
                var container = document.createElement("div");
                container.appendChild(input);
                params.appendChild(container);
            }
            else
            {
                params.appendChild(input);
            }

            setDefaultValue(input, param);
        });
    }
});

function fieldsToJson()
{
    var info_dict = {};

    Object.values(fields).forEach(function(param) {
        var name = param["name"];
        var type = param["type"];
        var input = param["input"];
        if(!inputIsBlank(input, param))
        {
            info_dict[name] = getInputValue(input, type);
        }
    });

    return JSON.stringify(info_dict, null, 2);
}

function fieldsToRaw()
{
    raw_box.value = fieldsToJson();
    showTab("raw");
}

function submitRequest(body)
{
    response_box.value = "";
    response_box.placeholder = "Fetching response..."

    timeout(
        fetch("", {
            method: 'post',
            body: body
        })
    , 30000).then((response) => response.text()).then((text) => {
        response_box.value = text;
    }).catch((err) => {
        response_box.value = err;
    });
}

function submitFields()
{
    submitRequest(fieldsToJson());
}

function submitRaw()
{
    submitRequest(raw_box.value);
}