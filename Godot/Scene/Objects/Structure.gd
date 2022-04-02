extends Spatial


func applyMaterial(mat: SpatialMaterial) -> void:
	$Roof/Mesh.set_surface_material(0, mat)
	$Core/Mesh.set_surface_material(0, mat)
