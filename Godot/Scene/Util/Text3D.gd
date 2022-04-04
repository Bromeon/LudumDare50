extends Spatial
class_name Text3D

# for later: https://www.youtube.com/watch?v=ilBCnt_WE58

export(String) var text setget _setString
export(Color) var textColor setget _setColor

onready var label = $CanvasLayer/CenterContainer/Label

func _ready():
	pass
	#$Node2D.material = textMaterial

func _setString(s: String) -> void:
	#print("Set string: ", s)

	# Note: cannot use onready var if set via editor
	$CanvasLayer/CenterContainer/Label.text = s


func _setColor(c: Color) -> void:
	$CanvasLayer/CenterContainer.modulate = c


func _process(_delta):
	var textPos = get_global_transform().origin
	var cam = get_viewport().get_camera()
	var pos2d = cam.unproject_position(textPos)

	var factor = 1#.1 * textPos.distance_to(cam.global_transform.origin)

	label.rect_position = pos2d - label.rect_size / 2.0
	label.rect_scale = factor * Vector2.ONE

	#print("rect scale: ", label.rect_scale)
