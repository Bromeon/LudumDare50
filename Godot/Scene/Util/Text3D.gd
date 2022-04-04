extends Spatial

export(String) var text setget _setString
export(Color) var textColor


func _ready():
	pass # Replace with function body.


func _setString(s: String) -> void:
	#print("Set string: ", s)
	$CanvasLayer/Control/Label.text = s



func _process(_delta):
	var cam = get_viewport().get_camera()
	var pos2d = cam.unproject_position(get_global_transform().origin)
	
	$CanvasLayer/Control.add_color_override("font_color", textColor)
	$CanvasLayer/Control.set("custom_colors/font_color", Color(1,0,0))

	var label = $CanvasLayer/Control/Label
	label.rect_position = pos2d - label.rect_size / 2.0

	#print("2D Pos: ", pos2d)
