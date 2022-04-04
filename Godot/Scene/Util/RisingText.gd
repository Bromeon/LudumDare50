extends Spatial
class_name RisingText

const SPEED = 3.0

export(Color) var positiveColor
export(Color) var negativeColor

var usedColor

var num: int

func init(number: int):
	num = number
	if num >= 0:
		usedColor = positiveColor
	else:
		usedColor = negativeColor

	print("children: ", get_children())
	$Text2D.text = str(num)
	$Text2D.textColor = usedColor

func _ready():
	pass
	
	

func _process(delta):
# 	$Text3D.textColor = usedColor
# 	$Text3D.text = str(num)
	
	self.translate(Vector3.UP * delta * SPEED)
