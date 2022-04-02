extends Spatial


# Called when the node enters the scene tree for the first time.
func _ready():
	var scene = preload("res://Scene/Objects/Structure.tscn")
	$SpatialObjects.load(scene)