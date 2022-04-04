extends Spatial

func _ready():
	setPowered(false)

func setPowered(powered: bool):
	if powered:
		$AnimationPlayer.play("Pump")
	else:
		$AnimationPlayer.stop()

