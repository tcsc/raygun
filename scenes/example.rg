camera {
    location: {-25, 0, -30},
    look_at: {0, 0, 0}
}

let white = colour { 1, 1, 1 }
let mid_red = colour { 0.5, 0, 0 }
let dull_green = colour { 0, 0.25, 0 }
let dull_blue = colour { 0, 0, 0.25 }

point_light {
    location: { 100, 0, 0 },
    colour: white
}

point_light {
    location: { 100, 0, -100 },
    colour: mid_red
}

point_light {
    location: { 0, 0, -100 },
    colour: dull_green
}

point_light {
    location: {-100, 0, -100},
    colour: dull_blue
}

sphere {
    centre: { 0, 0, 10 },
    radius: 4
}

{% for y in (0..10) %}
{% for x in (0..10) %}
box {
    lower: { {{ x | times: 2 | minus: 10.5 }}, {{ y | times: 2 | minus: 10.5}}, -0.5},
    upper: { {{ x | times: 2 | minus: 9.5 }}, {{ y | times: 2 | minus: 9.5}}, 0.5}
}
{% endfor %}
{% endfor %}