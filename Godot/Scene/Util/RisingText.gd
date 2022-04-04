
extends Spatial

export(Color) var textColor
export(String) var text setget _setString



func _setString(s: String) -> void:
	#print("Set string: ", s)
	$CanvasLayer/RichTextLabel.bbcode_text = makeText(s)


func makeText(s: String) -> String:
	var col = textColor.to_html()
	return str("[center][color=", col, "]", s, "[/color][/center]")


func _process(_delta):
	_setString("hei hei")
	var cam = get_viewport().get_camera()
	var pos2d = cam.unproject_position(get_global_transform().origin)

	var label = $CanvasLayer/RichTextLabel
	label.rect_position = pos2d - label.rect_size / 2.0

	#print("2D Pos: ", pos2d)
