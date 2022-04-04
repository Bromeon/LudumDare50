extends Spatial

export(bool) var ghost = false
export(String) var baseText = ""

const ghostMaterial = preload("res://Assets/GhostMaterial.tres")

const poweredColor = Color("19a2c5")
const normalColor = Color("c16b24")

func setPowered(powered):
	var mat = $Core/Mesh.get_surface_material(0)
	mat.set_shader_param("tint", poweredColor if powered else normalColor)


func _ready():
	if ghost:
		applyMaterial(ghostMaterial)

		# Remove collision shapes
		for node in get_children():
			var collision = node.get_node_or_null("CollisionShape")
			if collision:
				collision.queue_free()


func updateAmount(amount: int) -> void:
	var tnode = get_node_or_null("Text3D")
	if tnode:
		tnode.text = str(baseText, ": ", amount)


func applyMaterial(mat: SpatialMaterial) -> void:
	$Core/Mesh.set_surface_material(0, mat)

	if has_node("Roof"):
		$Roof/Mesh.set_surface_material(0, mat)
