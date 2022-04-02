extends Spatial


func applyMaterial(mat: SpatialMaterial) -> void:
	$Roof.set_surface_material(0, mat)
	$Core.set_surface_material(0, mat)
