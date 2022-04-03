extends Spatial


func applyMaterial(mat: SpatialMaterial) -> void:
	$Core/Mesh.set_surface_material(0, mat)

	if has_node("Roof"):
		$Roof/Mesh.set_surface_material(0, mat)
