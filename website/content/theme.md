+++
title = "Theme"
template = "docgen.html"

[extra]
toc = true

# DO NOT EDIT
# Use `make_docs.sh` to generate content
+++
# Theme file

Launch retherm with the path to your custom theme.

```bash
retherm --theme ./your_theme.toml
```

All theme options have a default; you only need to include options
you would like to override in your theme file.

The screen size is 320x320 pixels, with the origin in the top left.

## Fonts

Font can be specified in the format `"<name>:<size>"` where name is one
of the following:

* Icon: FontAwesome 7.1.0
* Regular: Roboto Regular
* Bold: Roboto Bold

# Main screen

Customize the look and feel of the main thermostat screen.

```toml
[main_screen]
fg_color = "#00ff00"
```

## fg_colour

Colour of text on main screen, default "#ffffff"

## bg_colour

Background colour, default "#000000"

## bg_heat_colour

Background colour when heating is turned on, default "#F17E3B"

## bg_cool_colour

Background colour when cooling is turned on, default "#3B72F1"

## away_icon_center

Position of away icon, default `[160, 230]`

## away_icon

Away icon styling, default `{ icon_font: "Icon:42", icon: "\u{e50b}", colour: "#696969" }`

# Main screen gauge

```toml
[main_screen.gauge]
fg_colour = "#00ff00"
```

## fg_colour

Colour of text, default "#ffffff"

## arc_dia

Diameter of guage arch, default 260

## arc_width

Width of arc, default 20

## arc_start_deg

Arc start angle; 0 degrees at 3'oclock, default 120

## arc_sweed_deg

Sweep angle of arc, default 300

## target_font

Target temp decimal digit font, default "Bold:100"

## target_decimal_font

Target temp fraction digit font, default "Bold:40"

## current_font

Current temp font, default "Regular:20"

## arc_bg_colour

Background fill colour of arc, default "#696969"

## arc_heat_colour

Arc background for heating, default "#E65D10"

## arc_heat_dot_colour

Target heat temp dot colour, default "#C4500E"

## arc_cool_colour

Arc background for cooling, default "#1050E6"

## arc_cool_dot_colour

Target cool temp dot colour, default "#0E44C4""

## arc_target_dot_dia

Diameter of target temp dot, default 30

## arc_temp_dot_dia

Current temp dot diameter, default 12

## arc_temp_dot_colour

Current temp dot colour, default "#C0C0C0"

## arc_temp_text_dia

Diameter of arc current temp label position, default 220

# Mode select screen

Customize the look and feel of the mode select screen.

```toml
[mode_select]
bg_color = "#000000"
```

## bg_colour

Background colour, default "#000000"

## icon_heat_colour

Heat mode icon colour, default "#E65D10"

## icon_cool_colour

Cool mode icon colour, default "#1050E6"

## icon_center

Position of mode icon, default `[160, 25]`

## mode_icon

Mode icon styling, default `{ icon_font: "Icon:42", icon: "\u{f72e}", colour: "#696969" }`

# Mode select list style

## colour

Colour of list item text, default "#d3d3d3"

## label_font

List item font, default "Bold:36"

## icon_font

Selected item icon font, default "Icon:20"

## selected_icon

Selected item icon, default "\u{f00c}"

## highlight_text_colour

Highlighted row text colour, default "#ffffff"

## highlight_rect

Style of the highlight row, default `{ fill_colour: "#", corner_radius: 18 }`

## row_size

List item row size, default `[140, 40]`

