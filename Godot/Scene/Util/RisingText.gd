extends Spatial
class_name RisingText

const SPEED = 3.0
const LIFETIME = 1.5

export(Color) var positiveColor
export(Color) var negativeColor

var usedColor: Color

var remainingLife: float = LIFETIME
var num: int

func init(number: int):
	num = number
	if num >= 0:
		usedColor = positiveColor
	else:
		usedColor = negativeColor

	print("children: ", get_children())
	$Text.text = str(num)
	$Text.textColor = usedColor


func _ready():
	pass
	

func _process(delta):
# 	$Text3D.textColor = usedColor
# 	$Text3D.text = str(num)
	remainingLife -= delta
	if remainingLife <= 0:
		queue_free()

	usedColor.a = remainingLife / LIFETIME
	$Text.textColor = usedColor
	
	self.translate(Vector3.UP * delta * SPEED)
