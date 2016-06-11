"use strict";

fetch('data/data.js')
    .then(checkStatus)
    .then(parseJSON)
    .then(presentData)
    .catch(presentFailure);

function checkStatus(response) {
    if (response.ok) {
        return response
    } else {
       var error = new Error(response.statusText)
        error.response = response
        throw error
    }
}

function parseJSON(response) {
    return response.json()
}

function presentData(json) {
    var itemNames = _.chain(json.suites)
        .pluck('items')
        .flatten()
        .sortBy('name')
        .pluck('name')
        .uniq(true /* isSorted */)
        .value();

    var colorsClone = chartColors.slice(0);

    var datasets = _.chain(json.suites)
        .sortBy('name')
        .map(function(suite) {
            var colors = colorsClone.shift();
            var dataset = {
                label: suite.name,
                data: [],
                backgroundColor: colors.backgroundColor,
                borderColor: colors.borderColor,
                borderWidth: 1,
            };
            for (var itemName of itemNames) {
                var item = _.find(suite.items, function(item) { return item.name === itemName; });
                if (item) {
                    dataset.data.push(item.average_ns);
                } else {
                    dataset.data.push(undefined);
                }
            }
            return dataset;
        })
        .value();

    document.getElementById('data-view').innerHTML =
        '<canvas id="chart-view" height="4000" width="1000"></canvas>';

    var chartCtx = document.getElementById('chart-view');

    new Chart(chartCtx, {
        type: 'horizontalBar',
        data: {
            labels: itemNames,
            datasets: datasets,
        },
        options: {
            scales: {
                xAxes: [{
                    ticks: {
                        beginAtZero:true
                    },
                    scaleLabel: {
                        display: true,
                        labelString: "Average ns per iteration",
                    },
                }],
            },
        },
    });
}

function presentFailure(ex) {
    // TODO: escape, test, clean up
    document.getElementById('fetch-failure').innerHTML =
        '<p>Failed!</p>' +
        '<p>Details: ' + ex + '</p>';
}

var chartColors = [
    {
        backgroundColor: 'rgba(255, 99, 132, 0.2)',
        borderColor: 'rgba(255,99,132,1)',
    },
    {
        backgroundColor: 'rgba(54, 162, 235, 0.2)',
        borderColor: 'rgba(54, 162, 235, 1)',
    },
    {
        backgroundColor: 'rgba(255, 206, 86, 0.2)',
        borderColor: 'rgba(255, 206, 86, 1)',
    },
    {
        backgroundColor: 'rgba(75, 192, 192, 0.2)',
        borderColor: 'rgba(75, 192, 192, 1)',
    },
    {
        backgroundColor: 'rgba(153, 102, 255, 0.2)',
        borderColor: 'rgba(153, 102, 255, 1)',
    },
    {
        backgroundColor: 'rgba(255, 159, 64, 0.2)',
        borderColor: 'rgba(255, 159, 64, 1)',
    },
];
