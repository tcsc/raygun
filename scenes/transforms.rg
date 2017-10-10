camera {
    location: {0, 0, -15},
    look_at: {0, 0, 0}
}

point_light {
    location: { 50, 50, -100 },
    colour: {1, 1, 1}
}

point_light {
    location: { 0, 0, -100 },
    colour: {0.25, 0.25, 0.25}
}

{% for y in (0..20) %}
{% for x in (0..20) %}
sphere {
    centre: { {{x | minus: 10}}, {{y | minus: 10}}, 0},
    radius: 0.1
}
{% endfor %}
{% endfor %}

group {
    scale: {1, 1, 1},
    translate: {0, 0, 0},
    objects: {
        sphere {
            centre: { 0, 0, 0 },
            radius: 0.5,
            material: {
                pigment: solid { colour: {1, 0, 0} }
            }
        }
    }
}

group {
    scale: {2, 1, 1},
    translate: {0, 2, 0},
    objects: {
        sphere {
            centre: { 0, 0, 0 },
            radius: 0.5,
            material: {
                pigment: solid { colour: {0, 1, 0} }
            }
        }
    }
}

group {
    scale: {3, 1, 1},
    translate: {0, -2, 0},
    rotate: {0, 45, 0},
    objects: {
        sphere {
            centre: { 0, 0, 0 },
            radius: 0.5,
            material: {
                pigment: solid { colour: {0, 0, 1} }
            }
        }
    }
}