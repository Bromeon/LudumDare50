extends Spatial

export(bool) var ghost = false

const ghostMaterial = preload("res://Assets/GhostMaterial.tres")


func _ready():
	if ghost:
		applyMaterial(ghostMaterial)

		# Remove colliders
		for node in get_children():
			var collision = node.get_node("CollisionShape")
			collision.queue_free()


func applyMaterial(mat: SpatialMaterial) -> void:
	$Core/Mesh.set_surface_material(0, mat)

	if has_node("Roof"):
		$Roof/Mesh.set_surface_material(0, mat)
