camera {
    location: {-15, 0, -30},
    look_at: {-4, 0, 0}
}

{% assign white = '{ 1, 1, 1 }' %}
{% assign mid_red = '{ 0.5, 0, 0 }' %}
{% assign dull_green = '{ 0, 0.25, 0 }' %}
{% assign dull_blue = '{ 0, 0, 0.25 }' %}

point_light {
    location: { 100, 100, -100 },
    colour: {{ white }}
}

point_light {
    location: { 0, 0, -100 },
    colour: {0.25, 0.25, 0.25}
}

sphere {
    centre: { -12, 0, 20 },
    radius: 6,
    material: {
        pigment: solid { colour: {1, 0, 0} },
        finish: { reflection: 0.5 }
    }
}

sphere {
    centre: { 0, 0, 20 },
    radius: 6,
    material: {
        pigment: solid { colour: {0.5, 0.5, 0.5} },
        finish: { reflection: 0.9 }
    }
}

sphere {
    centre: { 12, 0, 20 },
    radius: 6,
    material: {
        pigment: solid { colour: {0, 0, 1} },
        finish: { reflection: 0.5 }
    }
}

{% for y in (0..10) %}
{% for x in (0..10) %}
box {
    lower: { {{ x | times: 2 | minus: 10.5 }}, {{ y | times: 2 | minus: 10.5}}, -0.5},
    upper: { {{ x | times: 2 | minus: 9.5 }}, {{ y | times: 2 | minus: 9.5}}, 0.5}
}
{% endfor %}
{% endfor %}