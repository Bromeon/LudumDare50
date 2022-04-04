extends Spatial

func _ready():
	setPowered(false)

func setPowered(powered: bool):
	$Smoke.visible = powered
