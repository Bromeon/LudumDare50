extends Spatial
#class_name Text2D

# for later: https://www.youtube.com/watch?v=ilBCnt_WE58

export(String) var text setget _setString
export(Color) var textColor setget _setColor

onready var label = $CanvasLayer/CenterContainer/Label

var lateText: String
var lateColor: Color
var init = false


func _setString(s: String) -> void:
	# Note: cannot use onready var if set via editor
	lateText = s
	init = false

func _setColor(c: Color) -> void:
	lateColor = c
	init = false

func _process(_delta):
	if !init:
		$CanvasLayer/CenterContainer/Label.text = lateText
		$CanvasLayer/CenterContainer.modulate = lateColor
		init = true

	var textPos = get_global_transform().origin
	var cam = get_viewport().get_camera()
	var pos2d = cam.unproject_position(textPos)

	var factor = 1#.1 * textPos.distance_to(cam.global_transform.origin)

	label.rect_position = pos2d - label.rect_size / 2.0
	label.rect_scale = factor * Vector2.ONE

	#print("rect scale: ", label.rect_scale)
