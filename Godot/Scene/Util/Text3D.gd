extends Spatial

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
	var cam = get_viewport().get_camera()
	var pos2d = cam.unproject_position(get_global_transform().origin)

	label.rect_position = pos2d - label.rect_size / 2.0

	#print("2D Pos: ", pos2d)
